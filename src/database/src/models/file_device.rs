use async_trait::async_trait;

use crate::{errors::DatabaseError, models, Database};

use super::traits::Insert;

#[derive(sqlx::FromRow)]
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
                .bind(self.is_unlocked)
                .bind(self.is_encrypted)
                .fetch_one(database)
                .await?;

        Ok(file_device)
    }
}
