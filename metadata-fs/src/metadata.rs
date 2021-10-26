use std::{path::PathBuf, sync::Arc};

use chrono::{DateTime, Utc};
use tokio::{fs::File, sync::Semaphore};

use crate::{PathFinder, MAX_CONCURRENT_FILE_OPERATIONS};

#[derive(Clone, Debug)]
pub struct Metadata {
    // The actual Path
    pub path: PathBuf,
    // Optional file size, if found
    pub size: Option<u64>,
    // Optional modified_at, if found
    pub modified_at: Option<DateTime<Utc>>,
}

impl From<&PathBuf> for Metadata {
    fn from(path: &PathBuf) -> Self {
        Self {
            path: path.clone(),
            size: None,
            modified_at: None,
        }
    }
}

impl Metadata {
    /// Retrieve self.size or get default value in case it is not available
    pub fn size_or_default(&self) -> u64 {
        self.size.unwrap_or(0)
    }

    /// Retrieve self.modified_at or get default value in case it is not available
    pub fn modified_at_or_default(&self) -> DateTime<Utc> {
        self.modified_at.unwrap_or(Utc::now())
    }

    /// Try fs access and update fields if needed
    async fn try_update_fields(&mut self, absolute_source_path: PathBuf) -> anyhow::Result<()> {
        // Don't waste time if fs access is not required
        if self.size.is_some() && self.modified_at.is_some() {
            return Ok(());
        }

        let file = File::open(absolute_source_path).await?;
        let metadata = file.metadata().await?;

        if self.size.is_none() {
            let size: u64 = metadata.len();
            self.size = Some(size);
        }

        if self.modified_at.is_none() {
            let modified_at: DateTime<Utc> = metadata.modified()?.into();
            self.modified_at = Some(modified_at);
        }

        Ok(())
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
                match metadata.try_update_fields(absolute_source_path).await {
                    Ok(_) => (),
                    Err(err) => print!("Metadata error: {:?}", err),
                };

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
