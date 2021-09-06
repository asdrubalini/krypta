use std::path::{Path, PathBuf};

use walkdir::WalkDir;

/// This trait is used in order to strip the "local bits" from a PathBuf
/// so that it can be safely inserted into the database without polluting it
/// with host-specific folders
trait CanonicalizeAndSkipPath<T> {
    fn canonicalize_and_skip_n(&mut self, n: usize) -> T;
}

impl CanonicalizeAndSkipPath<PathBuf> for &Path {
    fn canonicalize_and_skip_n(&mut self, n: usize) -> PathBuf {
        self.canonicalize()
            .unwrap()
            .iter()
            .skip(n)
            .collect::<PathBuf>()
    }
}

/// - Find all files in `full_source_path`, ignoring folders and without following links
/// - Turn DirItem(s) into PathBuf and strip off the host-specific paths in order to
/// have something that we can put into the database
pub fn find_paths_relative(full_source_path: &Path) -> Vec<PathBuf> {
    // /path/to/foo/bar -> 4
    let full_source_path_length = full_source_path.iter().peekable().count();

    WalkDir::new(full_source_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| match e.metadata() {
            Ok(metadata) => metadata.is_file(),
            Err(_) => true,
        })
        .map(|e| e.path().canonicalize_and_skip_n(full_source_path_length))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir_all, remove_dir_all, File};

    use super::*;

    #[test]
    fn test_path_finder() {
        let source_path = Path::new("/tmp/test_dir/foo/bar/");
        create_dir_all(source_path).unwrap();

        for i in 0..256 {
            let mut filename = PathBuf::from(source_path);
            filename.push(format!("file_{}", i));

            File::create(filename).unwrap();
        }

        let relative_paths_found = find_paths_relative(source_path);

        for path in relative_paths_found {
            assert!(path.to_string_lossy().to_string().starts_with("file_"));
        }

        remove_dir_all(source_path).unwrap();
    }
}
