use std::{
    collections::HashMap,
    fs::Metadata,
    path::{Path, PathBuf},
};

use crypto::hash::Blake3Hash;
use database::{
    models::{self, Device},
    traits::InsertMany,
    Database,
};

pub fn sync_database_from_unlocked_path(
    db: &mut Database,
    unlocked_path: impl AsRef<Path>,
    device: &Device,
) -> anyhow::Result<Vec<models::File>> {
    // Find paths that need to be inserted into the database
    let paths_requiring_insertion = find_paths_requiring_insertion(db, unlocked_path)?;

    // TODO: run concurrently
    // Find Metadata for paths that need to be inserted
    let metadatas = find_metadata_for_paths(&paths_requiring_insertion)?;
    // Compute hash for paths that need to be inserted
    let hashes = find_hash_for_paths(&paths_requiring_insertion)?;

    // Obtain models::InsertFile and populate the database
    let insert_files = paths_requiring_insertion
        .into_iter()
        .map(|path| {
            // Should always be Some
            let metadata = metadatas.get(&path).unwrap();
            let hash = hashes.get(&path).unwrap();

            models::MetadataFile::new(&path, metadata).into_insert_file(hash.to_string())
        })
        .collect::<Vec<models::InsertFile>>();

    // Insert models::FileDevice
    Ok(models::InsertFile::insert_many(db, &insert_files)?)
}

/// Find all files in `unlocked_path` that need to be inserted into the database
fn find_paths_requiring_insertion(
    db: &Database,
    unlocked_path: impl AsRef<Path>,
) -> anyhow::Result<Vec<PathBuf>> {
    todo!()
}

// TODO: consider using &Path instead of PathBuf
fn find_metadata_for_paths<P: AsRef<Path>>(
    paths: &[P],
) -> anyhow::Result<HashMap<PathBuf, Metadata>> {
    todo!()
}

// TODO: consider using &Path instead of PathBuf
fn find_hash_for_paths<P: AsRef<Path>>(
    paths: &[P],
) -> anyhow::Result<HashMap<PathBuf, Blake3Hash>> {
    todo!()
}
