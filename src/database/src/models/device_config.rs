use std::path::{Path, PathBuf};

use rusqlite::{params, OptionalExtension, Row};

use crate::{errors::DatabaseResult, traits::Update, Database};

use super::Device;

#[derive(Debug, Clone)]
pub struct DeviceConfig {
    pub id: i64,
    pub device_id: String,
    pub locked_path: Option<PathBuf>,
    pub unlocked_path: Option<PathBuf>,
}

impl TryFrom<&Row<'_>> for DeviceConfig {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        let locked_path = row.get::<_, String>(2).optional()?.map(PathBuf::from);
        let unlocked_path = row.get::<_, String>(3).optional()?.map(PathBuf::from);

        Ok(DeviceConfig {
            id: row.get(0)?,
            device_id: row.get(1)?,
            locked_path,
            unlocked_path,
        })
    }
}

impl Update for DeviceConfig {
    fn update(&self, db: &Database) -> DatabaseResult<Self> {
        let locked_path = self
            .locked_path
            .to_owned()
            .map(|p| p.to_string_lossy().to_string());
        let unlocked_path = self
            .unlocked_path
            .to_owned()
            .map(|p| p.to_string_lossy().to_string());

        let device_config = db.query_row(
            include_str!("sql/device_config/update.sql"),
            params![self.device_id, locked_path, unlocked_path, self.id],
            |row| DeviceConfig::try_from(row),
        )?;

        Ok(device_config)
    }
}

impl DeviceConfig {
    fn find_or_create_current(db: &Database, device: &Device) -> DatabaseResult<Self> {
        let config = db.query_row(
            include_str!("sql/device_config/find_or_create_current.sql"),
            params![device.id],
            |row| DeviceConfig::try_from(row),
        )?;

        Ok(config)
    }

    /// Get the `locked_path` for the current device from database
    pub fn get_locked_path(db: &Database, device: &Device) -> DatabaseResult<Option<PathBuf>> {
        let locked_path = Self::find_or_create_current(db, device)?.locked_path;
        Ok(locked_path)
    }

    /// Get the `unlocked_path` for the current device from database
    pub fn get_unlocked_path(db: &Database, device: &Device) -> DatabaseResult<Option<PathBuf>> {
        let unlocked_path = Self::find_or_create_current(db, device)?.unlocked_path;
        Ok(unlocked_path)
    }

    /// Set the `locked_path` for the current device into database
    pub fn set_locked_path(
        db: &Database,
        locked_path: impl AsRef<Path>,
        device: &Device,
    ) -> DatabaseResult<()> {
        let mut config = Self::find_or_create_current(db, device)?;
        config.locked_path = Some(locked_path.as_ref().to_path_buf());
        config.update(db)?;
        Ok(())
    }

    /// Set the `unlocked_path` for the current device into database
    pub fn set_unlocked_path(
        db: &Database,
        unlocked_path: impl AsRef<Path>,
        device: &Device,
    ) -> DatabaseResult<()> {
        let mut config = Self::find_or_create_current(db, device)?;
        config.unlocked_path = Some(unlocked_path.as_ref().to_path_buf());
        config.update(db)?;
        Ok(())
    }
}
