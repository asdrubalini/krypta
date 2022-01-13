use std::{
    collections::HashMap,
    fs::Metadata,
    path::{Path, PathBuf},
    time::Instant,
};

use walkdir::WalkDir;

use crate::errors::FsError;

/// Holds the information about the found files
#[derive(Debug)]
pub struct PathFinder {
    pub metadatas: HashMap<PathBuf, Metadata>,
}

impl PathFinder {
    /// Build a PathFinder instance and populate it with file paths from absolute_source_path
    pub fn from_source_path<P: AsRef<Path>>(source_path: P) -> Result<Self, FsError> {
        let source_path = source_path.as_ref().to_owned().canonicalize()?;
        let source_path_length = source_path.iter().count();

        log::trace!("starting with search in {:?}", source_path);
        let start = Instant::now();

        // Find paths and relative metadata
        let paths_metadata = WalkDir::new(&source_path)
            .follow_links(false)
            .into_iter()
            .filter_map(|res| res.ok())
            // Map into a tuple of (RelativePath, &Metadata)
            .map(|entry| {
                // Skip host-specific bits
                let relative_path = entry
                    .path()
                    .canonicalize()?
                    .iter()
                    .skip(source_path_length)
                    .collect();
                let metadata = entry.metadata()?;

                Ok((relative_path, metadata))
            })
            .collect::<Result<Vec<(_, _)>, FsError>>()?;

        // Exclude dirs
        let metadatas = paths_metadata
            .into_iter()
            .filter(|(_, metadata)| metadata.is_file())
            .collect::<HashMap<_, _>>();

        log::trace!(
            "took {:?} to find {} files",
            start.elapsed(),
            metadatas.len()
        );

        Ok(Self { metadatas })
    }

    /// Filter out paths based on `relative_paths_to_filter`, mutating the struct
    pub fn filter_out_paths(&mut self, relative_paths_to_filter: &[PathBuf]) {
        for path in relative_paths_to_filter {
            self.metadatas.remove(path);
        }
    }

    /// Get all relative paths
    pub fn relative_paths(&self) -> Vec<PathBuf> {
        self.metadatas
            .iter()
            .map(|(path, _metadata)| path.to_owned())
            .collect()
    }
}
