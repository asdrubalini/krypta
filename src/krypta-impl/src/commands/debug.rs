use std::path::PathBuf;

use database::{models, Database};

use crate::actions::sync::{sync_database_from_unlocked_path, sync_locked_path_from_database};

pub async fn execute(db: &mut Database) -> anyhow::Result<()> {
    let unlocked_path = PathBuf::from("/vault/test_data/");
    let locked_path = PathBuf::from("/vault/encrypted/");

    let current_device = models::Device::find_or_create_current(db)?;

    let database_sync_report =
        sync_database_from_unlocked_path(db, &unlocked_path, &current_device).await?;

    println!(
        "{} files have been added into the database",
        database_sync_report.processed_files
    );

    let encryption_report =
        sync_locked_path_from_database(db, &current_device, &unlocked_path, &locked_path).await?;

    println!(
        "{} files have been encrypted",
        encryption_report.processed_files
    );

    Ok(())
}
