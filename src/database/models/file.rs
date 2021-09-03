use super::{Fetchable, Insertable, Searchable};
use crate::database::Database;

use async_trait::async_trait;
use rand::Rng;
use sqlx::types::chrono::{DateTime, Utc};
use std::path::PathBuf;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct File {
    pub id: i64,
    pub title: String,
    pub path: String,
    pub is_remote: bool,
    pub is_encrypted: bool,
    pub random_hash: String,
    pub size: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<&InsertableFile> for File {
    fn from(file: &InsertableFile) -> Self {
        let random_hash = File::pseudorandom_sha256_string();
        let now = chrono::Utc::now();

        File {
            id: 0,
            title: file.title.clone(),
            path: file.path.to_string_lossy().to_string(),
            is_remote: file.is_remote,
            is_encrypted: file.is_encrypted,
            random_hash,
            size: file.size,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct InsertableFile {
    pub title: String,
    pub path: PathBuf,
    pub is_remote: bool,
    pub is_encrypted: bool,
    pub size: i64,
}

impl From<&File> for InsertableFile {
    fn from(file: &File) -> Self {
        InsertableFile {
            title: file.title.clone(),
            path: PathBuf::from(file.path.clone()),
            is_remote: file.is_remote,
            is_encrypted: file.is_encrypted,
            size: file.size,
        }
    }
}

#[async_trait]
impl Fetchable<File> for File {
    /// Fetch all records from database
    async fn fetch_all(database: &Database) -> Result<Vec<File>, sqlx::Error> {
        let files = sqlx::query_as::<_, File>(include_str!("./sql/file/fetch_all.sql"))
            .fetch_all(database)
            .await?;

        Ok(files)
    }
}

#[async_trait]
impl Searchable<File> for File {
    /// Search files stored in database
    async fn search(database: &Database, query: &str) -> Result<Vec<File>, sqlx::Error> {
        let files = sqlx::query_as::<_, File>(include_str!("./sql/file/search.sql"))
            .bind(format!("%{}%", query))
            .fetch_all(database)
            .await?;

        Ok(files)
    }
}

#[async_trait]
impl Insertable<InsertableFile> for File {
    /// Insert a new file into the database
    async fn insert(database: &Database, file: InsertableFile) -> Result<(), sqlx::Error> {
        let file = File::from(&file);

        sqlx::query(include_str!("./sql/file/insert.sql"))
            .bind(file.title)
            .bind(file.path)
            .bind(file.is_remote)
            .bind(file.is_encrypted)
            .bind(file.random_hash)
            .bind(file.size)
            .bind(file.created_at)
            .bind(file.updated_at)
            .execute(database)
            .await?;

        Ok(())
    }

    async fn insert_many(database: &Database, files: &[InsertableFile]) -> Result<(), sqlx::Error> {
        let mut transaction = database.begin().await?;

        for file in files {
            let file = File::from(file);

            log::trace!("{}", file.title);

            sqlx::query(include_str!("./sql/file/insert.sql"))
                .bind(file.title)
                .bind(file.path)
                .bind(file.is_remote)
                .bind(file.is_encrypted)
                .bind(file.random_hash)
                .bind(file.size)
                .bind(file.created_at)
                .bind(file.updated_at)
                .execute(&mut transaction)
                .await?;
        }

        transaction.commit().await?;

        Ok(())
    }
}

#[allow(dead_code)]
impl File {
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

    pub async fn get_file_paths(database: &Database) -> Result<Vec<PathBuf>, sqlx::Error> {
        let files = sqlx::query_as::<_, (String,)>(include_str!("./sql/file/find_paths.sql"))
            .fetch_all(database)
            .await?;

        let paths = files.iter().map(|path| path.0.to_owned());

        Ok(paths.map(PathBuf::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::File;
    use crate::database::{
        create_in_memory,
        models::{file::InsertableFile, Fetchable, Insertable},
    };

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
            title: "foobar".to_string(),
            path: PathBuf::from("/path/to/foo/bar"),
            is_remote: false,
            is_encrypted: false,
            size: 0,
        };

        assert!(File::insert(&database, file1).await.is_ok());

        let file2 = InsertableFile {
            title: "foobar".to_string(),
            path: PathBuf::from("/path/to/foo/bar"),
            is_remote: false,
            is_encrypted: false,
            size: 0,
        };

        assert!(File::insert(&database, file2).await.is_err());
    }

    #[tokio::test]
    async fn test_insert_and_fetch() {
        let database = create_in_memory().await.unwrap();

        let insert_file = InsertableFile {
            title: "foobar".to_string(),
            path: PathBuf::from("/path/to/foo/bar"),
            is_remote: false,
            is_encrypted: false,
            size: 0,
        };

        assert!(File::insert(&database, insert_file.clone()).await.is_ok());

        let files = File::fetch_all(&database).await;

        assert!(files.is_ok());

        let files = files.unwrap();

        assert_eq!(files.len(), 1);

        let fetched_file = InsertableFile::from(files.get(0).unwrap());

        assert_eq!(insert_file, fetched_file);
    }

    #[tokio::test]
    async fn test_insert_many() {
        let database = create_in_memory().await.unwrap();

        let insert_files = (0..128)
            .map(|i| InsertableFile {
                title: format!("foobar_{}", i),
                path: PathBuf::from(format!("/path/to/foo/bar/{}", i)),
                is_remote: false,
                is_encrypted: false,
                size: 0,
            })
            .collect::<Vec<InsertableFile>>();

        let result = File::insert_many(&database, &insert_files).await;
        assert!(result.is_ok());
        let files = File::fetch_all(&database).await;
        assert!(files.is_ok());

        let files = files.unwrap();

        assert_eq!(files.len(), 128);
    }
}
