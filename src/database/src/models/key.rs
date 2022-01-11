use crypto::crypt::generate_random_secure_key_nonce_pair;
use rusqlite::{params, OptionalExtension};

use crate::{errors::DatabaseResult, Database};

#[derive(Clone, Debug)]
pub struct Key {
    pub key: [u8; 32],
}

impl Key {
    /// Get the current key, creating one if it doesn't exist yet
    pub fn get(db: &Database) -> DatabaseResult<Self> {
        let key = Self::try_find_key(db)?;

        match key {
            Some(key) => {
                let key = Key { key };
                Ok(key)
            }

            None => {
                Self::create(db, generate_random_secure_key_nonce_pair())?;
                let key = Key {
                    key: Self::try_find_key(db)?.unwrap(),
                };

                Ok(key)
            }
        }
    }

    fn try_find_key(db: &Database) -> DatabaseResult<Option<[u8; 32]>> {
        let maybe_key: Option<Vec<u8>> = db
            .query_row(include_str!("sql/key/find.sql"), [], |row| row.get(0))
            .optional()?;

        let maybe_key = maybe_key.map(|key| key.try_into().unwrap());
        Ok(maybe_key)
    }

    fn create(db: &Database, key: [u8; 32]) -> DatabaseResult<()> {
        let key = Vec::from(key);
        db.execute(include_str!("sql/key/create.sql"), params![key])?;
        Ok(())
    }
}
