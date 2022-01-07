use rusqlite::{params, Row};

use crate::{errors::DatabaseError, models, Database};

use crate::traits::{Insert, InsertMany};

#[derive(Clone, Debug)]
pub struct FileDevice {
    file_id: i64,
    device_id: i64,
    is_unlocked: bool,
    is_encrypted: bool,
}

impl TryFrom<&Row<'_>> for FileDevice {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(FileDevice {
            file_id: row.get(0)?,
            device_id: row.get(1)?,
            is_unlocked: row.get(2)?,
            is_encrypted: row.get(3)?,
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
    ) -> Self {
        FileDevice {
            file_id: file.id,
            device_id: device.id,
            is_unlocked,
            is_encrypted,
        }
    }
}

impl Insert<FileDevice> for FileDevice {
    fn insert(&self, db: &Database) -> Result<FileDevice, DatabaseError> {
        let device = db.query_row(
            include_str!("sql/file_device/insert.sql"),
            params![
                self.file_id,
                self.device_id,
                self.is_unlocked,
                self.is_encrypted
            ],
            |row| Ok(FileDevice::try_from(row)?),
        )?;

        Ok(device)
    }
}

impl InsertMany<FileDevice> for FileDevice {
    fn insert_many(db: &mut Database, items: &[Self]) -> Result<Vec<FileDevice>, DatabaseError> {
        let tx = db.transaction()?;
        let mut inserted_items = vec![];

        for file in items {
            inserted_items.push(file.insert(&tx)?);
        }

        tx.commit()?;

        Ok(inserted_items)
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

        let to_insert = FileDevice::new(&file, &device, false, false);
        to_insert.insert(&database).unwrap();
    }
}
