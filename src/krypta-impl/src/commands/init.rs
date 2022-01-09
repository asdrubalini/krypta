use std::path::Path;

use database::{models, Database};

use crate::actions::sync::{sync_database_from_source_path, sync_encrypted_path_from_database};

pub async fn execute(
    database: &mut Database,
    source_path: impl AsRef<Path>,
    encrypted_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let source_path = source_path.as_ref().canonicalize()?;
    let encrypted_path = encrypted_path.as_ref().canonicalize()?;

    let current_device = models::Device::find_or_create_current(database)?;

    // On init the database is empty, so sync::sync_database_from_source_path effectively initialized the database
    let database_report =
        sync_database_from_source_path(database, &source_path, &current_device).await?;
    log::trace!(
        "Database has been populated, {} files inserted",
        database_report.processed_files
    );

    let encryption_report =
        sync_encrypted_path_from_database(database, &current_device, &source_path, &encrypted_path)
            .await?;
    log::trace!(
        "Done with encryption, {} files have been encrypted",
        encryption_report.processed_files
    );

    Ok(())
}
