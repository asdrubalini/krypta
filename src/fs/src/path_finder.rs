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
                let relative_path = entry.path().canonicalize_and_skip_n(source_path_length)?;
                let metadata = entry.metadata()?;

                Ok((relative_path, metadata))
            })
            .collect::<Result<Vec<(_, _)>, FsError>>()?;

        // Exclude dirs
        let metadatas = metadatas
            .into_iter()
            .filter(|(_, metadata)| metadata.is_file())
            .collect::<HashMap<_, _>>();

        Ok(Self { metadatas })
    }

    /// Filter out paths based on `relative_paths_to_filter`, mutating the struct
    pub fn filter_out_paths(&mut self, relative_paths_to_filter: &[PathBuf]) {
        for path in relative_paths_to_filter {
            self.metadatas.remove(path);
        }
    }
}
