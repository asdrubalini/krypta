use std::path::{Path, PathBuf};

use walkdir::WalkDir;

/// This trait is used in order to strip the "local bits" from a PathBuf
/// so that it can be safely inserted into the database without polluting it
/// with host-specific folders
trait CanonicalizeAndSkipPath<T> {
    fn canonicalize_and_skip_n(&mut self, n: usize) -> Result<T, std::io::Error>;
}

impl CanonicalizeAndSkipPath<PathBuf> for &Path {
    fn canonicalize_and_skip_n(&mut self, n: usize) -> Result<PathBuf, std::io::Error> {
        Ok(self.canonicalize()?.iter().skip(n).collect::<PathBuf>())
    }
}

/// Holds the information about the absolute path and all the found files
/// without the host-specific bits
pub struct PathFinder {
    pub absolute_source_path: PathBuf,
    pub paths: Vec<PathBuf>,
}

impl PathFinder {
    /// Build a PathFinder instance and populate it with paths from absolute_source_path
    pub fn with_source_path(absolute_source_path: &Path) -> Self {
        // /path/to/foo/bar -> 4
        let absolute_source_path_length = absolute_source_path.iter().peekable().count();

        // - Find all files in `absolute_source_path`, ignoring folders and without following links
        // - Turn DirItem(s) into PathBuf and strip off the host-specific paths in order to
        // have something that we can put into the database
        let paths = WalkDir::new(absolute_source_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| match e.metadata() {
                Ok(metadata) => metadata.is_file(),
                Err(_) => true,
            })
            .map(|e| {
                e.path()
                    .canonicalize_and_skip_n(absolute_source_path_length)
                    .unwrap()
            })
            .collect();

        Self {
            absolute_source_path: absolute_source_path.to_path_buf(),
            paths,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs::{create_dir_all, remove_dir_all, File};

    use super::*;

    #[test]
    fn test_path_finder() {
        let source_path = Path::new("/tmp/test_dir/path_finder/foo/bar/");
        create_dir_all(source_path).unwrap();

        for i in 0..256 {
            let mut filename = PathBuf::from(source_path);
            filename.push(format!("file_{}", i));

            File::create(filename).unwrap();
        }

        let path_finder = PathFinder::with_source_path(source_path);

        for path in path_finder.paths {
            assert!(path.to_string_lossy().to_string().starts_with("file_"));
        }

        remove_dir_all(source_path).unwrap();
    }
}
