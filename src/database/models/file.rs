use crate::database::Database;
use rand::Rng;
use sqlx::types::chrono::{DateTime, Utc};

#[derive(Debug, sqlx::FromRow)]
pub struct File {
    pub id: i64,
    pub title: String,
    pub path: String,
    pub is_remote: bool,
    pub is_encrypted: bool,
    pub random_hash: String,
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

    /// Generate a pseudorandom sha256 hash
    fn pseudorandom_sha256_string() -> String {
        let mut generator = rand::thread_rng();

        (0..32)
            .into_iter()
            .map(|_| {
                let random_byte: u8 = generator.gen_range(0..255);
                format!("{:02x}", random_byte)
            })
            .collect()
    }

    /// Insert a new file into the database
    pub async fn insert(
        database: &Database,
        title: &str,
        path: &str,
        is_remote: &bool,
        is_encrypted: &bool,
    ) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now();
        let random_hash = File::pseudorandom_sha256_string();

        sqlx::query(include_str!("./sql/insert_file.sql"))
            .bind(title)
            .bind(path)
            .bind(is_remote)
            .bind(is_encrypted)
            .bind(random_hash)
            .bind(&now)
            .bind(&now)
            .execute(database)
            .await?;

        Ok(())
    }

    pub async fn get_file_paths(database: &Database) -> Result<Vec<String>, sqlx::Error> {
        let files = sqlx::query_as::<_, (String,)>(include_str!("./sql/find_file_paths.sql"))
            .fetch_all(database)
            .await?;

        Ok(files.iter().map(|path| path.0.to_owned()).collect())
    }
}
