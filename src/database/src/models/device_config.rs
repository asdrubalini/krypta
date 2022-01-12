use std::path::PathBuf;

use rusqlite::{params, OptionalExtension, Row};

use crate::{errors::DatabaseResult, Database};

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

impl DeviceConfig {
    pub fn get_config(db: &Database) -> DatabaseResult<Option<Self>> {
        let device = Device::find_or_create_current(db)?;

        Ok(db
            .query_row(
                include_str!("sql/device_config/find_by_device.sql"),
                params![device.platform_id],
                |row| DeviceConfig::try_from(row),
            )
            .optional()?)
    }

    pub fn get_locked_path(db: &Database) -> DatabaseResult<Option<PathBuf>> {
        let locked_path = Self::get_config(db)?;
        Ok(locked_path.map(|config| config.locked_path).flatten())
    }

    pub fn get_unlocked_path(db: &Database) -> DatabaseResult<Option<PathBuf>> {
        let unlocked_path = Self::get_config(db)?;
        Ok(unlocked_path.map(|config| config.unlocked_path).flatten())
    }
}
