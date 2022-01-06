use async_trait::async_trait;

use crate::database::{errors::DatabaseError, Database};

/// A model that can be full-text searched
#[async_trait]
pub trait Search: Sized {
    async fn search(database: &Database, query: &str) -> Result<Vec<Self>, DatabaseError>;
}

/// A model that can be inserted
#[async_trait]
pub trait Insert: Sized {
    async fn insert(database: &Database, to_insert: Self) -> Result<(), DatabaseError>;
}

/// A model that can be mass-inserted
#[async_trait]
pub trait InsertMany: Sized {
    async fn insert_many(database: &Database, files: &[Self]) -> Result<(), DatabaseError>;
}

/// A model that can be fetched
#[async_trait]
pub trait Fetch: Sized {
    async fn fetch_all(database: &Database) -> Result<Vec<Self>, DatabaseError>;
}
