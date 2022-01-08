use std::path::PathBuf;

use database::{models, Database};

use crate::actions::sync::{sync_database_from_source_path, sync_encrypted_path_from_database};

pub async fn execute(database: &mut Database) -> anyhow::Result<()> {
    // TODO: read from database or cli
    let source_path = PathBuf::new();
    let encrypted_path = PathBuf::new();

    let current_device = models::Device::find_or_create_current(database)?;

    let database_sync_report =
        sync_database_from_source_path(database, &source_path, current_device).await?;

    println!(
        "Added {} new files to the database",
        database_sync_report.processed_files
    );

    let encrypted_path_report =
        sync_encrypted_path_from_database(database, &encrypted_path).await?;

    println!(
        "Added {} new files to the encrypted path",
        encrypted_path_report.processed_files
    );

    Ok(())
}
