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

    /// Filter paths based on `path_to_filter`, mutating the struct
    pub fn filter_paths(&mut self, paths_to_filter: &[PathBuf]) {
        let filtered_paths = self
            .paths
            .iter()
            .filter(|path| !paths_to_filter.contains(path))
            .map(|path| path.to_owned())
            .collect::<Vec<PathBuf>>();

        self.paths = filtered_paths;
    }
}
