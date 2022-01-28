use std::{env, path::Path};

use rusqlite::Connection;

use crate::errors::DatabaseResult;

pub type Database = Connection;

/// Connect to SQLite database
pub fn connect_or_create() -> DatabaseResult<Database> {
    let database_path = env::var("DATABASE_FILE").expect("Cannot read DATABASE_FILE env");

    let is_database_new = !Path::new(&database_path).exists();
    let connection = Connection::open(database_path)?;

    if is_database_new {
        load_schema(&connection)?;
    }

    Ok(connection)
}

/// Load database schema
fn load_schema(db: &Database) -> DatabaseResult<()> {
    log::trace!("New database... loading schema");

    db.execute_batch(include_str!("../schema.sql"))?;

    Ok(())
}

/// Create a temporary SQLite database in memory, used in tests
pub fn create_in_memory() -> DatabaseResult<Database> {
    let connection = Connection::open_in_memory()?;
    load_schema(&connection)?;

    Ok(connection)
}

#[cfg(test)]
pub mod tests {
    use std::env;

    use tmp::Tmp;

    use crate::{connect_or_create, utils::create_in_memory};

    #[test]
    fn test_create_sqlite_connection_in_memory() {
        create_in_memory().unwrap();
    }

    #[test]
    fn test_connect_and_create() {
        let tmp = Tmp::empty();

        let mut database_file = tmp.path();
        database_file.push("database.db");

        env::set_var("DATABASE_FILE", &database_file);

        connect_or_create().unwrap();
        assert!(database_file.is_file());
    }
}
