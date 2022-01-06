use std::{env, path::Path, str::FromStr};

use sqlx::{ConnectOptions, Executor, sqlite::SqliteConnectOptions, SqlitePool};

pub type Database = SqlitePool;

/// Connect to SQLite database
pub async fn connect_or_create() -> Result<Database, sqlx::Error> {
    let database_path = env::var("DATABASE_FILE").expect("Cannot read DATABASE_FILE env");

    let is_database_new = !Path::new(&database_path).exists();

    let mut options = SqliteConnectOptions::from_str(&format!("sqlite:{}", &database_path))?
        .auto_vacuum(sqlx::sqlite::SqliteAutoVacuum::Full)
        .create_if_missing(true);

    options.disable_statement_logging();

    let connection = SqlitePool::connect_with(options).await?;

    if is_database_new {
        load_schema(&connection).await;
    }

    Ok(connection)
}

/// Load database schema
async fn load_schema(database: &Database) {
    log::info!("New database... loading schema");

    database
        .execute(include_str!("../schema.sql"))
        .await
        .expect("Cannot load database schema");
}

#[cfg(test)]
pub mod tests {
    use std::{env, fs::remove_file, path::Path};
    use std::str::FromStr;
    use sqlx::sqlite::SqliteConnectOptions;
    use sqlx::SqlitePool;

    use super::{connect_or_create, load_schema, Database};

    /// Connect to SQLite database
    pub async fn create_in_memory() -> Result<Database, sqlx::Error> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")?;
        let connection = SqlitePool::connect_with(options).await?;

        load_schema(&connection).await;

        Ok(connection)
    }

    #[tokio::test]
    async fn test_create_sqlite_connection_in_memory() {
        let database = create_in_memory().await;
        assert!(database.is_ok());
    }

    #[tokio::test]
    async fn test_connect_and_create() {
        let tmp_file = "/tmp/database.db";
        env::set_var("DATABASE_FILE", tmp_file);

        remove_file(tmp_file).unwrap_or(());

        let database = connect_or_create().await;
        assert!(database.is_ok());
        assert!(Path::new(tmp_file).is_file());

        remove_file(tmp_file).unwrap_or(());
    }
}
