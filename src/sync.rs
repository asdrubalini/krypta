use tokio::task::JoinHandle;
use walkdir::WalkDir;

use crate::database::{models::File, Database};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub enum SyncError {
    DatabaseError(sqlx::Error),
    SourceFolderNotFound { folder: String },
}

/// Adds missing fields into database according to source folder
/// Return value is the amount of files that have been added to the database
pub async fn sync_database_from_source_folder(
    database: &Database,
    source_folder: String,
) -> Result<usize, SyncError> {
    let source_path = Path::new(&source_folder);

    if !source_path.exists() {
        return Err(SyncError::SourceFolderNotFound {
            folder: source_folder.to_owned(),
        });
    }

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

    let mut handles: Vec<JoinHandle<Result<(), sqlx::Error>>> = vec![];

    // Finally add files to database
    for file_to_sync in files_to_sync {
        let database = database.clone();
        let handle = tokio::spawn(async move {
            let title = file_to_sync.file_name().to_string_lossy().to_string();
            let path = file_to_sync
                .path()
                .iter()
                .skip(1) // Remove the first part of the path which is the local `source_folder`
                .collect::<PathBuf>()
                .to_string_lossy()
                .to_string();

            log::info!("Adding {} to the database", path);

            Ok(File::insert(&database, &title, &path, false, false).await?)
        });

        handles.push(handle);
    }

    let files_to_sync_count = handles.len();

    // Wait for all tasks to terminate
    for handle in handles {
        handle.await.unwrap().map_err(SyncError::DatabaseError)?;
    }

    Ok(files_to_sync_count)
}
