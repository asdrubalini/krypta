use database_macros::{Insert, TableName, TryFromRow};

#[derive(TableName, TryFromRow, Insert)]
pub struct Tag {
    pub id: i32,
    pub name: String,
}
