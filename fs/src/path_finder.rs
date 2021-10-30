/// PathFinder is a module that is able to, given a source_directory, recursively
/// find files and strip the host-specific bits from them, obtaining something
/// that can be safely inserted into a database
///
/// `CuttablePathBuf` is a structure that holds a single full path and a `cut_index`,
/// and can provide both relative and absolute paths when needed.
use std::{
    collections::HashMap,
    fs::Metadata,
    hash::Hash,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

/// A relative Path which can also be turned into an absolute Path
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelativePath<'a> {
    source_path: &'a Path,
    relative_file_path: PathBuf,
}

impl<'a> Hash for RelativePath<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.relative_file_path.hash(state);
    }
}

impl<'a> RelativePath<'a> {
    /// Build a CuttablePathBuf from a Path-like and a cut_index
    fn from_path<'b>(source_path: &'a Path, absolute_file_path: PathBuf, cut_index: usize) -> Self {
        let relative_file_path = absolute_file_path
            .iter()
            .skip(cut_index)
            .collect::<PathBuf>();

        Self {
            source_path,
            relative_file_path,
        }
    }

    /// Gets the absolute path
    pub fn get_absolute(&self) -> PathBuf {
        let mut absolute = self.source_path.to_path_buf();
        absolute.push(&self.relative_file_path);
        absolute
    }

    /// Gets the relative path
    pub fn get_relative(&self) -> &Path {
        &self.relative_file_path
    }
}

/// Holds the information about the found files (paths are excluded)
#[derive(Debug)]
pub struct PathFinder<'a> {
    source_path: &'a PathBuf,
    pub metadatas: HashMap<RelativePath<'a>, Metadata>,
}

impl<'a> PathFinder<'a> {
    /// Build a PathFinder instance and populate it with file paths from absolute_source_path
    pub fn from_source_path<P: AsRef<Path>>(source_path: P) -> Self {
        let source_path = source_path.as_ref().to_owned();
        let cut_index = source_path.iter().peekable().count();

        let metadatas = WalkDir::new(source_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|res| res.ok())
            // Map into a tuple of (RelativePath, &Metadata)
            .map(|entry| {
                let absolute_file_path = entry.path().to_path_buf();

                let relative_path =
                    RelativePath::from_path(source_path.as_path(), absolute_file_path, cut_index);
                let metadata = entry.metadata().unwrap();

                (relative_path, metadata)
            })
            // Exclude dirs
            .filter(|(_, metadata)| metadata.is_file())
            .collect::<HashMap<_, _>>();

        Self {
            metadatas,
            source_path,
        }
    }

    /// Filter out paths based on `path_to_filter`, mutating the struct
    pub fn filter_out_paths(&mut self, paths_to_filter: &[PathBuf]) {
        for path in paths_to_filter {
            self.metadatas.remove(path);
        }
    }
}
