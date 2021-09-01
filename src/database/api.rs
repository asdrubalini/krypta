use std::str::FromStr;

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

pub type Database = SqlitePool;

pub async fn connect_or_create() -> Result<Database, sqlx::Error> {
    let options = SqliteConnectOptions::from_str("sqlite:files.db")?
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
        .create_if_missing(true);

    let connection = SqlitePool::connect_with(options).await?;

    Ok(connection)
}
