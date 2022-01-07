use async_trait::async_trait;

use crate::{errors::DatabaseError, models, Database};

use crate::traits::{Insert, InsertMany};

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct FileDevice {
    file_id: i64,
    device_id: i64,
    is_unlocked: bool,
    is_encrypted: bool,
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

#[async_trait]
impl Insert<FileDevice> for FileDevice {
    async fn insert(self, database: &Database) -> Result<FileDevice, DatabaseError> {
        let file_device =
            sqlx::query_as::<_, FileDevice>(include_str!("./sql/file_device/insert.sql"))
                .bind(self.file_id)
                .bind(self.device_id)
                .bind(self.is_unlocked as i64)
                .bind(self.is_encrypted as i64)
                .fetch_one(database)
                .await?;

        Ok(file_device)
    }
}

#[async_trait]
impl InsertMany<FileDevice> for FileDevice {
    async fn insert_many(
        database: &Database,
        items: &[Self],
    ) -> Result<Vec<FileDevice>, DatabaseError> {
        let mut transaction = database.begin().await?;
        let mut inserted_items = vec![];

        for file_device in items {
            let file_device = file_device.to_owned();
            log::trace!(
                "FileDevice::InsertMany: {} {}",
                file_device.file_id,
                file_device.device_id
            );

            let inserted =
                sqlx::query_as::<_, FileDevice>(include_str!("./sql/file_device/insert.sql"))
                    .bind(file_device.file_id)
                    .bind(file_device.device_id)
                    .bind(file_device.is_unlocked)
                    .bind(file_device.is_encrypted)
                    .fetch_one(&mut transaction)
                    .await?;

            inserted_items.push(inserted);
        }

        transaction.commit().await?;

        Ok(inserted_items)
    }
}

#[cfg(test)]
mod tests {
    use super::FileDevice;
    use crate::create_in_memory;
    use crate::models::{Device, InsertFile};
    use crate::traits::Insert;

    #[tokio::test]
    async fn test_insert() {
        let database = create_in_memory().await.unwrap();

        // Prepare file
        let file = InsertFile::new(
            "random title".to_string(),
            Default::default(),
            "random unique hash".to_string(),
            0,
        )
        .insert(&database)
        .await
        .unwrap();

        // Prepare device
        let device = Device::find_or_create_current(&database).await.unwrap();

        let to_insert = FileDevice::new(&file, &device, false, false);
        to_insert.insert(&database).await.unwrap();
    }
}
