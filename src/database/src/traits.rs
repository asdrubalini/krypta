use crate::{errors::DatabaseResult, Database};

/// A model that can be full-text searched
pub trait Search: Sized {
    fn search(db: &Database, query: impl AsRef<str>) -> DatabaseResult<Vec<Self>>;
}

/// A model that can be inserted
pub trait Insert: Sized {
    fn insert(&self, db: &Database) -> DatabaseResult<Self>;
}

/// A model that can be mass-inserted
pub trait InsertMany: Sized {
    fn insert_many(db: &mut Database, insertables: Vec<Self>) -> DatabaseResult<Vec<Self>>;
}

/// A model that can be fetched
pub trait Fetch: Sized {
    fn fetch_all(db: &Database) -> DatabaseResult<Vec<Self>>;
}

/// A model that can be updated
pub trait Update: Sized {
    fn update(self, db: &Database) -> DatabaseResult<Self>;
}

/// A model that can be mass-updated
pub trait UpdateMany: Sized {
    fn update_many(db: &mut Database, updatables: Vec<Self>) -> DatabaseResult<Vec<Self>>;
}

/// A model that can be counted
pub trait Count: Sized {
    fn count(db: &Database) -> DatabaseResult<i64>;
}
