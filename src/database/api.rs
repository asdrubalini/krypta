use std::{env, path::Path, str::FromStr};

use sqlx::{sqlite::SqliteConnectOptions, Executor, SqlitePool};

pub type Database = SqlitePool;

/// Connect to SQLite database
pub async fn connect_or_create() -> Result<Database, sqlx::Error> {
    let database_path = env::var("DATABASE_FILE").expect("Cannot read DATABASE_FILE env");

    let is_database_new = !Path::new(&database_path).exists();

    let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", &database_path))?
        .create_if_missing(true);

    let connection = SqlitePool::connect_with(options).await?;

    if is_database_new {
        load_schema(&connection).await;
    }

    Ok(connection)
}

/// Load database schema
pub async fn load_schema(database: &Database) {
    log::info!("New database... loading schema");

    database
        .execute(include_str!("../schema.sql"))
        .await
        .expect("Cannot load database schema");
}
