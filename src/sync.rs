use tokio::task::JoinError;
use walkdir::WalkDir;

use crate::database::{
    models::{File, Insertable, InsertableFile},
    Database,
};
use std::path::{Path, PathBuf};

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

/// Holds all the informations that require fs action about a PathBuf
struct OpenInfo {
    pub path: PathBuf,
    pub size: Option<u32>,
}

impl OpenInfo {
    fn size_or_default(&self) -> u32 {
        self.size.unwrap_or(0)
    }
}

impl From<PathBuf> for OpenInfo {
    fn from(path: PathBuf) -> Self {
        Self { path, size: None }
    }
}

/// Holds a collection of paths together with its OpenInfo, if any
type OpenInfosInner = Vec<OpenInfo>;
struct OpenInfos {
    inner: OpenInfosInner,
}

impl OpenInfos {
    /// Populate the missing OpenInfo(s) with fs access
    async fn populate_all(self, prefix: PathBuf) -> Self {
        let mut populated_paths: Vec<OpenInfo> = Vec::new();

        for open_info in self.inner {
            let mut full_path = prefix.clone();
            full_path.push(&open_info.path);

            let file = tokio::fs::File::open(full_path).await.unwrap();
            let size: u32 = file.metadata().await.unwrap().len().try_into().unwrap();

            let open_info = OpenInfo {
                path: open_info.path,
                size: Some(size),
            };

            populated_paths.push(open_info);
        }

        Self::from(populated_paths)
    }
}

impl From<OpenInfosInner> for OpenInfos {
    fn from(inner: OpenInfosInner) -> Self {
        Self { inner }
    }
}

/// Convert OpenInfos into a vector of InsertableFile(s)
impl From<OpenInfos> for Vec<InsertableFile> {
    fn from(open_infos: OpenInfos) -> Self {
        open_infos
            .inner
            .iter()
            .map(|open_info| InsertableFile {
                title: open_info.path.to_string_lossy().to_string(),
                path: open_info.path.clone(),
                is_remote: false,
                is_encrypted: false,
                size: open_info.size_or_default(),
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

    // Build the OpenInfos struct from all the files that need to be inserted into the database
    // Then, call populate_all() in order to start to fill the parameters that require fs acccess
    let paths_to_sync_with_open_info = OpenInfos::from(
        paths_to_sync
            .map(OpenInfo::from)
            .collect::<OpenInfosInner>(),
    )
    .populate_all(full_source_path)
    .await;

    // Finally build InsertableFile(s) from the just populated paths_to_sync_with_open_info
    let files_to_insert: Vec<InsertableFile> = paths_to_sync_with_open_info.into();

    log::trace!("Start adding to database");

    let result = File::insert_many(database, &files_to_insert)
        .await
        .map_err(SyncError::DatabaseError)?;

    let processed_files = files_to_insert.len();

    Ok(SyncReport { processed_files })
}
