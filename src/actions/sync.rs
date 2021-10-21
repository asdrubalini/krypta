use std::path::PathBuf;

use metadata_fs::{MetadataCollection, PathFinder};
use tokio::task::JoinError;

use crate::database::{
    models::{self, Insertable},
    Database,
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
pub async fn sync_database_from_source_path(
    database: &Database,
    source_path: &PathBuf,
) -> Result<SyncReport, SyncError> {
    // Transform relative path into a full one
    let absolute_source_path =
        std::fs::canonicalize(source_path).map_err(SyncError::SourceFolderNotFound)?;

    log::trace!("Start fetching paths from database");
    // Start fetching files' paths we know from database
    let database_paths_handle = {
        let database = database.clone();
        tokio::spawn(async move { models::File::get_file_paths(&database).await })
    };

    log::trace!("Start finding local files");
    // let local_paths = find_paths_relative(&full_source_path);
    let mut path_finder = PathFinder::with_source_path(&absolute_source_path);

    log::trace!("Done with finding local files");

    // Await for paths from database
    let database_paths = database_paths_handle
        .await
        .map_err(SyncError::TaskError)?
        .map_err(SyncError::DatabaseError)?;

    // Filter out only files that needs to be added to the database
    path_finder.filter_paths(&database_paths);

    // Build a MetadataCollection from PathFinder
    let paths_with_metadata = MetadataCollection::from_path_finder(path_finder).await;

    // Finally build File(s) from MetadataCollection
    let files_to_insert: Vec<models::File> = paths_with_metadata
        .metadatas
        .iter()
        .map(|metadata| models::File::from(metadata))
        .collect();

    log::trace!("Start adding to database");

    // Use the File(s) we just got with the database api
    models::File::insert_many(database, &files_to_insert)
        .await
        .map_err(SyncError::DatabaseError)?;

    let processed_files = files_to_insert.len();

    Ok(SyncReport { processed_files })
}

/// Add missing files in the encrypted path, encrypting them first
pub async fn sync_encrypted_path_from_database(
    database: &Database,
    encrypted_path: &PathBuf,
) -> Result<SyncReport, SyncError> {
    todo!()
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
        let source_path = PathBuf::from("/tmp/test_dir/foo/bar/");
        let files_count = 256;

        let database = create_in_memory().await.unwrap();
        create_dir_all(&source_path).unwrap();

        for i in 0..files_count {
            let mut filename = source_path.clone();
            filename.push(format!("file_{}", i));

            File::create(filename).unwrap();
        }

        let report = sync_database_from_source_path(&database, &source_path)
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