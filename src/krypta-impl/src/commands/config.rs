use std::path::PathBuf;

use database::{models, Database};

pub async fn set_unlocked(db: &Database, unlocked_path: PathBuf) -> anyhow::Result<()> {
    let device = models::Device::find_or_create_current(db)?;
    models::DeviceConfig::set_unlocked_path(db, unlocked_path, &device)?;
    Ok(())
}

pub async fn set_locked(db: &Database, locked_path: PathBuf) -> anyhow::Result<()> {
    let device = models::Device::find_or_create_current(db)?;
    models::DeviceConfig::set_locked_path(db, locked_path, &device)?;
    Ok(())
}
