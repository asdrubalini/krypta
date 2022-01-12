use database::{models, Database};

use crate::actions::database_sync::sync_database_from_unlocked_path;

pub async fn execute(db: &mut Database) -> anyhow::Result<()> {
    let current_device = models::Device::find_or_create_current(db)?;
    let unlocked_path = models::DeviceConfig::get_unlocked_path(db, &current_device)?
        .expect("Cannot find `unlocked_path` in config");

    let inserted_files =
        sync_database_from_unlocked_path(db, unlocked_path, &current_device).await?;

    println!(
        "{} files have been added into the database",
        inserted_files.len()
    );

    Ok(())
}
