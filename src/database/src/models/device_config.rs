use std::path::{Path, PathBuf};

use rusqlite::{params, OptionalExtension, Row};

use crate::{
    errors::DatabaseResult,
    traits::{Count, Update},
    Database,
};

use super::Device;

#[derive(Debug, Clone)]
pub struct DeviceConfig {
    pub id: i64,
    pub device_id: i64,
    pub locked_path: Option<PathBuf>,
    pub unlocked_path: Option<PathBuf>,
}

impl TryFrom<&Row<'_>> for DeviceConfig {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        let locked_path = row.get::<_, Option<String>>(2)?.map(PathBuf::from);
        let unlocked_path = row.get::<_, Option<String>>(3)?.map(PathBuf::from);

        Ok(DeviceConfig {
            id: row.get(0)?,
            device_id: row.get(1)?,
            locked_path,
            unlocked_path,
        })
    }
}

pub struct UpdateDeviceConfig {
    pub id: i64,
    pub device_id: i64,
    pub locked_path: Option<PathBuf>,
    pub unlocked_path: Option<PathBuf>,
}

impl From<DeviceConfig> for UpdateDeviceConfig {
    fn from(device_config: DeviceConfig) -> Self {
        UpdateDeviceConfig {
            id: device_config.id,
            device_id: device_config.device_id,
            locked_path: device_config.locked_path,
            unlocked_path: device_config.unlocked_path,
        }
    }
}

impl Update<DeviceConfig> for UpdateDeviceConfig {
    fn update(&self, db: &Database) -> DatabaseResult<DeviceConfig> {
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

impl Count for DeviceConfig {
    fn count(db: &Database) -> DatabaseResult<i64> {
        let count = db.query_row(include_str!("sql/device_config/count.sql"), [], |row| {
            row.get(0)
        })?;
        Ok(count)
    }
}

impl DeviceConfig {
    fn find_or_create_current(db: &Database, device: &Device) -> DatabaseResult<Self> {
        let maybe_config = Self::find_by_device(db, device)?;

        let config = match maybe_config {
            Some(config) => config,
            None => db.query_row(
                include_str!("sql/device_config/create_empty.sql"),
                params![device.id],
                |row| DeviceConfig::try_from(row),
            )?,
        };

        Ok(config)
    }

    fn find_by_device(db: &Database, device: &Device) -> DatabaseResult<Option<Self>> {
        let config = db
            .query_row(
                include_str!("sql/device_config/find_by_device.sql"),
                params![device.id],
                |row| DeviceConfig::try_from(row),
            )
            .optional()?;

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
        let mut config: UpdateDeviceConfig = Self::find_or_create_current(db, device)?.into();
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
        let mut config: UpdateDeviceConfig = Self::find_or_create_current(db, device)?.into();
        config.unlocked_path = Some(unlocked_path.as_ref().to_path_buf());
        config.update(db)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::DeviceConfig;
    use crate::{create_in_memory, models::Device, traits::Count};

    #[test]
    fn test_find_or_create_current() {
        let database = create_in_memory().unwrap();
        let device = Device::find_or_create_current(&database).unwrap();

        DeviceConfig::find_or_create_current(&database, &device).unwrap();
        assert_eq!(DeviceConfig::count(&database).unwrap(), 1);

        DeviceConfig::find_or_create_current(&database, &device).unwrap();
        // Subsequent access should not create more rows
        assert_eq!(DeviceConfig::count(&database).unwrap(), 1);
    }

    #[test]
    fn test_get_and_set_paths() {
        let database = create_in_memory().unwrap();
        let device = Device::find_or_create_current(&database).unwrap();

        let locked_path = DeviceConfig::get_locked_path(&database, &device).unwrap();
        assert_eq!(locked_path, None);

        let unlocked_path = DeviceConfig::get_unlocked_path(&database, &device).unwrap();
        assert_eq!(unlocked_path, None);

        DeviceConfig::set_locked_path(
            &database,
            PathBuf::from("/test/locked_path/foo/bar"),
            &device,
        )
        .unwrap();
        assert_eq!(DeviceConfig::count(&database).unwrap(), 1);

        DeviceConfig::set_unlocked_path(
            &database,
            PathBuf::from("/test/unlocked_path/foo/bar"),
            &device,
        )
        .unwrap();
        assert_eq!(DeviceConfig::count(&database).unwrap(), 1);

        let locked_path = DeviceConfig::get_locked_path(&database, &device).unwrap();
        assert_eq!(
            locked_path,
            Some(PathBuf::from("/test/locked_path/foo/bar"))
        );

        let unlocked_path = DeviceConfig::get_unlocked_path(&database, &device).unwrap();
        assert_eq!(
            unlocked_path,
            Some(PathBuf::from("/test/unlocked_path/foo/bar"))
        );
    }
}
