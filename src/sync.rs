use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::{fs::File, sync::Semaphore, task::JoinError};
use walkdir::WalkDir;

use crate::database::{
    models::{self, Insertable},
    Database,
};

/// This trait is used in order to strip the "local bits" from a PathBuf
/// so that it can be safely inserted into the database without polluting it
/// with host-specific folders
trait CanonicalizeAndSkipPathBuf {
    fn canonicalize_and_skip_n(&mut self, n: usize) -> Self;
}

impl CanonicalizeAndSkipPathBuf for PathBuf {
    fn canonicalize_and_skip_n(&mut self, n: usize) -> Self {
        self.canonicalize()
            .unwrap()
            .iter()
            .skip(n)
            .collect::<PathBuf>()
    }
}

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

impl From<PathBuf> for PathInfo {
    /// Build a PathInfo from a PathBuf, with empty fields
    fn from(path: PathBuf) -> Self {
        Self { path, size: None }
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

    // /path/to/foo/bar -> 4
    let full_source_path_length = full_source_path.iter().peekable().count();

    log::trace!("Start fetching paths from database");
    // Start fetching files' paths we know from database
    let database_paths_handle = {
        let database = database.clone();
        tokio::spawn(async move { models::File::get_file_paths(&database).await })
    };

    log::trace!("Start finding local files");
    // - Find all files in `source_folder`, ignoring folders and without following links
    // - Turn DirItem(s) into PathBuf and strip off the host-specific paths in order to
    // have something that we can put into the database
    let local_paths = WalkDir::new(source_folder)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| match e.metadata() {
            Ok(metadata) => metadata.is_file(),
            Err(_) => true,
        })
        .map(|e| {
            e.path()
                .to_path_buf()
                .canonicalize_and_skip_n(full_source_path_length)
        });

    log::trace!("Done with finding local files");

    // Await for paths from database
    let database_paths = database_paths_handle
        .await
        .map_err(SyncError::TaskError)?
        .map_err(SyncError::DatabaseError)?;

    // Extract only files that needs to be added to the database
    let paths_to_sync = local_paths.filter(|file_path| !database_paths.contains(file_path));

    // Build the PathInfos struct from all the files we need to insert into the database
    let paths_to_sync_with_path_info = PathInfos::from(
        paths_to_sync
            .map(PathInfo::from)
            .collect::<PathInfosInner>(),
    );

    // Call try_populate_all() in order to start trying to fill the missing parameters
    let paths_to_sync_with_path_info = paths_to_sync_with_path_info
        .try_populate_all(full_source_path)
        .await;

    // Finally build File(s) from the just populated paths_to_sync_with_path_info
    let files_to_insert: Vec<models::File> = paths_to_sync_with_path_info.into();

    log::trace!("Start adding to database");

    // Use the File(s) we just got with the database api
    models::File::insert_many(database, &files_to_insert)
        .await
        .map_err(SyncError::DatabaseError)?;

    let processed_files = files_to_insert.len();

    Ok(SyncReport { processed_files })
}
