use database::{models, Database};

use crate::actions::locked_sync::sync_locked_path_from_database;

pub async fn execute(db: &mut Database) -> anyhow::Result<()> {
    let current_device = models::Device::find_or_create_current(db)?;
    let locked_path = models::DeviceConfig::get_locked_path(db, &current_device)?
        .expect("Cannot find `locked_path` in config");
    let unlocked_path = models::DeviceConfig::get_unlocked_path(db, &current_device)?
        .expect("Cannot find `unlocked_path` in config");

    let _synced_files =
        sync_locked_path_from_database(db, &current_device, locked_path, unlocked_path).await?;

    Ok(())
}
