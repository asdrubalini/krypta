use rusqlite::Row;

use crate::{errors::DatabaseResult, Database};

/// Convert Row into Self
pub trait TryFromRow: Sized {
    fn try_from_row(row: &Row) -> Result<Self, rusqlite::Error>;
}

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
pub trait Fetch: Sized + TryFromRow {
    fn table_name() -> &'static str;

    fn fetch_all(db: &Database) -> DatabaseResult<Vec<Self>> {
        let table = Self::table_name();

        let mut stmt = db.prepare(&format!("SELECT * FROM `{table}`;"))?;
        let mut rows = stmt.query([])?;

        let mut files = vec![];
        while let Some(row) = rows.next()? {
            files.push(Self::try_from_row(row)?);
        }

        Ok(files)
    }
}

/// A model that can be updated
pub trait Update<T>: Sized {
    fn update(&self, db: &Database) -> DatabaseResult<T>;
}

/// A model that can be mass-updated
pub trait UpdateMany<T>: Sized {
    fn update_many(db: &mut Database, updatables: &[Self]) -> DatabaseResult<Vec<T>>;
}

/// A model that can be counted
pub trait Count: Sized {
    fn table_name() -> &'static str;

    fn count(db: &Database) -> DatabaseResult<i64> {
        let table = Self::table_name();

        let count = db.query_row(&format!("SELECT COUNT(*) FROM `{table}`"), [], |row| {
            row.get(0)
        })?;
        Ok(count)
    }
}
