use std::fs::Metadata;
use std::path::Path;
use std::time::UNIX_EPOCH;

use database_macros::TryFromRow;
use rusqlite::named_params;

use crate::errors::DatabaseResult;
use crate::{models, Database};

use crate::traits::{Insert, InsertMany, TryFromRow, Update, UpdateMany};

use super::File;

/// Convert a std::fs::Metadata into a UNIX epoch u64
#[cfg(target_os = "linux")]
pub fn metadata_to_last_modified(metadata: &Metadata) -> f64 {
    metadata
        .modified()
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

#[derive(TryFromRow, Clone, Debug)]
pub struct FileDevice {
    file_id: i64,
    device_id: i64,
    pub is_unlocked: bool,
    pub is_locked: bool,
    pub last_modified: f64,
}

impl FileDevice {
    /// Build a new `FileDevice`
    pub fn new(
        file: &models::File,
        device: &models::Device,
        is_unlocked: bool,
        is_locked: bool,
        last_modified: f64,
    ) -> Self {
        FileDevice {
            file_id: file.id.expect("missing file_id"),
            device_id: device.id.expect("missing device_id"),
            is_unlocked,
            is_locked,
            last_modified,
        }
    }

    pub fn find_by_paths(
        db: &mut Database,
        paths: &[impl AsRef<Path>],
    ) -> DatabaseResult<Vec<Self>> {
        let tx = db.transaction()?;
        let mut items = vec![];

        for path in paths {
            let path = path.as_ref().to_string_lossy();
            let device = tx.query_row(
                include_str!("sql/file_device/find_by_path.sql"),
                named_params! { ":path": path },
                |row| FileDevice::try_from_row(row),
            )?;
            items.push(device);
        }

        tx.commit()?;
        Ok(items)
    }

    pub fn find_by_files(db: &mut Database, files: &[File]) -> DatabaseResult<Vec<Self>> {
        let tx = db.transaction()?;
        let mut items = vec![];

        for file in files {
            let device = tx.query_row(
                include_str!("sql/file_device/find_by_file.sql"),
                named_params! { ":file_id": file.id },
                |row| FileDevice::try_from_row(row),
            )?;
            items.push(device);
        }

        tx.commit()?;
        Ok(items)
    }
}

impl Insert for FileDevice {
    fn insert(self, db: &Database) -> DatabaseResult<FileDevice> {
        let device = db.query_row(
            include_str!("sql/file_device/insert.sql"),
            named_params! {
                ":file_id": self.file_id,
                ":device_id": self.device_id,
                ":is_unlocked": self.is_unlocked,
                ":is_locked": self.is_locked,
                ":last_modified": self.last_modified
            },
            |row| FileDevice::try_from_row(row),
        )?;

        Ok(device)
    }
}

impl InsertMany for FileDevice {}

impl Update for FileDevice {
    fn update(self, db: &Database) -> DatabaseResult<FileDevice> {
        let file_device = db.query_row(
            include_str!("sql/file_device/update.sql"),
            named_params! {
                ":is_unlocked": self.is_unlocked,
                ":is_locked": self.is_locked,
                ":last_modified": self.last_modified,
                ":file_id": self.file_id,
                ":device_id": self.device_id
            },
            |row| FileDevice::try_from_row(row),
        )?;

        Ok(file_device)
    }
}

impl UpdateMany for FileDevice {}

#[cfg(test)]
mod tests {
    use super::FileDevice;
    use crate::create_in_memory;
    use crate::models::{Device, File};
    use crate::traits::Insert;

    #[test]
    fn test_insert() {
        let database = create_in_memory().unwrap();

        // Prepare file
        let file = File::new(
            "random title".to_string(),
            Default::default(),
            "random unique hash".to_string(),
            0,
        )
        .insert(&database)
        .unwrap();

        // Prepare device
        let device = Device::find_or_create_current(&database).unwrap();

        let to_insert = FileDevice::new(&file, &device, false, false, 0.0);
        to_insert.insert(&database).unwrap();
    }
}
