use crate::database::Database;
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Debug, sqlx::FromRow)]
pub struct File {
    pub id: i64,
    pub title: String,
    pub path: String,
    pub is_remote: bool,
    pub is_encrypted: bool,
    pub random_hash: String,
    pub data_hash: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl File {
    /// Search files stored in database
    pub async fn search(database: &Database, query: &str) -> Result<Vec<File>, sqlx::Error> {
        let files = sqlx::query_as::<_, File>(include_str!("./sql/query_file.sql"))
            .bind(format!("%{}%", query))
            .fetch_all(database)
            .await?;

        Ok(files)
    }

    /// Insert a new file into the database
    pub async fn insert(
        database: &Database,
        title: &str,
        path: &str,
        is_remote: &bool,
        is_encrypted: &bool,
        random_hash: &str,
        data_hash: &str,
    ) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now();

        sqlx::query(include_str!("./sql/insert_file.sql"))
            .bind(title)
            .bind(path)
            .bind(is_remote)
            .bind(is_encrypted)
            .bind(random_hash)
            .bind(data_hash)
            .bind(&now)
            .bind(&now)
            .execute(database)
            .await?;

        Ok(())
    }
}
