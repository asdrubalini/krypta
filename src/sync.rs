use tokio::task::JoinHandle;
use walkdir::WalkDir;

use crate::database::{models::File, Database};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum SyncError {
    DatabaseError(sqlx::Error),
    SourceFolderNotFound(std::io::Error),
    FileMovedDuringSync(std::io::Error),
}

#[derive(Debug)]
pub struct SyncReport {
    pub processed_files: usize,
    pub error_count: usize,
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

    // Find all files in source_path, ignoring folders
    let local_files = WalkDir::new(source_folder)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| match e.metadata() {
            Ok(metadata) => metadata.is_file(),
            Err(_) => true,
        });

    // Fetch file paths from database
    let database_files = File::get_file_paths(database)
        .await
        .map_err(SyncError::DatabaseError)?;

    // Extract only files that needs to be added to the database
    let files_to_sync = local_files.filter(|file| {
        let local_file_name = file.path().to_string_lossy().to_string();
        !database_files.contains(&local_file_name)
    });

    let mut handles: Vec<JoinHandle<Result<(), SyncError>>> = vec![];

    // Finally add files to database
    for file_to_sync in files_to_sync {
        let database = database.clone();
        let handle = tokio::spawn(async move {
            let title = file_to_sync.file_name().to_string_lossy().to_string();

            // Resolve path into absolute and strip the local bit keeping only the part after `source_folder`
            // which will be stored into the database
            let path = file_to_sync
                .path()
                .canonicalize()
                .map_err(SyncError::FileMovedDuringSync)?
                .iter()
                .skip(full_source_path_length)
                .collect::<PathBuf>()
                .to_string_lossy()
                .to_string();

            log::info!("Adding {} to the database", path);

            File::insert(&database, &title, &path, false, false)
                .await
                .map_err(SyncError::DatabaseError)
        });

        handles.push(handle);
    }

    let files_to_sync_count = handles.len();
    let mut error_count = 0;

    // Wait for all tasks to terminate
    for handle in handles {
        let res = handle.await;

        if res.is_err() {
            error_count += 1;
        }
    }

    Ok(SyncReport {
        processed_files: files_to_sync_count,
        error_count,
    })
}
