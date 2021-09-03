use async_trait::async_trait;

use crate::database::Database;

/// A model that can be full-text searched
#[async_trait]
pub trait Searchable<T> {
    async fn search(database: &Database, query: &str) -> Result<Vec<T>, sqlx::Error>;
}

/// A model that can be inserted and mass-inserted
#[async_trait]
pub trait Insertable<T> {
    async fn insert(database: &Database, to_insert: T) -> Result<(), sqlx::Error>;
    async fn insert_many(database: &Database, files: &[T]) -> Result<(), sqlx::Error>;
}

/// A model that can be fetched
#[async_trait]
pub trait Fetchable<T> {
    async fn fetch_all(database: &Database) -> Result<Vec<T>, sqlx::Error>;
}
