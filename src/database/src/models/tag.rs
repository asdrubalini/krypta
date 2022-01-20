use database_macros::{TableName, TryFromRow};

#[derive(TableName, TryFromRow)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}
