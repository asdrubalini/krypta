use std::fs::File;
use std::io::Read;

use rusqlite::params;

use crate::{
    errors::{DatabaseError, DatabaseResult},
    traits::{Insert, Search},
    Database,
};

#[derive(Debug, Clone)]
pub struct Device {
    /// Database internal id
    pub id: i64,
    /// Platform specific id
    pub platform_id: String,
    /// Friendly name
    pub name: String,
}

/// A device that can be inserted
#[derive(Debug, Clone)]
pub struct InsertDevice {
    pub platform_id: String,
    pub name: String,
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

impl InsertDevice {
    pub fn new<S: AsRef<str>>(platform_id: S, name: S) -> Self {
        let platform_id = platform_id.as_ref().to_owned();
        let name = name.as_ref().to_owned();

        InsertDevice { platform_id, name }
    }
}

impl Device {
    /// Attempts to find the current device in the database, creating one if it doesn't
    /// exists yet
    pub fn find_or_create_current(db: &Database) -> Result<Self, DatabaseError> {
        let platform_id = get_current_platform_id()?;
        let mut results = Self::search(db, &platform_id)?;

        if results.is_empty() {
            // Create and insert
            let device = InsertDevice::new(&platform_id, &platform_id);
            let device = device.insert(db)?;

            Ok(device)
        } else {
            // Return the existing one
            Ok(results.swap_remove(0))
        }
    }
}

impl Search for Device {
    fn search(db: &Database, query: impl AsRef<str>) -> Result<Vec<Device>, DatabaseError> {
        let mut stmt = db.prepare(include_str!("./sql/device/search.sql"))?;
        let mut rows = stmt.query([format!("%{}%", query.as_ref())])?;

        let mut devices = vec![];
        while let Some(row) = rows.next()? {
            let device = Device {
                id: row.get(0)?,
                platform_id: row.get(1)?,
                name: row.get(2)?,
            };

            devices.push(device);
        }

        Ok(devices)
    }
}

impl Insert<Device> for InsertDevice {
    fn insert(self, db: &Database) -> DatabaseResult<Device> {
        let device = db.query_row(
            include_str!("./sql/device/insert.sql"),
            params![self.platform_id, self.name],
            |row| {
                let device = Device {
                    id: row.get(0)?,
                    platform_id: row.get(1)?,
                    name: row.get(2)?,
                };

                Ok(device)
            },
        )?;

        Ok(device)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::create_in_memory;

    use super::Device;

    #[test]
    fn test_find_or_create_current() {
        let db = create_in_memory().unwrap();
        let device = Device::find_or_create_current(&db).unwrap();
    }
}
