use std::{
    collections::HashMap,
    fs::Metadata,
    path::{Path, PathBuf},
};

use crypto::{hash::Blake3Concurrent, traits::ComputeBulk};
use database::{
    models::{self, metadata_to_last_modified, Device},
    traits::{InsertMany, UpdateMany},
    Database,
};
use fs::PathFinder;

pub async fn sync_database_from_unlocked_path(
    db: &mut Database,
    unlocked_path: impl AsRef<Path>,
    device: &Device,
) -> anyhow::Result<Vec<models::File>> {
    let unlocked_path = unlocked_path.as_ref();

    let (paths_requiring_insertion, paths_requiring_update) =
        find_paths_requiring_insertion_or_update(db, unlocked_path, device)?;

    let involved_paths = paths_requiring_insertion
        .iter()
        .chain(paths_requiring_update.iter())
        .map(|(item, _)| item)
        .collect::<Vec<_>>();

    // Compute all hashes for files that have been updated or just inserted
    let hashes = find_hashes_for_local_paths(unlocked_path, &involved_paths)?;

    // First handle newly created files
    // Obtain models::InsertFile and populate the database
    let files_to_insert = paths_requiring_insertion
        .iter()
        .map(|(path, metadata)| {
            // Should always be Some
            let hash = hashes.get(path).unwrap();
            models::MetadataFile::new(path, metadata).into_file(hash.to_string())
        })
        .collect::<Vec<models::File>>();

    // Insert models::File
    let inserted_files = models::File::insert_many(db, files_to_insert)?;

    let file_devices_to_insert = inserted_files
        .iter()
        .map(|file| {
            // Should never fail
            let metadata = paths_requiring_insertion.get(&file.path).unwrap();
            models::FileDevice::new(
                file,
                device,
                true,
                false,
                metadata_to_last_modified(metadata),
            )
        })
        .collect::<Vec<_>>();

    // Insert models::FileDevice
    models::FileDevice::insert_many(db, file_devices_to_insert)?;

    // Then handle files that have been updated
    let update_paths = paths_requiring_update
        .iter()
        .map(|(item, _)| item)
        .collect::<Vec<_>>();

    let files_to_update = models::File::find_files_from_paths(db, &update_paths)?
        .into_iter()
        .map(|mut file| {
            // Should never fail
            let hash = hashes.get(&file.path).unwrap();
            file.contents_hash = hash.to_owned();
            file
        })
        .collect::<Vec<_>>();

    let mut updated_files = models::File::update_many(db, files_to_update)?;

    let file_devices_to_update = models::FileDevice::find_by_files(db, &updated_files)?
        .into_iter()
        .zip(updated_files.iter())
        .map(|(mut file_device, file)| {
            // Should never fail
            let metadata = paths_requiring_update.get(&file.path).unwrap();

            // Convert into UpdateFileDevice and mutate last_modified
            file_device.last_modified = metadata_to_last_modified(metadata);

            file_device
        })
        .collect::<Vec<_>>();

    models::FileDevice::update_many(db, file_devices_to_update)?;

    let mut affected_files = inserted_files;
    affected_files.append(&mut updated_files);
    Ok(affected_files)
}

/// Find all files in `unlocked_path` that need to be inserted into the database
/// returned paths are relative and do not contain host-specific bits
fn find_paths_requiring_insertion_or_update(
    db: &Database,
    unlocked_path: impl AsRef<Path>,
    device: &Device,
) -> anyhow::Result<(HashMap<PathBuf, Metadata>, HashMap<PathBuf, Metadata>)> {
    let unlocked_path = unlocked_path.as_ref();

    let local_paths_metadata = find_local_paths_and_metadata(unlocked_path)?;
    let database_paths_last_modified =
        models::File::find_known_paths_with_last_modified(db, device)?;

    let mut require_insertion: HashMap<PathBuf, Metadata> = HashMap::new();
    let mut require_update: HashMap<PathBuf, Metadata> = HashMap::new();

    for (relative_local_path, metadata) in local_paths_metadata {
        // New path never seen in database
        if !database_paths_last_modified.contains_key(&relative_local_path) {
            require_insertion.insert(relative_local_path, metadata);
            continue;
        }

        let device_last_modified = metadata_to_last_modified(&metadata);
        let db_last_modified = *database_paths_last_modified
            .get(&relative_local_path)
            .unwrap();

        // Updated path already present in database
        if device_last_modified > db_last_modified {
            require_update.insert(relative_local_path, metadata);
            continue;
        }
    }

    Ok((require_insertion, require_update))
}

