use async_trait::async_trait;

use crate::database::Database;

#[async_trait]
pub trait Searchable<T> {
    async fn search(database: &Database, query: &str) -> Result<Vec<T>, sqlx::Error>;
}

#[async_trait]
pub trait Insertable<T> {
    async fn insert(database: &Database, to_insert: T) -> Result<(), sqlx::Error>;
    async fn insert_many(database: &Database, files: &[T]) -> Result<(), sqlx::Error>;
}
