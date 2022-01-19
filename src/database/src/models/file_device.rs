use std::fs::Metadata;
use std::path::Path;
use std::time::UNIX_EPOCH;

use rusqlite::{params, Row};

use crate::errors::DatabaseResult;
use crate::{models, Database};

use crate::traits::{Insert, InsertMany, Update, UpdateMany};

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

#[derive(Clone, Debug)]
pub struct FileDevice {
    file_id: i64,
    device_id: i64,
    pub is_unlocked: bool,
    pub is_encrypted: bool,
    pub last_modified: f64,
}

impl TryFrom<&Row<'_>> for FileDevice {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(FileDevice {
            file_id: row.get(0)?,
            device_id: row.get(1)?,
            is_unlocked: row.get(2)?,
            is_encrypted: row.get(3)?,
            last_modified: row.get(4)?,
        })
    }
}

impl FileDevice {
    /// Build a new `FileDevice`
    pub fn new(
        file: &models::File,
        device: &models::Device,
        is_unlocked: bool,
        is_encrypted: bool,
        last_modified: f64,
    ) -> Self {
        FileDevice {
            file_id: file.id.expect("missing file_id"),
            device_id: device.id.expect("missing device_id"),
            is_unlocked,
            is_encrypted,
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
                params![path],
                |row| FileDevice::try_from(row),
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
                params![file.id],
                |row| FileDevice::try_from(row),
            )?;
            items.push(device);
        }

        tx.commit()?;
        Ok(items)
    }
}

impl Insert for FileDevice {
    fn insert(&self, db: &Database) -> DatabaseResult<FileDevice> {
        let device = db.query_row(
            include_str!("sql/file_device/insert.sql"),
            params![
                self.file_id,
                self.device_id,
                self.is_unlocked,
                self.is_encrypted,
                self.last_modified
            ],
            |row| FileDevice::try_from(row),
        )?;

        Ok(device)
    }
}

impl InsertMany for FileDevice {}

impl Update for FileDevice {
    fn update(self, db: &Database) -> DatabaseResult<FileDevice> {
        let file_device = db.query_row(
            include_str!("sql/file_device/update.sql"),
            params![
                self.is_unlocked,
                self.is_encrypted,
                self.last_modified,
                self.file_id,
                self.device_id
            ],
            |row| FileDevice::try_from(row),
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
