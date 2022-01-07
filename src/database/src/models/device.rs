use std::fs::File;

use async_trait::async_trait;

use crate::{errors::DatabaseError, Database};

use super::traits::{Insert, Search};

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct Device {
    /// Database internal id
    pub id: i64,
    /// Platform specific id
    pub platform_id: String,
    /// Friendly name
    pub name: String,
}

/// A device that can be inserted
pub struct InsertDevice {
    pub platform_id: String,
    pub name: String,
}

#[cfg(target_os = "linux")]
pub fn get_current_platform_id() -> Result<String, std::io::Error> {
    use std::io::Read;

    // On Linux, platform id is `/etc/machine-id`

    let mut f = File::open("/etc/machine-id")?;
    let mut machine_id = String::new();

    f.read_to_string(&mut machine_id)?;

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
    /// Build a new `Device`
    // pub fn new(platform_id: String, name: String) -> Self {
    // Device { platform_id, name }
    // }

    /// Attempts to find the current device in the database, creating one if it doesn't
    /// exists yet
    pub async fn find_or_create_current(database: &Database) -> Result<Self, DatabaseError> {
        let platform_id = get_current_platform_id()?;
        let mut results = Self::search(database, &platform_id).await?;

        if results.len() == 0 {
            // Create and inser
            let device = InsertDevice::new(&platform_id, &platform_id);
            let device = device.insert(database).await?;

            Ok(device)
        } else {
            // Return the existing one
            Ok(results.swap_remove(0))
        }
    }
}

#[async_trait]
impl Search for Device {
    async fn search(database: &Database, query: &str) -> Result<Vec<Device>, DatabaseError> {
        let devices = sqlx::query_as::<_, Device>(include_str!("./sql/device/search.sql"))
            .bind(format!("%{}%", query))
            .fetch_all(database)
            .await?;

        Ok(devices)
    }
}

#[async_trait]
impl Insert<Device> for InsertDevice {
    async fn insert(self, database: &Database) -> Result<Device, DatabaseError> {
        let device = sqlx::query_as::<_, Device>(include_str!("./sql/device/insert.sql"))
            .bind(self.platform_id)
            .bind(self.name)
            .fetch_one(database)
            .await?;

        Ok(device)
    }
}
