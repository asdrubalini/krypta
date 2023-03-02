use std::{
    collections::HashMap,
    fs::Metadata,
    path::{Path, PathBuf},
};

use crypto::{hash::Blake3Concurrent, traits::ComputeBulk};
use database::{
    models::{self, Device},
    traits::{InsertMany, UpdateMany},
    Database,
};
use fs::PathFinder;

/// Find all files in a certain path that need to be inserted into the database
/// returned paths are relative and do not contain host-specific bits
#[deprecated]
fn find_paths_requiring_insertion_or_update(
    db: &Database,
    unlocked_path: impl AsRef<Path>,
    device: &Device,
) -> anyhow::Result<(HashMap<PathBuf, Metadata>, HashMap<PathBuf, Metadata>)> {
    let unlocked_path = unlocked_path.as_ref();

    let local_paths_metadata = PathFinder::from_source_path(unlocked_path)?.metadatas;

    let database_paths_last_modified =
        models::File::find_unlocked_paths_with_last_modified(db, device)?;

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

/// Compute BLAKE3 hashes for files in `unlocked_path`
/// returned paths are relative and do not contain host-specific bits
#[deprecated]
fn compute_hash_for_paths(
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
            let relative_path: PathBuf = absolute_path.iter().skip(unlocked_path_len).collect();

            (relative_path, hash.to_string())
        })
        .collect::<HashMap<_, _>>();

    Ok(relative_result)
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashSet,
        fs::{remove_file, OpenOptions},
        io::Write,
        path::PathBuf,
    };

    use database::{models, traits::FetchAll};
    use rand::{prelude::SmallRng, Rng, SeedableRng};
    use tmp::{RandomFill, Tmp};

    use crate::actions::database_sync::sync_database_from_unlocked_path;

    #[tokio::test]
    async fn test_standard_database_sync() {
        const FILES_COUNT: usize = 10_000;

        let mut rng = SmallRng::seed_from_u64(3);

        // Prepare
        let tmp = Tmp::random();
        let created_files = tmp.random_fill(FILES_COUNT, &mut rng).unwrap();

        let mut database = database::create_in_memory().unwrap();
        let current_device = models::Device::find_or_create_current(&database).unwrap();

        // Populate database
        let processed_files =
            sync_database_from_unlocked_path(&mut database, &tmp.base_path(), &current_device)
                .await
                .unwrap();

        assert_eq!(processed_files.len(), FILES_COUNT);

        let database_files = models::File::fetch_all(&database).unwrap();
        assert_eq!(database_files.len(), FILES_COUNT);

        let database_paths = database_files
            .into_iter()
            .map(|file| PathBuf::from(&file))
            .collect::<HashSet<_>>(); // HashSet for faster lookups

        let tmp_len = Tmp::prefix_len() + 1;
        // Make sure that each created file exists in the database
        for file in created_files.iter() {
            let created_file = file.iter().skip(tmp_len).collect::<PathBuf>();
            assert!(database_paths.contains(&created_file));
        }

        // Subsequent syncs should return zero files
        let processed_files =
            sync_database_from_unlocked_path(&mut database, &tmp.base_path(), &current_device)
                .await
                .unwrap();

        assert_eq!(processed_files.len(), 0);

        // Create a new file and make sure that it gets detected
        let new_file = tmp
            .random_fill(1, &mut rng)
            .unwrap()
            .first()
            .unwrap()
            .to_owned();
        let new_file_relative = tmp.to_relative(&new_file);

        let processed_files =
            sync_database_from_unlocked_path(&mut database, &tmp.base_path(), &current_device)
                .await
                .unwrap();

        assert_eq!(processed_files.len(), 1);
        assert_eq!(
            PathBuf::from(processed_files.first().unwrap()),
            new_file_relative
        );

        // Mutate random file and make sure that it gets detected
        let rand_file = created_files
            .get(rng.gen_range(0..created_files.len() - 1))
            .unwrap()
            .to_owned();
        let rand_file_relative = tmp.to_relative(&rand_file);

        // Write "random" data to file
        let mut rand_file = OpenOptions::new().write(true).open(&rand_file).unwrap();
        rand_file.write_all(&[0u8; 128]).unwrap();
        rand_file.flush().unwrap();

        let processed_files =
            sync_database_from_unlocked_path(&mut database, &tmp.base_path(), &current_device)
                .await
                .unwrap();

        assert_eq!(processed_files.len(), 1);
        assert_eq!(
            PathBuf::from(processed_files.first().unwrap()),
            rand_file_relative
        );

        // Remove random file and make sure that nothing happens
        let rand_file = created_files
            .get(rng.gen_range(0..created_files.len() - 1))
            .unwrap()
            .to_owned();

        remove_file(&rand_file).unwrap();

        let processed_files =
            sync_database_from_unlocked_path(&mut database, &tmp.base_path(), &current_device)
                .await
                .unwrap();

        assert_eq!(processed_files.len(), 0);
    }
}
