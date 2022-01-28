use std::{fs::create_dir_all, path::PathBuf};

use database::{models, traits::FetchAll, Database};
use fs::PathTree;

pub async fn unlock_structure(db: &mut Database) -> anyhow::Result<()> {
    let current_device = models::Device::find_or_create_current(db)?;
    let unlocked_path = models::DeviceConfig::get_unlocked_path(db, &current_device)?
        .expect("Cannot find `unlocked_path` in config");

    let tree: PathTree = models::File::fetch_all(db)?
        .iter()
        .map(PathBuf::from)
        .collect();

    let directories = tree.directory_structure();

    println!("Unlocking structure in {unlocked_path:?}");

    for directory in directories {
        let mut base_path = unlocked_path.clone();
        base_path.push(directory);

        create_dir_all(base_path)?;
    }

    Ok(())
}
