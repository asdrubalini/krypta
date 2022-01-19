use std::{any::type_name, time::Instant};

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
pub trait InsertMany: Sized + Insert {
    fn insert_many(db: &mut Database, insertables: Vec<Self>) -> DatabaseResult<Vec<Self>> {
        let tx = db.transaction()?;
        let mut inserted_items = vec![];

        log::trace!(
            "[{}] Start inserting {} items",
            type_name::<Self>(),
            insertables.len()
        );

        let start = Instant::now();

        for insertable in insertables {
            inserted_items.push(insertable.insert(&tx)?);
        }

        tx.commit()?;

        log::trace!(
            "[{}] Took {:?} for inserting {} items",
            type_name::<Self>(),
            start.elapsed(),
            inserted_items.len()
        );

        Ok(inserted_items)
    }
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
pub trait UpdateMany: Sized + Update {
    fn update_many(db: &mut Database, updatables: Vec<Self>) -> DatabaseResult<Vec<Self>> {
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

        Ok(updated_items)
    }
}

/// A model that can be counted
pub trait Count: Sized {
    fn count(db: &Database) -> DatabaseResult<i64>;
}
