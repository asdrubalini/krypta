use tokio::{sync::Semaphore, task::JoinError};
use walkdir::WalkDir;

use crate::database::{
    models::{File, Insertable, InsertableFile},
    Database,
};
use std::{
    path::{Path, PathBuf},
    sync::Arc,
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

/// Holds all the information we have about a path
#[derive(Clone)]
struct PathInfo {
    pub path: PathBuf,
    pub size: Option<u32>,
}

impl PathInfo {
    /// Retrieve self.size or default value in case it is not available
    fn size_or_default(&self) -> u32 {
        self.size.unwrap_or(0)
    }

    /// Try fs access and get new Self version with updated fields where possible
    async fn try_update_size(&mut self, full_path: PathBuf) -> Option<()> {
        if self.size.is_some() {
            return None;
        }

        let file = tokio::fs::File::open(full_path).await.ok()?;
        let size: u32 = file
            .metadata()
            .await
            .ok()?
            .len()
            .try_into()
            .unwrap_or(u32::MAX);

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
    /// Populate the empty PathInfo(s)
    async fn populate_all(self, prefix: PathBuf) -> Self {
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

/// Convert PathInfos into a vector of InsertableFile(s) files
impl From<PathInfos> for Vec<InsertableFile> {
    fn from(path_infos: PathInfos) -> Self {
        path_infos
            .inner
            .iter()
            .map(|path_info| InsertableFile {
                title: path_info.path.to_string_lossy().to_string(),
                path: path_info.path.clone(),
                is_remote: false,
                is_encrypted: false,
                size: path_info.size_or_default(),
            })
            .collect::<Vec<InsertableFile>>()
    }
}

#[derive(Debug)]
pub enum SyncError {
    DatabaseError(sqlx::Error),
    SourceFolderNotFound(std::io::Error),
    FileMovedDuringSync(std::io::Error),
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

    let full_source_path_length = full_source_path.iter().peekable().count();

    log::trace!("Start fetching paths from database");
    // Start fetching files' paths from database
    let database_paths_handle = {
        let database = database.clone();
        tokio::spawn(async move { File::get_file_paths(&database).await })
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

    // Build the PathInfos struct from all the files that need to be inserted into the database
    let paths_to_sync_with_path_info = PathInfos::from(
        paths_to_sync
            .map(PathInfo::from)
            .collect::<PathInfosInner>(),
    );

    // Call populate_all() in order to start to fill the parameters
    let paths_to_sync_with_path_info = paths_to_sync_with_path_info
        .populate_all(full_source_path)
        .await;

    // Finally build InsertableFile(s) from the just populated paths_to_sync_with_path_info
    let files_to_insert: Vec<InsertableFile> = paths_to_sync_with_path_info.into();

    log::trace!("Start adding to database");

    let result = File::insert_many(database, &files_to_insert)
        .await
        .map_err(SyncError::DatabaseError)?;

    let processed_files = files_to_insert.len();

    Ok(SyncReport { processed_files })
}
