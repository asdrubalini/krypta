use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::{fs::File, sync::Semaphore, task::JoinError};

use crate::database::{
    models::{self, Insertable},
    Database,
};
use crate::utils;

/// Holds all the information we have about a path and its details
#[derive(Clone)]
struct PathInfo {
    pub path: PathBuf,
    pub size: Option<u64>,
}

impl PathInfo {
    /// Retrieve self.size or get default value in case it is not available
    fn size_or_default(&self) -> u64 {
        self.size.unwrap_or(0)
    }

    /// Try fs access and update self.size with path's file size if possible
    async fn try_update_size(&mut self, full_path: PathBuf) -> Option<()> {
        if self.size.is_some() {
            return None;
        }

        let file = File::open(full_path).await.ok()?;
        let size: u64 = file.metadata().await.ok()?.len();

        self.size = Some(size);

        Some(())
    }
}

impl From<&PathBuf> for PathInfo {
    /// Build a PathInfo from a PathBuf, with empty fields
    fn from(path: &PathBuf) -> Self {
        Self {
            path: path.to_owned(),
            size: None,
        }
    }
}

/// Holds a collection of paths together with their info, if available
type PathInfosInner = Vec<PathInfo>;
struct PathInfos {
    inner: PathInfosInner,
}

impl PathInfos {
    /// Try to populate the empty fields in PathInfo(s), returning a new copy
    async fn try_populate_all(self, prefix: PathBuf) -> Self {
        // Use a semaphore in order not to exceed os's max file open count
        let semaphore = Arc::new(Semaphore::new(128));
        let mut handles = Vec::new();

        for path_info in self.inner {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let mut full_path = prefix.clone();

            let handle = tokio::spawn(async move {
                full_path.push(&path_info.path);

                let mut path_info = path_info.clone();
                path_info.try_update_size(full_path).await;

                drop(permit);

                path_info
            });

            handles.push(handle);
        }

        // New inner that will be populated with updated details
        let mut inner = Vec::new();

        // Wait for all tasks
        for handle in handles {
            inner.push(handle.await.unwrap());
        }

        Self { inner }
    }
}

impl From<PathInfosInner> for PathInfos {
    /// Build a PathInfos from its type alias, PathInfosInner
    fn from(inner: PathInfosInner) -> Self {
        Self { inner }
    }
}

/// Convert PathInfos into a vector of File(s)
impl From<PathInfos> for Vec<models::File> {
    fn from(path_infos: PathInfos) -> Self {
        path_infos
            .inner
            .iter()
            .map(|path_info| {
                models::File::new(
                    path_info.path.to_string_lossy().to_string(),
                    path_info.path.clone(),
                    false,
                    false,
                    path_info.size_or_default(),
                )
            })
            .collect::<Vec<models::File>>()
    }
}

#[derive(Debug)]
pub enum SyncError {
    DatabaseError(sqlx::Error),
    SourceFolderNotFound(std::io::Error),
    TaskError(JoinError),
}

/// Final report of sync job, thrown if no fatal errors are encountered
#[derive(Debug)]
pub struct SyncReport {
    pub processed_files: usize,
}

/// Adds missing fields into database according to source folder
pub async fn sync_database_from_source_folder(
    database: &Database,
    source_folder: String,
) -> Result<SyncReport, SyncError> {
    // Transform relative path into a full one
    let full_source_path = std::fs::canonicalize(Path::new(&source_folder))
        .map_err(SyncError::SourceFolderNotFound)?;

    log::trace!("Start fetching paths from database");
    // Start fetching files' paths we know from database
    let database_paths_handle = {
        let database = database.clone();
        tokio::spawn(async move { models::File::get_file_paths(&database).await })
    };

    log::trace!("Start finding local files");
    let local_paths = utils::path_finder::find_paths(&full_source_path);

    log::trace!("Done with finding local files");

    // Await for paths from database
    let database_paths = database_paths_handle
        .await
        .map_err(SyncError::TaskError)?
        .map_err(SyncError::DatabaseError)?;

    // Extract only files that needs to be added to the database
    // Then build the PathInfo structs
    let paths_to_sync = PathInfos::from(
        local_paths
            .iter()
            .filter(|file_path| !database_paths.contains(file_path))
            .map(PathInfo::from)
            .collect::<PathInfosInner>(),
    );

    // Call try_populate_all() in order to start trying to fill the missing parameters
    let paths_to_sync = paths_to_sync.try_populate_all(full_source_path).await;

    // Finally build File(s) from the just populated paths_to_sync_with_path_info
    let files_to_insert: Vec<models::File> = paths_to_sync.into();

    log::trace!("Start adding to database");

    // Use the File(s) we just got with the database api
    models::File::insert_many(database, &files_to_insert)
        .await
        .map_err(SyncError::DatabaseError)?;

    let processed_files = files_to_insert.len();

    Ok(SyncReport { processed_files })
}
