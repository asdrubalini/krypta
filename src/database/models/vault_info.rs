use sqlx::{sqlite::SqliteRow, FromRow, Row};

use crate::database::BigIntAsBlob;

pub struct VaultInfo {
    name: String,
    total_size: u64,
}

impl<'r> FromRow<'r, SqliteRow> for VaultInfo {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let name = row.try_get("name")?;
        let total_size: Vec<u8> = row.try_get("total_size")?;

        Ok(VaultInfo {
            name,
            total_size: BigIntAsBlob::from_bytes(&total_size),
        })
    }
}
