use crate::{errors::DatabaseResult, Database};

/// A model that can be full-text searched
pub trait Search: Sized {
    fn search(db: &Database, query: impl AsRef<str>) -> DatabaseResult<Vec<Self>>;
}

/// A model that can be inserted
pub trait Insert<T>: Sized {
    fn insert(&self, db: &Database) -> DatabaseResult<T>;
}

/// A model that can be mass-inserted
pub trait InsertMany<T>: Sized {
    fn insert_many(db: &mut Database, insertables: &[Self]) -> DatabaseResult<Vec<T>>;
}

/// A model that can be fetched
pub trait Fetch: Sized {
    fn fetch_all(db: &Database) -> DatabaseResult<Vec<Self>>;
}

pub trait UpdateMany: Sized {
    fn update_many(db: &mut Database, updatables: &[Self]) -> DatabaseResult<Vec<Self>>;
}

pub trait Update: Sized {
    fn update(&self, db: &Database) -> DatabaseResult<Self>;
}
