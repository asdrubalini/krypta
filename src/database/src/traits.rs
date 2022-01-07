use crate::{errors::DatabaseError, Database};

/// A model that can be full-text searched
pub trait Search: Sized {
    async fn search(database: &Database, query: &str) -> Result<Vec<Self>, DatabaseError>;
}

/// A model that can be inserted
pub trait Insert<T>: Sized {
    async fn insert(self, database: &Database) -> Result<T, DatabaseError>;
}

/// A model that can be mass-inserted
pub trait InsertMany<T>: Sized {
    async fn insert_many(
        database: &Database,
        insertables: &[Self],
    ) -> Result<Vec<T>, DatabaseError>;
}

/// A model that can be fetched
pub trait Fetch: Sized {
    async fn fetch_all(database: &Database) -> Result<Vec<Self>, DatabaseError>;
}
