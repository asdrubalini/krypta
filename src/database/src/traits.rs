use crate::{errors::DatabaseResult, Database};

/// A model that can be full-text searched
pub trait Search: Sized {
    fn search(database: &Database, query: impl AsRef<str>) -> DatabaseResult<Vec<Self>>;
}

/// A model that can be inserted
pub trait Insert<T>: Sized {
    fn insert(self, database: &Database) -> DatabaseResult<T>;
}

/// A model that can be mass-inserted
pub trait InsertMany<T>: Sized {
    fn insert_many(database: &Database, insertables: &[Self]) -> DatabaseResult<Vec<T>>;
}

/// A model that can be fetched
pub trait Fetch: Sized {
    fn fetch_all(database: &Database) -> DatabaseResult<Vec<Self>>;
}
