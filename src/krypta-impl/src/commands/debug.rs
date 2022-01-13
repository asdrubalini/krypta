use std::path::PathBuf;

use database::{models, Database};

pub async fn execute(db: &mut Database) -> anyhow::Result<()> {
    let current_device = models::Device::find_or_create_current(db)?;

    models::DeviceConfig::set_unlocked_path(
        db,
        PathBuf::from("/vault/test_data"),
        &current_device,
    )?;
    models::DeviceConfig::set_locked_path(db, PathBuf::from("/vault/encrypted"), &current_device)?;

    Ok(())
}
