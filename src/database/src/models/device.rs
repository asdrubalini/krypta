use std::fs::File;
use std::io::Read;

use rusqlite::{named_params, Row};

use crate::{
    errors::DatabaseResult,
    traits::{Insert, Search},
    Database,
};

#[derive(Debug, Clone)]
pub struct Device {
    /// Database internal id
    pub id: Option<i64>,
    /// Platform specific id
    pub platform_id: String,
    /// Friendly name
    pub name: String,
}

impl TryFrom<&Row<'_>> for Device {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(Device {
            id: row.get(0)?,
            platform_id: row.get(1)?,
            name: row.get(2)?,
        })
    }
}

#[cfg(target_os = "linux")]
pub fn get_current_platform_id() -> Result<String, std::io::Error> {
    // On Linux, platform id is `/etc/machine-id`

    let mut f = File::open("/etc/machine-id")?;
    let mut machine_id = String::new();

    f.read_to_string(&mut machine_id)?;

    // Trim newline at the end
    machine_id = machine_id.replace("\n", "");

    Ok(machine_id)
}

#[cfg(target_os = "windows")]
pub fn get_current_platform_id() -> Result<String, std::io::Error> {
    Ok("no-platform-id-on-windows".to_string())
}

impl Device {
    pub fn new<S: AsRef<str>>(platform_id: S, name: S) -> Self {
        let platform_id = platform_id.as_ref().to_owned();
        let name = name.as_ref().to_owned();

        Device {
            id: None,
            platform_id,
            name,
        }
    }

    /// Attempts to find the current device in the database, creating one if it doesn't
    /// exists yet
    pub fn find_or_create_current(db: &Database) -> DatabaseResult<Self> {
        let platform_id = get_current_platform_id()?;
        let mut results = Self::search(db, &platform_id)?;

        if results.is_empty() {
            // Create and insert
            let device = Device::new(&platform_id, &platform_id);
            let device = device.insert(db)?;

            Ok(device)
        } else {
            // Return the existing one
            Ok(results.swap_remove(0))
        }
    }
}

impl Search for Device {
    fn search(db: &Database, query: impl AsRef<str>) -> DatabaseResult<Vec<Device>> {
        let mut stmt = db.prepare(include_str!("sql/device/search.sql"))?;
        let mut rows =
            stmt.query(named_params! { ":platform_id": format!("%{}%", query.as_ref()) })?;

        let mut devices = vec![];
        while let Some(row) = rows.next()? {
            devices.push(Device::try_from(row)?);
        }

        Ok(devices)
    }
}

impl Insert for Device {
    fn insert(self, db: &Database) -> DatabaseResult<Device> {
        let device = db.query_row(
            include_str!("sql/device/insert.sql"),
            named_params! { ":platform_id": self.platform_id, ":name": self.name },
            |row| Device::try_from(row),
        )?;

        Ok(device)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{create_in_memory, traits::Search};

    use super::Device;

    #[test]
    fn test_find_or_create_current() {
        let db = create_in_memory().unwrap();
        let device = Device::find_or_create_current(&db).unwrap();

        let found_device = Device::search(&db, &device.platform_id).unwrap();

        assert_eq!(found_device.len(), 1);
        assert_eq!(
            found_device.first().unwrap().platform_id,
            device.platform_id
        );
    }
}
