use std::{
    collections::HashMap,
    fs::Metadata,
    path::{Path, PathBuf},
};

use crypto::{
    hash::{Blake3Concurrent, Blake3Hash},
    traits::ComputeBulk,
};
use database::{
    models::{self, Device},
    traits::InsertMany,
    Database,
};
use fs::PathFinder;

pub async fn sync_database_from_unlocked_path(
    db: &mut Database,
    unlocked_path: impl AsRef<Path>,
    device: &Device,
) -> anyhow::Result<Vec<models::File>> {
    let unlocked_path = unlocked_path.as_ref();

    let (path_metadata_map, relative_paths_requiring_insertion) =
        find_paths_requiring_insertion(db, unlocked_path)?;

    let path_hash_map = find_hash_for_paths(unlocked_path, &relative_paths_requiring_insertion)?;

    // Obtain models::InsertFile and populate the database
    let insert_files = relative_paths_requiring_insertion
        .into_iter()
        .map(|path| {
            // Should always be Some
            let metadata = path_metadata_map.get(&path).unwrap();
            let hash = path_hash_map.get(&path).unwrap();

            models::MetadataFile::new(&path, metadata).into_insert_file(hash.to_string())
        })
        .collect::<Vec<models::InsertFile>>();

    // Insert models::FileDevice
    let inserted_files = models::InsertFile::insert_many(db, &insert_files)?;

    let file_devices = inserted_files
        .iter()
        .map(|file| models::FileDevice::new(&file, device, true, false))
        .collect::<Vec<_>>();

    // Insert models::FileDevice
    models::FileDevice::insert_many(db, &file_devices)?;

    Ok(inserted_files)
}

/// Find all files in `unlocked_path` that need to be inserted into the database
/// returned paths are relative and do not contain host-specific bits
fn find_paths_requiring_insertion(
    db: &Database,
    unlocked_path: impl AsRef<Path>,
) -> anyhow::Result<(HashMap<PathBuf, Metadata>, Vec<PathBuf>)> {
    let unlocked_path = unlocked_path.as_ref().to_path_buf();
    let path_finder_handle = std::thread::spawn(|| PathFinder::from_source_path(unlocked_path));

    let database_paths = models::File::get_file_paths(db)?;
    let mut path_finder = path_finder_handle.join().unwrap()?;

    path_finder.filter_out_paths(&database_paths);

    let relative_paths = path_finder.relative_paths();
    let metadatas = path_finder.metadatas;

    Ok((metadatas, relative_paths))
}

/// Compute BLAKE3 hashes for files in `unlocked_path`
/// returned paths are relative and do not contain host-specific bits
fn find_hash_for_paths(
    unlocked_path: impl AsRef<Path>,
    relative_paths: &[impl AsRef<Path>],
) -> anyhow::Result<HashMap<PathBuf, Blake3Hash>> {
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
                absolute_path.into_iter().skip(unlocked_path_len).collect();

            (relative_path, hash)
        })
        .collect::<HashMap<_, _>>();

    Ok(relative_result)
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
        for file in created_files {
            let created_file = file.into_iter().skip(3).collect::<PathBuf>();
            assert!(database_paths.contains(&created_file));
        }

        // Subsequent syncs should return zero files
        let processed_files =
            sync_database_from_unlocked_path(&mut database, &tmp.path(), &current_device)
                .await
                .unwrap();

        assert_eq!(processed_files.len(), 0);
    }
}
