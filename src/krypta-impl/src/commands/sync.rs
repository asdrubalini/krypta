use database::{models, Database};

use crate::actions::sync::sync_database_from_unlocked_path;

pub async fn execute(db: &mut Database) -> anyhow::Result<()> {
    let unlocked_path = models::DeviceConfig::get_unlocked_path(db)?
        .expect("Cannot find `unlocked_path` in config");

    let current_device = models::Device::find_or_create_current(db)?;

    let database_sync_report =
        sync_database_from_unlocked_path(db, &unlocked_path, &current_device).await?;

    println!(
        "{} files have been added into the database",
        database_sync_report.processed_files
    );

    Ok(())
}
