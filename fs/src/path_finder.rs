/// PathFinder is a module that is able to, given a source_directory, recursively
/// find files and strip the host-specific bits from them, obtaining something
/// that can be safely inserted into a database
///
/// `CuttablePathBuf` is a structure that holds a single full path and a `cut_index`,
/// and can provide both relative and absolute paths when needed.
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

/// This struct is used to obtain a reference to both an absolute path and a relative path, without
/// allocating it twice and cutting with an index when necessary
#[derive(Clone, Debug)]
pub struct CuttablePathBuf {
    path: PathBuf,
    cut_index: usize,
}

// TODO: check if we can return absolute and relatives referencing self.path without allocation
impl CuttablePathBuf {
    /// Build a CuttablePathBuf from a Path-like and a cut_index
    pub fn from_path<P: AsRef<Path>>(path: P, cut_index: usize) -> Self {
        Self {
            cut_index,
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Gets the absolute path
    pub fn get_absolute(&self) -> PathBuf {
        self.path.clone()
    }

    /// Gets the relative path
    pub fn get_relative(&self) -> PathBuf {
        self.path.iter().skip(self.cut_index).collect::<PathBuf>()
    }
}

/// Holds the information about the found files (paths are excluded)
pub struct PathFinder {
    pub paths: Vec<CuttablePathBuf>,
}

impl PathFinder {
    /// Build a PathFinder instance and populate it with file paths from absolute_source_path
    pub fn with_source_path<P: AsRef<Path>>(absolute_source_path: P) -> Self {
        let absolute_source_path = absolute_source_path.as_ref();

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
            .map(|e| CuttablePathBuf::from_path(e.path(), absolute_source_path_length))
            .collect();

        Self { paths }
    }

    /// Filter out paths based on `path_to_filter`, mutating the struct
    pub fn filter_out_paths(&mut self, paths_to_filter: &[PathBuf]) {
        let filtered_paths = self
            .paths
            .iter()
            .filter(|path| !paths_to_filter.contains(&path.get_relative().to_path_buf()))
            .map(|path| path.to_owned())
            .collect();

        self.paths = filtered_paths;
    }
}
