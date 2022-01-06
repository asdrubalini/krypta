use std::path::PathBuf;

use crypto::{hash::Sha256ConcurrentFileHasher, traits::ConcurrentCompute};
use fs::PathFinder;

use crate::database::{
    models::{self, Insert},
    Database,
};

/// Final report of sync job, thrown if no fatal errors are encountered
#[derive(Debug)]
pub struct SyncReport {
    pub processed_files: usize,
}

/// Adds missing files into database according to source folder
pub async fn sync_database_from_source_path(
    database: &Database,
    source_path: &PathBuf,
) -> anyhow::Result<SyncReport> {
    // Transform relative path into a full one
    let absolute_source_path = std::fs::canonicalize(source_path)?;

    log::trace!("Start fetching paths from database");
    // Start fetching files' paths we know from database
    let database_paths_handle = {
        let database = database.clone();
        tokio::spawn(async move { models::File::get_file_paths(&database).await })
    };

    log::trace!("Start finding local files");
    // Find all file paths
    let mut path_finder = PathFinder::from_source_path(&absolute_source_path).unwrap();

    log::trace!("Done with finding local files");

    // Await for paths from database
    let database_paths = database_paths_handle.await??;

    // Now that we have files already in database and all the local files,
    // filter out only files that needs to be added to the database

    // TODO: they may have changed, so here we should check the last modified date
    // to make sure that they have not, or we don't try to detect them at all
    // and instead rely on the user with a special `add` command or something like that.
    path_finder.filter_out_paths(&database_paths);

    // Start computing new file's hashes in the background
    let mut hasher = Sha256ConcurrentFileHasher::try_new(&path_finder.get_all_absolute_paths())?;
    let hashes_join = tokio::task::spawn(async move { hasher.start_all().await });

    let files_to_insert = path_finder
        .metadatas
        .iter()
        .map(|(path, metadata)| models::MetadataFile::new(path, metadata));

    // Wait until all requested hashes have been computed
    let hashes = hashes_join.await.unwrap();

    // Put hashes together with files constructing `models::File` objects
    let files_to_insert: Vec<models::File> = files_to_insert
        .map(|file| {
            // Since `crypto::Sha256ConcurrentHasher` expects absolute paths, they need to be constructed here
            let mut absolute_file_path = absolute_source_path.clone();
            absolute_file_path.push(&file.path);

            let hash = hashes.get(&absolute_file_path).expect(&format!(
                "Hash for required file {:?} cannot be found",
                file.path
            ));
            file.into_file(hash.as_hex())
        })
        .collect();

    log::trace!("Start adding to database");

    // Use the File(s) we just got with the database api and insert them all
    models::File::insert_many(database, &files_to_insert).await?;

    let processed_files = files_to_insert.len();

    Ok(SyncReport { processed_files })
}

/// Add missing files in the encrypted path, encrypting them first
pub async fn sync_encrypted_path_from_database(
    _database: &Database,
    _encrypted_path: &PathBuf,
) -> anyhow::Result<SyncReport> {
    todo!()
}

// TODO: uncomment and fix when sync action is fixed
// #[cfg(test)]
// mod tests {
// use std::{
// fs::{create_dir_all, remove_dir_all, File},
// path::PathBuf,
// };

// use crate::database::{create_in_memory, models::Fetchable};

// use super::*;

// #[tokio::test]
// async fn test_database_sync() {
// let source_path = PathBuf::from("/tmp/test_dir/foo/bar/");
// let files_count = 256;

// let database = create_in_memory().await.unwrap();
// create_dir_all(&source_path).unwrap();

// for i in 0..files_count {
// let mut filename = source_path.clone();
// filename.push(format!("file_{}", i));

// File::create(filename).unwrap();
// }

// let report = sync_database_from_source_path(&database, &source_path)
// .await
// .unwrap();

// assert_eq!(report.processed_files, files_count);

// let files = models::File::fetch_all(&database).await.unwrap();
// assert_eq!(files.len(), files_count);

// for file in files {
// assert!(file.path.starts_with("file_"));
// }

// remove_dir_all(source_path).unwrap();
// }
// }
