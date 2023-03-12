use rusqlite::{Connection, Row};
use std::{any::type_name, time::Instant};

use crate::{errors::DatabaseResult, Database};

/// Get table name
pub trait TableName {
    fn table_name() -> &'static str;
}

/// Convert Row into Self
pub trait TryFromRow: Sized {
    fn try_from_row(row: &Row) -> Result<Self, rusqlite::Error>;
}

/// Get model from its primary key
pub trait Get: Sized {
    fn get(db: &Database, id: i64) -> DatabaseResult<Option<Self>>;
}

/// A model that can be full-text searched
pub trait Search: Sized {
    fn search(db: &Database, query: impl AsRef<str>) -> DatabaseResult<Vec<Self>>;
}

/// A model that can be inserted
pub trait Insert: Sized + TableName + TryFromRow {
    fn insert(self, db: &Database) -> DatabaseResult<Self>;
}

/// A model that can be mass-inserted
pub trait InsertMany: Sized + Insert {
    fn insert_many(
        db: &Connection,
        insertables: impl IntoIterator<Item = Self>,
    ) -> DatabaseResult<Vec<Self>> {
        let insertables: Vec<Self> = insertables.into_iter().collect();
        let mut inserted_items = vec![];

        log::trace!(
            "[{}] Start inserting {} items",
            type_name::<Self>(),
            insertables.len()
        );

        let start = Instant::now();

        for insertable in insertables {
            inserted_items.push(insertable.insert(db)?);
        }

        log::trace!(
            "[{}] Took {:?} for inserting {} items",
            type_name::<Self>(),
            start.elapsed(),
            inserted_items.len()
        );

        let callback_result = InsertMany::insert_many_hook(db, inserted_items)?;
        Ok(callback_result)
    }

    /// Custom hook for doing something else after inserting
    #[inline]
    fn insert_many_hook(
        _tx: &Connection,
        insertables: impl IntoIterator<Item = Self>,
    ) -> DatabaseResult<Vec<Self>> {
        // Do nothing by default
        Ok(insertables.into_iter().collect())
    }
}

/// A model that can be fetched
pub trait FetchAll: Sized + TryFromRow + TableName {
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
pub trait Update: Sized {
    fn update(self, db: &Database) -> DatabaseResult<Self>;
}

/// A model that can be mass-updated
pub trait UpdateMany: Sized + Update {
    fn update_many(
        db: &mut Database,
        updatables: impl IntoIterator<Item = Self>,
    ) -> DatabaseResult<Vec<Self>> {
        let updatables: Vec<Self> = updatables.into_iter().collect();
        let tx = db.transaction()?;
        let mut updated_items = vec![];

        log::trace!(
            "[{}] Start updading {} items",
            type_name::<Self>(),
            updatables.len()
        );

        let start = Instant::now();

        for updatable in updatables {
            updated_items.push(updatable.update(&tx)?);
        }

        tx.commit()?;

        log::trace!(
            "[{}] Took {:?} for updating {} items",
            type_name::<Self>(),
            start.elapsed(),
            updated_items.len()
        );

        let callback_result = UpdateMany::update_many_hook(db, updated_items)?;
        Ok(callback_result)
    }

    /// Custom hook for doing something else after updating
    #[inline]
    fn update_many_hook(
        _db: &mut Database,
        updatables: impl IntoIterator<Item = Self>,
    ) -> DatabaseResult<Vec<Self>> {
        // Do nothing by default
        Ok(updatables.into_iter().collect())
    }
}

/// A model that can be counted
pub trait Count: Sized + TableName {
    fn count(db: &Database) -> DatabaseResult<i64> {
        let table = Self::table_name();

        let count = db.query_row(&format!("SELECT COUNT(*) FROM `{table}`"), [], |row| {
            row.get(0)
        })?;
        Ok(count)
    }
}
