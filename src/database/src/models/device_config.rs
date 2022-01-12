use std::path::PathBuf;

use rusqlite::Row;

use crate::{errors::DatabaseResult, Database};

#[derive(Debug, Clone)]
pub struct DeviceConfig {
    pub id: i64,
    pub device_id: String,
    pub locked_path: PathBuf,
    pub unlocked_path: PathBuf,
}

impl TryFrom<&Row<'_>> for DeviceConfig {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(DeviceConfig {
            id: row.get(0)?,
            device_id: row.get(1)?,
            locked_path: PathBuf::from(row.get::<_, String>(2)?),
            unlocked_path: PathBuf::from(row.get::<_, String>(3)?),
        })
    }
}

impl DeviceConfig {
    pub fn get_locked_path(_db: &Database) -> DatabaseResult<PathBuf> {
        // TODO: read from database
        Ok(PathBuf::from("/vault/encrypted/"))
    }

    pub fn get_unlocked_path(_db: &Database) -> DatabaseResult<PathBuf> {
        // TODO: read from database
        Ok(PathBuf::from("/vault/test_data/"))
    }
}
