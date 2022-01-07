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
        todo!()
    }
}

/*
#[async_trait]
impl Insert for FileDevice {
    async fn insert(database: &Database, file_device: Self) -> Result<(), DatabaseError> {
        sqlx::query(include_str!("./sql/file_device/insert.sql"))
            .bind(file_device.file_id)
            .bind(file_device.device_id)
            .bind(file_device.is_unlocked)
            .bind(file_device.is_encrypted)
            .execute(database)
            .await?;

        Ok(())
    }
}
*/
