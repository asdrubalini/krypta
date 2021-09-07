use std::path::Path;

use tokio::task::JoinError;

use crate::utils::path_finder::find_paths_relative;
use crate::{
    database::{
        models::{self, Insertable},
        Database,
    },
    utils::path_info::{PathInfo, PathInfos},
};

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
    source_path: String,
) -> Result<SyncReport, SyncError> {
    // Transform relative path into a full one
    let full_source_path =
        std::fs::canonicalize(Path::new(&source_path)).map_err(SyncError::SourceFolderNotFound)?;

    log::trace!("Start fetching paths from database");
    // Start fetching files' paths we know from database
    let database_paths_handle = {
        let database = database.clone();
        tokio::spawn(async move { models::File::get_file_paths(&database).await })
    };

    log::trace!("Start finding local files");
    let local_paths = find_paths_relative(&full_source_path);

    log::trace!("Done with finding local files");

    // Await for paths from database
    let database_paths = database_paths_handle
        .await
        .map_err(SyncError::TaskError)?
        .map_err(SyncError::DatabaseError)?;

    // Extract only files that needs to be added to the database
    // Then build the PathInfo structs
    let paths_to_sync: PathInfos = local_paths
        .iter()
        .filter(|file_path| !database_paths.contains(file_path))
        .map(PathInfo::from)
        .collect();

    // Call try_populate_all() in order to start trying to fill the missing parameters
    let paths_to_sync = paths_to_sync.try_populate_all(full_source_path).await;

    // Finally build File(s) from the just populated paths
    let files_to_insert: Vec<models::File> = paths_to_sync.into();

    log::trace!("Start adding to database");

    // Use the File(s) we just got with the database api
    models::File::insert_many(database, &files_to_insert)
        .await
        .map_err(SyncError::DatabaseError)?;

    let processed_files = files_to_insert.len();

    Ok(SyncReport { processed_files })
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{create_dir_all, remove_dir_all, File},
        path::PathBuf,
    };

    use crate::database::{create_in_memory, models::Fetchable};

    use super::*;

    #[tokio::test]
    async fn test_database_sync() {
        let source_path = Path::new("/tmp/test_dir/foo/bar/");
        let files_count = 256;

        let database = create_in_memory().await.unwrap();
        create_dir_all(source_path).unwrap();

        for i in 0..files_count {
            let mut filename = PathBuf::from(source_path);
            filename.push(format!("file_{}", i));

            File::create(filename).unwrap();
        }

        let report =
            sync_database_from_source_folder(&database, source_path.to_string_lossy().to_string())
                .await
                .unwrap();

        assert_eq!(report.processed_files, files_count);

        let files = models::File::fetch_all(&database).await.unwrap();
        assert_eq!(files.len(), files_count);

        for file in files {
            assert!(file.path.starts_with("file_"));
        }

        remove_dir_all(source_path).unwrap();
    }
}
