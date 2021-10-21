use std::{path::PathBuf, sync::Arc};

use tokio::{fs::File, sync::Semaphore};

use crate::{PathFinder, MAX_CONCURRENT_FILE_OPERATIONS};

#[derive(Clone, Debug)]
pub struct Metadata {
    // The actual Path
    pub path: PathBuf,
    // Optional file size, if found
    pub size: Option<u64>,
}

impl From<&PathBuf> for Metadata {
    fn from(path: &PathBuf) -> Self {
        Self {
            path: path.clone(),
            size: None,
        }
    }
}

impl Metadata {
    /// Retrieve self.size or get default value in case it is not available
    pub fn size_or_default(&self) -> u64 {
        self.size.unwrap_or(0)
    }

    /// Try fs access and update self.size with path's file size if possible
    async fn try_update_size(&mut self, absolute_source_path: PathBuf) -> Option<()> {
        if self.size.is_some() {
            return None;
        }

        let file = File::open(absolute_source_path).await.ok()?;
        let size: u64 = file.metadata().await.ok()?.len();

        self.size = Some(size);

        Some(())
    }
}

pub struct MetadataCollection {
    absolute_source_path: PathBuf,
    pub metadatas: Vec<Metadata>,
}

impl MetadataCollection {
    /// Build MetadataCollection instance from a PathFinder instance populating some fields
    pub async fn from_path_finder(path_finder: PathFinder) -> Self {
        let absolute_source_path = path_finder.absolute_source_path.clone();

        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_FILE_OPERATIONS));
        let mut handles = Vec::new();

        for path in path_finder.paths {
            let mut absolute_source_path = absolute_source_path.clone();
            let permit = semaphore.clone().acquire_owned().await.unwrap();

            let handle = tokio::spawn(async move {
                absolute_source_path.push(&path);

                let mut metadata = Metadata::from(&path);
                metadata.try_update_size(absolute_source_path).await;

                drop(permit);
                metadata
            });

            handles.push(handle);
        }

        let mut metadatas = Vec::new();
        for handle in handles {
            metadatas.push(handle.await.unwrap());
        }

        Self {
            absolute_source_path,
            metadatas,
        }
    }
}