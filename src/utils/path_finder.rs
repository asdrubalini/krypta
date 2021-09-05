use std::path::PathBuf;

use walkdir::WalkDir;

/// This trait is used in order to strip the "local bits" from a PathBuf
/// so that it can be safely inserted into the database without polluting it
/// with host-specific folders
trait CanonicalizeAndSkipPathBuf {
    fn canonicalize_and_skip_n(&mut self, n: usize) -> Self;
}

impl CanonicalizeAndSkipPathBuf for PathBuf {
    fn canonicalize_and_skip_n(&mut self, n: usize) -> Self {
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
pub fn find_paths(full_source_path: &PathBuf) -> Vec<PathBuf> {
    // /path/to/foo/bar -> 4
    let full_source_path_length = full_source_path.iter().peekable().count();

    let local_paths = WalkDir::new(full_source_path)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| match e.metadata() {
            Ok(metadata) => metadata.is_file(),
            Err(_) => true,
        })
        .map(|e| {
            e.path()
                .to_path_buf()
                .canonicalize_and_skip_n(full_source_path_length)
        })
        .collect();

    local_paths
}
