use std::path::PathBuf;

use rusqlite::Row;

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
