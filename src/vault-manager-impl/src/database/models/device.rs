use std::fs::File;

use async_trait::async_trait;

use crate::database::{errors::DatabaseError, Database};

use super::{Insert, Search};

#[derive(Debug, sqlx::FromRow, Clone)]
pub struct Device {
    // Platform specific id
    pub platform_id: String,
    // Friendly name
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

impl Device {
    /// Build a new `Device`
    pub fn new(platform_id: String, name: String) -> Self {
        Device { platform_id, name }
    }

    /// Attempts to find the current device in the database, creating one if it doesn't
    /// exists yet
    pub async fn find_or_create_current(database: &Database) -> Result<Self, DatabaseError> {
        let platform_id = get_current_platform_id()?;
        let mut results = Self::search(database, &platform_id).await?;

        if results.len() == 0 {
            // Create and inser
            let device = Self::new(platform_id.clone(), platform_id);
            Self::insert(database, device.clone()).await?;
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
impl Insert for Device {
    async fn insert(database: &Database, device: Self) -> Result<(), DatabaseError> {
        sqlx::query(include_str!("./sql/device/insert.sql"))
            .bind(device.platform_id)
            .bind(device.name)
            .execute(database)
            .await?;

        Ok(())
    }
}
