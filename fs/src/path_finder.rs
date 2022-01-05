/// PathFinder is a module that is able to, given a source_directory, recursively
/// find files and strip the host-specific bits from them, obtaining something
/// that can be safely inserted into a database, for example
///
/// `CuttablePathBuf` is a structure that holds a single full path and a `cut_index`,
/// and can provide both relative and absolute paths when needed.
use std::{
    collections::HashMap,
    fs::Metadata,
    path::{Path, PathBuf},
};

use walkdir::WalkDir;

use crate::errors::FsError;

/// This trait is used in order to strip the "local bits" from a PathBuf
/// so that it can be, for example, safely inserted into the database without polluting it
/// with host-specific folders
trait CanonicalizeAndSkipPath<T> {
    fn canonicalize_and_skip_n(&mut self, n: usize) -> Result<T, std::io::Error>;
}

impl CanonicalizeAndSkipPath<PathBuf> for &Path {
    fn canonicalize_and_skip_n(&mut self, n: usize) -> Result<PathBuf, std::io::Error> {
        Ok(self.canonicalize()?.iter().skip(n).collect::<PathBuf>())
    }
}

/// Holds the information about the found files (paths are excluded)
#[derive(Debug)]
pub struct PathFinder {
    source_path: PathBuf,
    pub metadatas: HashMap<PathBuf, Metadata>,
}

impl PathFinder {
    /// Build a PathFinder instance and populate it with file paths from absolute_source_path
    pub fn from_source_path<P: AsRef<Path>>(source_path: P) -> Result<Self, FsError> {
        let source_path = source_path.as_ref().to_owned().canonicalize()?;
        let source_path_length = source_path.iter().peekable().count();

        let metadatas = WalkDir::new(&source_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|res| res.ok())
            // Map into a tuple of (RelativePath, &Metadata)
            .map(|entry| {
                // TODO: handle errors here
                let relative_path = entry
                    .path()
                    .canonicalize_and_skip_n(source_path_length)
                    .unwrap();

                let metadata = entry.metadata().unwrap();

                (relative_path, metadata)
            })
            // Exclude dirs
            .filter(|(_, metadata)| metadata.is_file())
            .collect::<HashMap<_, _>>();

        Ok(Self {
            metadatas,
            source_path,
        })
    }

    /// Filter out paths based on `path_to_filter`, mutating the struct
    pub fn filter_out_paths(&mut self, relative_paths_to_filter: &[PathBuf]) {
        for path in relative_paths_to_filter {
            self.metadatas.remove(path);
        }
    }

    /// Get all paths as absolute
    pub fn get_all_absolute_paths(&self) -> Vec<PathBuf> {
        self.metadatas
            .iter()
            .map(|(path, _)| {
                let mut absolute = self.source_path.clone();
                absolute.push(path);
                absolute
            })
            .collect()
    }
}
