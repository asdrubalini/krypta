use tokio::task::{JoinError, JoinHandle};
use walkdir::WalkDir;

use crate::database::{models::File, Database};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum SyncError {
    DatabaseError(sqlx::Error),
    SourceFolderNotFound(std::io::Error),
    FileMovedDuringSync(std::io::Error),
    TaskError(JoinError),
}

#[derive(Debug)]
pub struct SyncReport {
    pub processed_files: usize,
    pub error_count: usize,
}

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

/// Adds missing fields into database according to source folder
/// Return value is the amount of files that have been added to the database
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
    let database_files_handle = {
        let database = database.clone();
        tokio::spawn(async move { File::get_file_paths(&database).await })
    };

    log::trace!("Start finding local files");
    // - Find all files in `source_folder`, ignoring folders and without following links
    // - Turn DirItem(s) into PathBuf and strip off the host-specific paths in order to
    // have something that we can put into the database
    let local_files = WalkDir::new(source_folder)
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
    let database_files = database_files_handle
        .await
        .map_err(SyncError::TaskError)?
        .map_err(SyncError::DatabaseError)?;

    // Extract only files that needs to be added to the database
    let files_to_sync = local_files.filter(|file_path| !database_files.contains(file_path));

    let mut handles: Vec<JoinHandle<Result<(), SyncError>>> = vec![];

    log::trace!("Start adding to database");

    let mut sync_count = 0;

    // Finally add files to database
    for file_to_sync in files_to_sync {
        sync_count += 1;

        let database = database.clone();
        let handle = tokio::spawn(async move {
            let title = file_to_sync.to_string_lossy().to_string();

            log::trace!("Adding {:?} to the database", file_to_sync);

            File::insert(&database, &title, &file_to_sync, false, false)
                .await
                .map_err(SyncError::DatabaseError)
        });

        handles.push(handle);
    }

    let files_to_sync_count = handles.len();
    let mut error_count = 0;

    // Wait for all tasks to terminate
    for handle in handles {
        let res = handle.await.unwrap();

        if res.is_err() {
            error_count += 1;
        }
    }

    Ok(SyncReport {
        processed_files: files_to_sync_count,
        error_count,
    })
}
