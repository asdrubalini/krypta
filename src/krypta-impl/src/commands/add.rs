use std::{
    collections::HashMap,
    io::Write,
    path::{Path, PathBuf},
};

use crypto::{
    crypt::FileEncryptBulk, errors::CryptoError, hash::Blake3Concurrent, traits::ComputeBulk,
};
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

/// Encrypt many files
pub async fn encrypt_many_files(
    files: Vec<models::File>,
    virtual_prefix: PathBuf,
    source_root_path: impl AsRef<Path>,
    locked_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let source_root_path = source_root_path.as_ref();
    let locked_path = locked_path.as_ref();

    let virtual_prefix_len = virtual_prefix.iter().count();

    // Start encryption job
    log::trace!("Encryption job started");

    let encryptors = files
        .into_iter()
        .map(|mut file| {
            // remove `virtual_prefix` from File
            let p: PathBuf = PathBuf::from(file.path)
                .iter()
                .skip(virtual_prefix_len)
                .collect();

            file.path = p.to_string_lossy().to_string();

            models::File::try_into_encryptor(file, locked_path, source_root_path)
        })
        .collect::<Result<Vec<_>, CryptoError>>()?;

    let encryptor = FileEncryptBulk::new(encryptors);
    let encryption_status = encryptor.start_all();

    log::trace!("Done with encryption job");

    // Error check
    let errors = encryption_status
        .iter()
        .filter_map(|(_, result)| match result {
            Ok(_) => None,
            Err(error) => Some(error),
        })
        .collect::<Vec<_>>();

    if errors.is_empty() {
        log::info!(
            "Encrypted {} files, with no errors",
            encryption_status.len(),
        );

        Ok(())
    } else {
        for error in &errors {
            println!("Error: {:?}", error);
        }

        log::warn!(
            "Encrypted {} files correctly, {} errors",
            encryption_status.len(),
            errors.len()
        );

        Err(anyhow::Error::msg("Unable to encrypt all files"))
    }
}

/// Add a path `target_path` to database in `prefix`
pub async fn add(db: &mut Database, source_path: PathBuf, virtual_prefix: Option<PathBuf>) {
    let locked_path = Config::get_locked_path();
    let virtual_prefix = virtual_prefix.unwrap_or("".into());

    let pathfinder = PathFinder::from_source_path(&source_path)
        .unwrap_or_else(|error| panic!("Cannot find files in {source_path:?}: {:?}", error));

    let found_paths = pathfinder
        .metadatas
        .keys()
        .map(|path| path.as_path())
        .collect::<Vec<_>>();

    print!(
        "You are inserting {} paths into krypta. Are you sure? y/N ",
        found_paths.len()
    );

    std::io::stdout().lock().flush().unwrap();

    let mut in_buf = String::new();
    std::io::stdin().read_line(&mut in_buf).unwrap();

    in_buf = in_buf.trim().to_lowercase();
    if in_buf.is_empty() || !in_buf.starts_with('y') {
        println!("Stopped.");
        std::process::exit(0);
    }

    let hashes_map = compute_paths_hashes(&source_path, &found_paths).unwrap();

    // first add files to database
    let mut files = vec![];

    for (file_path, metadata) in pathfinder.metadatas {
        let file_hash = hashes_map.get(&file_path).unwrap().to_owned();

        // full_path = prefix + host_relative_path
        let full_path = {
            let mut p = virtual_prefix.clone();
            p.push(file_path);
            p
        };

        let title = full_path.to_string_lossy().to_string();

        let f = models::File::new(title, full_path, file_hash, metadata.len());
        files.push(f);
    }

    let tx = db.transaction().unwrap();

    // insert all files
    let files = models::File::insert_many(&tx, files).unwrap();
    println!("Added {} files to the database", files.len());

    // start encryption job
    encrypt_many_files(files, virtual_prefix, source_path, locked_path)
        .await
        .unwrap();

    tx.commit().unwrap();

    println!("All done.");
}
