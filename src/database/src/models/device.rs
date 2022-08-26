use database_macros::{Insert, TableName, TryFromRow};
use rusqlite::named_params;

use crate::{
    errors::DatabaseResult,
    traits::{Insert, Search, TryFromRow},
    Database,
};

#[derive(TableName, TryFromRow, Insert, Debug, Clone)]
pub struct Device {
    /// Database internal id
    pub id: Option<i64>,
    /// Platform specific id
    pub platform_id: String,
    /// Friendly name
    pub name: String,
}

#[cfg(target_os = "macos")]
pub fn get_host_unique_id() -> String {
    use std::process::Command;

    use serde_json::Value;

    let output = Command::new("system_profiler")
        .args(["SPHardwareDataType", "-json"])
        .output()
        .expect("cannot run `system_profiler` command");

    let json_string = String::from_utf8(output.stdout).unwrap();
    let json: Value = serde_json::from_str(&json_string).unwrap();

    json.get("SPHardwareDataType")
        .unwrap()
        .get(0)
        .unwrap()
        .get("platform_UUID")
        .unwrap()
        .as_str()
        .unwrap()
        .to_string()
}

#[cfg(target_os = "linux")]
pub fn get_host_unique_id() -> String {
    use std::fs::File;
    use std::io::Read;
    use uuid::Uuid;

    // On Linux, platform id is `/etc/machine-id`

    let mut f = File::open("/etc/machine-id").expect("cannot open `/etc/machine-id`");
    let mut machine_id = String::new();

    f.read_to_string(&mut machine_id)
        .expect("cannot read `/etc/machine-id`");

    // Trim newline at the end
    machine_id = machine_id.replace('\n', "");

    Uuid::new_v5(&Uuid::NAMESPACE_URL, machine_id.as_bytes()).to_string()
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
        let platform_id = get_host_unique_id();
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
            devices.push(Device::try_from_row(row)?);
        }

        Ok(devices)
    }
}

#[cfg(test)]
pub mod tests {
    use crate::{create_in_memory, traits::Search};

    use super::{get_host_unique_id, Device};

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

    #[test]
    fn test_get_host_unique_id() {
        let platform_id = get_host_unique_id();
        assert_eq!(platform_id.len(), 36);
    }
}
