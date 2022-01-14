use std::any::type_name;
use std::fs::Metadata;
use std::path::Path;
use std::time::{Instant, UNIX_EPOCH};

use rusqlite::{params, Row};

use crate::errors::DatabaseResult;
use crate::{models, Database};

use crate::traits::{Insert, InsertMany, Update, UpdateMany};

/// Convert a std::fs::Metadata into a UNIX epoch u64
#[cfg(target_os = "linux")]
pub fn metadata_to_last_modified(metadata: &Metadata) -> i64 {
    metadata
        .modified()
        .unwrap()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[derive(Clone, Debug)]
pub struct FileDevice {
    file_id: i64,
    device_id: i64,
    pub is_unlocked: bool,
    pub is_encrypted: bool,
    pub last_modified: i64,
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
        last_modified: i64,
    ) -> Self {
        FileDevice {
            file_id: file.id,
            device_id: device.id,
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
}

impl Insert<FileDevice> for FileDevice {
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

impl InsertMany<FileDevice> for FileDevice {
    fn insert_many(db: &mut Database, items: &[Self]) -> DatabaseResult<Vec<FileDevice>> {
        let tx = db.transaction()?;
        let mut inserted_items = vec![];

        log::trace!(
            "[{}] Start inserting {} FileDevice",
            type_name::<Self>(),
            items.len()
        );

        let start = Instant::now();

        for file in items {
            inserted_items.push(file.insert(&tx)?);
        }

        tx.commit()?;

        log::trace!(
            "[{}] Took {:?} for updating {} items",
            type_name::<Self>(),
            start.elapsed(),
            items.len()
        );

        Ok(inserted_items)
    }
}

impl Update for FileDevice {
    fn update(&self, db: &Database) -> DatabaseResult<Self> {
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

impl UpdateMany for FileDevice {
    fn update_many(db: &mut Database, updatables: &[Self]) -> DatabaseResult<Vec<Self>> {
        let tx = db.transaction()?;
        let mut results = vec![];

        log::trace!(
            "[{}] Start updading {} FileDevice",
            type_name::<Self>(),
            updatables.len()
        );

        let start = Instant::now();

        for updatable in updatables {
            results.push(updatable.update(&tx)?);
        }

        tx.commit()?;

        log::trace!(
            "[{}] Took {:?} for updating {} items",
            type_name::<Self>(),
            start.elapsed(),
            updatables.len()
        );

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::FileDevice;
    use crate::create_in_memory;
    use crate::models::{Device, InsertFile};
    use crate::traits::Insert;

    #[test]
    fn test_insert() {
        let database = create_in_memory().unwrap();

        // Prepare file
        let file = InsertFile::new(
            "random title".to_string(),
            Default::default(),
            "random unique hash".to_string(),
            0,
        )
        .insert(&database)
        .unwrap();

        // Prepare device
        let device = Device::find_or_create_current(&database).unwrap();

        let to_insert = FileDevice::new(&file, &device, false, false, 0);
        to_insert.insert(&database).unwrap();
    }
}
