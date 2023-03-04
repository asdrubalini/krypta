use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crypto::{hash::Blake3Concurrent, traits::ComputeBulk};
use database::{models, traits::InsertMany, Database};
use fs::PathFinder;

use crate::utils::config::Config;

/// Compute BLAKE3 hashes for files in `unlocked_path`
/// returned paths are relative and do not contain host-specific bits
fn compute_paths_hashes(
    root_path: impl AsRef<Path>,
    files: &[impl AsRef<Path>],
) -> anyhow::Result<HashMap<PathBuf, String>> {
    let root_path = root_path.as_ref();

    let absolute_paths = files
        .iter()
        .map(|relative| {
            let mut path = root_path.to_path_buf();
            path.push(relative);
            path
        })
        .collect::<Vec<_>>();

    let hasher = Blake3Concurrent::try_new(&absolute_paths)?;
    let result = hasher.start_all();

    let root_path_len = root_path.iter().count();
    let relative_result = result
        .into_iter()
        .map(|(absolute_path, hash)| {
            // Skip hosts bits
            let relative_path: PathBuf = absolute_path.iter().skip(root_path_len).collect();

            (relative_path, hash.to_string())
        })
        .collect::<HashMap<_, _>>();

    Ok(relative_result)
}

/// encrypt files into `locked_path`
//pub async fn sync_locked_path_from_database(
//db: &mut Database,
//current_device: &Device,
//locked_path: impl AsRef<Path>,
//unlocked_path: impl AsRef<Path>,
//) -> anyhow::Result<()> {
//let unlocked_path = unlocked_path.as_ref();
//let locked_path = locked_path.as_ref();

//// Find paths that need encryption
//let need_encryption = models::File::find_need_encryption(db, current_device)?;

//// Start encryption job
//log::trace!("Encryption job started");
//let encryptor = files_into_encryptor(need_encryption, locked_path, unlocked_path)?;
//let encryption_status = encryptor.start_all();
//log::trace!("Done with encryption job");

//// Error check
//let errors_count = encryption_status
//.iter()
//.filter(|(_, status)| !(**status))
//.count();
//log::info!(
//"Encrypted {} files, with {} errors",
//encryption_status.len(),
//errors_count
//);

//// Get file_device(s) and mark them as encrypted
//let file_devices = encryption_status_into_file_device(db, encryption_status, unlocked_path)?;
//let file_devices: Vec<models::FileDevice> = file_devices
//.into_iter()
//.map(|mut file_device| {
//file_device.is_locked = true;
//file_device
//})
//.collect();

//models::FileDevice::update_many(db, file_devices)?;

//Ok(())
//}

/// Add a path `target_path` to database in `prefix`
pub async fn add(db: &mut Database, target_path: PathBuf, virtual_prefix: Option<PathBuf>) {
    let locked_path = Config::get_locked_path();
    let virtual_prefix = virtual_prefix.unwrap_or("".into());

    let found_files = PathFinder::from_source_path(&target_path)
        .unwrap_or_else(|error| panic!("Cannot find files in {target_path:?}: {:?}", error));

    let hashes_map = compute_paths_hashes(
        &target_path,
        &found_files
            .metadatas
            .iter()
            .map(|(path, _metadata)| path.as_path())
            .collect::<Vec<_>>(),
    )
    .unwrap();

    // first add files to database
    let mut files = vec![];

    for (file_path, metadata) in found_files.metadatas {
        let file_hash = hashes_map.get(&file_path).unwrap().to_owned();
        let title = file_path.to_string_lossy().to_string();

        // full_path = prefix + host_relative_path
        let full_path = {
            let mut p = virtual_prefix.clone();
            p.push(file_path);
            p
        };

        let f = models::File::new(title, full_path, file_hash, metadata.len());
        files.push(f);
    }

    let mut tx = db.transaction().unwrap();

    // insert all files
    models::File::insert_many(&mut tx, files).unwrap();

    // start encryption job
    // TODO

    tx.commit().unwrap();

    todo!()
}