fn find_local_paths_and_metadata(
    unlocked_path: &Path,
) -> anyhow::Result<HashMap<PathBuf, Metadata>> {
    let path_finder = PathFinder::from_source_path(unlocked_path)?;
    Ok(path_finder.metadatas)
}

/// Compute BLAKE3 hashes for files in `unlocked_path`
/// returned paths are relative and do not contain host-specific bits
fn find_hashes_for_local_paths(
    unlocked_path: impl AsRef<Path>,
    relative_paths: &[impl AsRef<Path>],
) -> anyhow::Result<HashMap<PathBuf, String>> {
    let unlocked_path = unlocked_path.as_ref();

    let absolute_paths = relative_paths
        .iter()
        .map(|relative| {
            let mut path = unlocked_path.to_path_buf();
            path.push(relative);
            path
        })
        .collect::<Vec<_>>();

    let hasher = Blake3Concurrent::try_new(&absolute_paths)?;
    let result = hasher.start_all();

    let unlocked_path_len = unlocked_path.iter().count();
    let relative_result = result
        .into_iter()
        .map(|(absolute_path, hash)| {
            // Skip hosts bits
            let relative_path: PathBuf =
                absolute_path.iter().skip(unlocked_path_len).collect();

            (relative_path, hash.to_string())
        })
        .collect::<HashMap<_, _>>();

    Ok(relative_result)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, fs::OpenOptions, io::Write, path::PathBuf};

    use database::{models, traits::FetchAll};
    use tmp::{RandomFill, Tmp};

    use crate::actions::database_sync::sync_database_from_unlocked_path;

    #[tokio::test]
    async fn test_standard_database_sync() {
        const FILES_COUNT: usize = 10_000;

        // Prepare
        let tmp = Tmp::new();
        let created_files = tmp.random_fill(FILES_COUNT, 16);

        let mut database = database::create_in_memory().unwrap();
        let current_device = models::Device::find_or_create_current(&database).unwrap();

        // Populate database
        let processed_files =
            sync_database_from_unlocked_path(&mut database, &tmp.path(), &current_device)
                .await
                .unwrap();

        assert_eq!(processed_files.len(), FILES_COUNT);

        let database_files = models::File::fetch_all(&mut database).unwrap();
        assert_eq!(database_files.len(), FILES_COUNT);

        let database_paths = database_files
            .into_iter()
            .map(|file| file.path)
            .collect::<HashSet<_>>(); // HashSet for faster lookups

        // Make sure that each created file exists in the database
        for file in created_files.iter() {
            let created_file = file.into_iter().skip(3).collect::<PathBuf>();
            assert!(database_paths.contains(&created_file));
        }

        // Subsequent syncs should return zero files
        let processed_files =
            sync_database_from_unlocked_path(&mut database, &tmp.path(), &current_device)
                .await
                .unwrap();

        assert_eq!(processed_files.len(), 0);

        // Create a new file and make sure that it gets detected
        let new_file = tmp.random_fill(1, 128).first().unwrap().to_owned();
        let new_file_relative = tmp.to_relative(&new_file);

        let processed_files =
            sync_database_from_unlocked_path(&mut database, &tmp.path(), &current_device)
                .await
                .unwrap();

        assert_eq!(processed_files.len(), 1);
        assert_eq!(processed_files.first().unwrap().path, new_file_relative);

        // Mutate random file and make sure that it gets detected
        let rand_file = created_files.get(1337).unwrap().to_owned();
        let rand_file_relative = tmp.to_relative(&rand_file);

        // Write "random" data to file
        let mut rand_file = OpenOptions::new().write(true).open(&rand_file).unwrap();
        rand_file.write_all(&[0u8; 128]).unwrap();
        rand_file.flush().unwrap();
        drop(rand_file);

        let processed_files =
            sync_database_from_unlocked_path(&mut database, &tmp.path(), &current_device)
                .await
                .unwrap();

        assert_eq!(processed_files.len(), 1);
        assert_eq!(processed_files.first().unwrap().path, rand_file_relative);
    }
}
