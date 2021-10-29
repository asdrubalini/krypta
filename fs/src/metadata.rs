/// Metadata is a module that allows to fetch metadata details from the filesystem such as file size
/// and file's modified date. This is usually used in pair with PathFinder
use std::sync::Arc;

use chrono::{DateTime, Utc};
use tokio::{fs::File, sync::Semaphore};

use crate::{path_finder::CuttablePathBuf, PathFinder, MAX_CONCURRENT_FILE_OPERATIONS};

/// Holds a single file's Metadata
#[derive(Clone, Debug)]
pub struct Metadata {
    // The actual Path
    pub path: CuttablePathBuf,
    // Optional file size, if found
    pub size: Option<u64>,
    // Optional modified_at, if found
    pub modified_at: Option<DateTime<Utc>>,
}

// TODO: maybe don't allocate here
impl From<&CuttablePathBuf> for Metadata {
    fn from(cuttable_path_buf: &CuttablePathBuf) -> Self {
        Self {
            path: cuttable_path_buf.clone(),
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
    async fn try_update_fields(&mut self) -> anyhow::Result<()> {
        // Don't waste time if fs access is not required
        if self.size.is_some() && self.modified_at.is_some() {
            return Ok(());
        }

        let file = File::open(self.cuttable_path_buf.get_absolute()).await?;
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

/// A collection of Metadata objects
pub struct MetadataCollection {
    pub metadatas: Vec<Metadata>,
}

impl MetadataCollection {
    /// Build MetadataCollection instance from a PathFinder instance populating some fields
    pub async fn from_path_finder(path_finder: PathFinder) -> Self {
        // let absolute_source_path = path_finder.absolute_source_path.clone();

        let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT_FILE_OPERATIONS));
        let mut handles = Vec::new();

        for path in path_finder.paths {
            let permit = semaphore.clone().acquire_owned().await.unwrap();

            let handle = tokio::spawn(async move {
                let relative_path = path.get_relative();

                let mut metadata = Metadata::from(&path);
                match metadata.try_update_fields(&path).await {
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

        Self { metadatas }
    }
}
