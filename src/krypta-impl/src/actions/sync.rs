use std::path::Path;

use crypto::crypt::{FileEncryptBulk, FileEncryptUnit, AEAD_KEY_SIZE, AEAD_NONCE_SIZE};
use crypto::errors::CryptoError;
use crypto::{hash::Blake3Concurrent, traits::ComputeBulk};
use database::traits::InsertMany;
use database::{
    models::{self, Device},
    Database,
};
use fs::PathFinder;

/// Final report of sync job, thrown if no fatal errors are encountered
#[derive(Debug)]
pub struct DatabaseSyncReport {
    pub processed_files: usize,
}

/// Final report of encryption job, thrown if no fatal errors are encountered
#[derive(Debug)]
pub struct EncryptionReport {
    pub processed_files: usize,
    pub errors_count: usize,
}

/// Add missing files into database according to source folder
pub async fn sync_database_from_unlocked_path(
    database: &mut Database,
    unlocked_path: impl AsRef<Path>,
    current_device: &Device,
) -> anyhow::Result<DatabaseSyncReport> {
    let unlocked_path = unlocked_path.as_ref().to_path_buf();
    log::trace!("Start finding local files");

    // Find all file paths
    let path_finder_handle = {
        let absolute_unlocked_path = unlocked_path.clone();

        tokio::task::spawn_blocking(move || {
            PathFinder::from_source_path(&absolute_unlocked_path).unwrap()
        })
    };

    log::trace!("Start fetching paths from database");
    // Start fetching files' paths we know from database
    let database_paths = models::File::get_file_paths(database)?;

    let mut path_finder = path_finder_handle.await?;
    log::trace!("Done with finding local files");

    // Now that we have files already in database and all the local files,
    // filter out only files that needs to be added to the database
    // TODO: they may have changed, so here we should check the last modified date
    // to make sure that they have not, or we don't try to detect them at all
    // and instead rely on the user with a special `add` command or something like that.
    path_finder.filter_out_paths(&database_paths);

    // Start computing new file's hashes in the background
    let hasher = Blake3Concurrent::try_new(&path_finder.get_all_absolute_paths())?;
    let hashes_join = tokio::task::spawn_blocking(move || hasher.start_all());

    let files_to_insert = path_finder
        .metadatas
        .iter()
        .map(|(path, metadata)| models::MetadataFile::new(path, metadata));

    // Wait until all requested hashes have been computed
    let hashes = hashes_join.await.unwrap();

    // Put hashes together with files constructing `models::File` objects
    let files_to_insert: Vec<models::InsertFile> = files_to_insert
        .map(|file| {
            // Since `crypto::Sha256ConcurrentHasher` expects absolute paths, they need to be constructed here
            let mut absolute_file_path = unlocked_path.clone();
            absolute_file_path.push(&file.path);

            let hash = hashes.get(&absolute_file_path).unwrap_or_else(|| {
                panic!("Hash for required file {:?} cannot be found", file.path)
            });
            file.into_insert_file(hash.to_string())
        })
        .collect();

    log::trace!("Start adding to database");

    // Use the File(s) we just got with the database api and insert them all
    let inserted_files = models::InsertFile::insert_many(database, &files_to_insert)?;

    // For each inserted File, create `FileDevice`s objects which marks each file as being unlocked
    // and not encrypted
    let file_devices = inserted_files
        .into_iter()
        .map(|file| models::FileDevice::new(&file, current_device, true, false))
        .collect::<Vec<_>>();

    models::FileDevice::insert_many(database, &file_devices)?;

    let processed_files = files_to_insert.len();

    Ok(DatabaseSyncReport { processed_files })
}

/// Add missing files in the encrypted path, encrypting them first
pub async fn sync_locked_path_from_database(
    db: &mut Database,
    current_device: &Device,
    unlocked_path: impl AsRef<Path>,
    locked_path: impl AsRef<Path>,
) -> anyhow::Result<EncryptionReport> {
    let unlocked_path = unlocked_path.as_ref().to_path_buf();
    let locked_path = locked_path.as_ref().to_path_buf();

    let need_encryption = models::File::find_need_encryption_files(db, current_device)?;

    // A collection of files and their own locked and unlocked paths
    let encryptors = need_encryption
        .iter()
        .map(|file| {
            let mut unlocked = unlocked_path.clone();
            unlocked.push(&file.path);

            let mut locked = locked_path.clone();
            locked.push(&file.random_hash);

            let key: [u8; AEAD_KEY_SIZE] = file.key.to_owned().try_into().unwrap();
            let nonce: [u8; AEAD_NONCE_SIZE] = file.nonce.to_owned().try_into().unwrap();

            FileEncryptUnit::try_new(unlocked, locked, key, nonce)
        })
        .collect::<Result<Vec<_>, CryptoError>>()?;

    log::trace!("Encryption job started");
    let encryptor = FileEncryptBulk::new(encryptors);
    let status = encryptor.start_all();
    log::trace!("Done with encryption job");

    let processed_files = status.len();
    let errors_count = status.iter().filter(|(_, status)| **status).count();

    // TODO: update FileDevice

    Ok(EncryptionReport {
        processed_files,
        errors_count,
    })
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, path::PathBuf};

    use database::traits::Fetch;
    use tmp::{RandomFill, Tmp};

    use super::*;

    #[tokio::test]
    async fn test_database_sync() {
        const FILES_COUNT: usize = 50_000;

        // Prepare
        let tmp = Tmp::new();
        let created_files = tmp.random_fill(FILES_COUNT, 16);

        let mut database = database::create_in_memory().unwrap();
        let current_device = models::Device::find_or_create_current(&database).unwrap();

        // Populate database
        let report = sync_database_from_unlocked_path(&mut database, &tmp.path(), &current_device)
            .await
            .unwrap();

        assert_eq!(report.processed_files, FILES_COUNT);

        let database_files = models::File::fetch_all(&mut database).unwrap();
        assert_eq!(database_files.len(), FILES_COUNT);

        let database_paths = database_files
            .into_iter()
            .map(|file| file.path)
            .collect::<HashSet<_>>(); // HashSet for faster lookups

        // Make sure that each created file exists in the database
        for file in created_files {
            let created_file = file.into_iter().skip(3).collect::<PathBuf>();
            assert!(database_paths.contains(&created_file));
        }

        // Subsequent syncs should return zero files
        let report = sync_database_from_unlocked_path(&mut database, &tmp.path(), &current_device)
            .await
            .unwrap();

        assert_eq!(report.processed_files, 0);
    }
}
