use std::path::PathBuf;

use database::{models, Database};

use crate::actions::sync::{sync_database_from_unlocked_path, sync_locked_path_from_database};

pub async fn execute(database: &mut Database) -> anyhow::Result<()> {
    // TODO: read from cli
    let unlocked_path = PathBuf::new();
    let locked_path = PathBuf::new();

    let current_device = models::Device::find_or_create_current(database)?;

    let database_sync_report =
        sync_database_from_unlocked_path(database, &unlocked_path, &current_device).await?;

    println!(
        "Added {} new files to the database",
        database_sync_report.processed_files
    );

    let locked_path_report =
        sync_locked_path_from_database(database, &current_device, &unlocked_path, &locked_path)
            .await?;

    println!(
        "Added {} new files to the encrypted path",
        locked_path_report.processed_files
    );

    Ok(())
}
