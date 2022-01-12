use std::path::Path;

use database::{models, Database};

use crate::actions::sync::{sync_database_from_unlocked_path, sync_locked_path_from_database};

pub async fn execute(
    database: &mut Database,
    unlocked_path: impl AsRef<Path>,
    locked_path: impl AsRef<Path>,
) -> anyhow::Result<()> {
    let unlocked_path = unlocked_path.as_ref().canonicalize()?;
    let locked_path = locked_path.as_ref().canonicalize()?;

    let current_device = models::Device::find_or_create_current(database)?;

    let database_report =
        sync_database_from_unlocked_path(database, &unlocked_path, &current_device).await?;
    log::trace!(
        "Database has been populated, {} files inserted",
        database_report.processed_files
    );

    let encryption_report =
        sync_locked_path_from_database(database, &current_device, &unlocked_path, &locked_path)
            .await?;
    log::trace!(
        "Done with encryption, {} files have been encrypted",
        encryption_report.processed_files
    );

    Ok(())
}
