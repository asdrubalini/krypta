use std::path::PathBuf;

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

pub struct InsertableFile<'a> {
    pub title: &'a str,
    pub path: &'a PathBuf,
    pub is_remote: bool,
    pub is_encrypted: bool,
}

#[allow(dead_code)]
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
    pub async fn insert(database: &Database, file: InsertableFile<'_>) -> Result<(), sqlx::Error> {
        let now = chrono::Utc::now();
        let random_hash = File::pseudorandom_sha256_string();

        sqlx::query(include_str!("./sql/insert_file.sql"))
            .bind(file.title)
            .bind(file.path.to_str())
            .bind(file.is_remote)
            .bind(file.is_encrypted)
            .bind(random_hash)
            .bind(&now)
            .bind(&now)
            .execute(database)
            .await?;

        Ok(())
    }

    pub async fn insert_many(
        database: &Database,
        files: Vec<InsertableFile<'_>>,
    ) -> Result<usize, sqlx::Error> {
        todo!()
    }

    pub async fn get_file_paths(database: &Database) -> Result<Vec<PathBuf>, sqlx::Error> {
        let files = sqlx::query_as::<_, (String,)>(include_str!("./sql/find_file_paths.sql"))
            .fetch_all(database)
            .await?;

        let paths = files.iter().map(|path| path.0.to_owned());

        Ok(paths
            .map(|path_string| PathBuf::from(path_string))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::File;
    use crate::database::{create_in_memory, models::file::InsertableFile};

    #[test]
    fn test_pseudorandom_sha256_string_is_valid_length_and_contains_valid_chars() {
        let valid_chars = "0123456789abcdfe";

        for _ in 0..10_000 {
            let result = File::pseudorandom_sha256_string();
            assert_eq!(result.len(), 64);

            for chr in result.chars() {
                assert!(valid_chars.contains(chr));
            }
        }
    }

    #[tokio::test]
    async fn test_insert_unique() {
        let database = create_in_memory().await.unwrap();

        let file1 = InsertableFile {
            title: "foobar",
            path: &PathBuf::from("/path/to/foo/bar"),
            is_remote: false,
            is_encrypted: false,
        };

        assert!(File::insert(&database, file1).await.is_ok());

        let file2 = InsertableFile {
            title: "foobar",
            path: &PathBuf::from("/path/to/foo/bar"),
            is_remote: false,
            is_encrypted: false,
        };

        assert!(File::insert(&database, file2).await.is_err());
    }
}
