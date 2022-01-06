use std::{fs::Metadata, path::PathBuf};

use async_trait::async_trait;
use rand::Rng;
use sqlx::{
    sqlite::SqliteRow,
    types::chrono::{DateTime, Utc},
    FromRow, Row,
};

use super::{Fetch, Insert, Search};
use crate::database::{BigIntAsBlob, Database};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub title: String,
    pub path: String,
    pub is_remote: bool,
    pub is_encrypted: bool,
    pub random_hash: String,
    pub contents_hash: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl<'r> FromRow<'r, SqliteRow> for File {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let title = row.try_get("title")?;
        let path = row.try_get("path")?;
        let is_remote = row.try_get("is_remote")?;
        let is_encrypted = row.try_get("is_encrypted")?;
        let random_hash = row.try_get("random_hash")?;
        let contents_hash = row.try_get("contents_hash")?;
        let size: Vec<u8> = row.try_get("size")?;
        let created_at = row.try_get("created_at")?;
        let updated_at = row.try_get("updated_at")?;

        Ok(File {
            title,
            path,
            is_remote,
            is_encrypted,
            random_hash,
            contents_hash,
            size: BigIntAsBlob::from_bytes(&size),
            created_at,
            updated_at,
        })
    }
}

impl File {
    /// Build a new `File`
    pub fn new(
        title: String,
        path: PathBuf,
        is_remote: bool,
        is_encrypted: bool,
        contents_hash: String,
        size: u64,
    ) -> Self {
        let random_hash = File::pseudorandom_sha256_string();
        let now = chrono::Utc::now();

        File {
            title,
            path: path.to_string_lossy().to_string(),
            is_remote,
            is_encrypted,
            random_hash,
            contents_hash,
            size,
            created_at: now,
            updated_at: now,
        }
    }
}

#[async_trait]
impl Fetch<File> for File {
    /// Fetch all records from database
    async fn fetch_all(database: &Database) -> Result<Vec<File>, sqlx::Error> {
        let files = sqlx::query_as::<_, File>(include_str!("./sql/file/fetch_all.sql"))
            .fetch_all(database)
            .await?;

        Ok(files)
    }
}

#[async_trait]
impl Search<File> for File {
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
impl Insert<File> for File {
    /// Insert a new file into the database
    async fn insert(database: &Database, file: File) -> Result<(), sqlx::Error> {
        sqlx::query(include_str!("./sql/file/insert.sql"))
            .bind(file.title)
            .bind(file.path)
            .bind(file.is_remote)
            .bind(file.is_encrypted)
            .bind(file.random_hash)
            .bind(file.contents_hash)
            .bind(BigIntAsBlob::from_u64(&file.size))
            .bind(file.created_at)
            .bind(file.updated_at)
            .execute(database)
            .await?;

        Ok(())
    }

    async fn insert_many(database: &Database, files: &[File]) -> Result<(), sqlx::Error> {
        let mut transaction = database.begin().await?;

        for file in files {
            let file = file.clone();
            log::trace!("{}", file.title);

            sqlx::query(include_str!("./sql/file/insert.sql"))
                .bind(file.title)
                .bind(file.path)
                .bind(file.is_remote)
                .bind(file.is_encrypted)
                .bind(file.random_hash)
                .bind(file.contents_hash)
                .bind(BigIntAsBlob::from_u64(&file.size))
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
                let random_byte: u8 = generator.gen_range(0..=255);
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

    pub async fn archive_size(database: &Database) -> Result<u64, sqlx::Error> {
        let files = sqlx::query_as::<_, (Vec<u8>,)>(include_str!("./sql/file/size.sql"))
            .fetch_all(database)
            .await?;

        let size = files
            .iter()
            .map(|size| BigIntAsBlob::from_bytes(&size.0))
            .sum();

        Ok(size)
    }

    pub async fn count(database: &Database) -> Result<u32, sqlx::Error> {
        let files = sqlx::query_as::<_, (u32,)>(include_str!("./sql/file/count.sql"))
            .fetch_one(database)
            .await?;

        Ok(files.0)
    }
}

pub struct MetadataFile {
    pub title: String,
    pub path: PathBuf,
    pub is_remote: bool,
    pub is_encrypted: bool,
    pub size: u64,
}

impl MetadataFile {
    /// Convert &Metadata into a MetadataFile
    pub fn new(path: &PathBuf, metadata: &Metadata) -> Self {
        MetadataFile {
            title: path.to_string_lossy().to_string(),
            path: path.clone(),
            is_remote: false,
            is_encrypted: false,
            size: metadata.len(),
        }
    }

    /// Converts a `MetadataFile` into a `File` with some additional fields that are
    /// not present in a `Metadata` struct
    pub fn into_file(self, contents_hash: String) -> File {
        File::new(
            self.title,
            self.path,
            self.is_remote,
            self.is_encrypted,
            contents_hash,
            self.size,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::File;
    use crate::database::{
        create_in_memory,
        models::{Fetch, Insert},
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

        let file1 = File::new(
            "foobar".to_string(),
            PathBuf::from("/path/to/foo7bar"),
            false,
            false,
            "asdas".to_string(),
            0,
        );

        assert!(File::insert(&database, file1).await.is_ok());

        let file2 = File::new(
            "foobar".to_string(),
            PathBuf::from("/path/to/foo7bar"),
            false,
            false,
            "bfsdfb".to_string(),
            0,
        );

        assert!(File::insert(&database, file2).await.is_err());
    }

    #[tokio::test]
    async fn test_insert_and_fetch() {
        let database = create_in_memory().await.unwrap();

        let insert_file = File::new(
            "foobar".to_string(),
            PathBuf::from("/path/to/foo/bar"),
            false,
            false,
            "sdadfb".to_string(),
            0,
        );

        File::insert(&database, insert_file.clone()).await.unwrap();

        let files = File::fetch_all(&database).await;

        let files = files.unwrap();

        assert_eq!(files.len(), 1);

        let fetched_file = files.get(0).unwrap().to_owned();

        assert_eq!(insert_file, fetched_file);
    }

    #[tokio::test]
    async fn test_insert_many() {
        let database = create_in_memory().await.unwrap();

        let insert_files = (0..128)
            .map(|i| {
                File::new(
                    format!("foobar_{}", i),
                    PathBuf::from(format!("/path/to/foo/bar/{}", i)),
                    false,
                    false,
                    format!("test_hash_placeholder_{}", i),
                    0,
                )
            })
            .collect::<Vec<File>>();

        let result = File::insert_many(&database, &insert_files).await;
        result.unwrap();
        let files = File::fetch_all(&database).await;
        let files = files.unwrap();

        assert_eq!(files.len(), 128);
    }

    #[tokio::test]
    async fn test_archive_size_and_count() {
        let database = create_in_memory().await.unwrap();

        let file = File::new(
            format!("foobar"),
            PathBuf::from("/path/to/foo/bar"),
            false,
            false,
            "test_hash_placeholder".to_string(),
            64,
        );

        let result = File::insert(&database, file).await;
        result.unwrap();
        let archive_size = File::archive_size(&database).await;
        let archive_size = archive_size.unwrap();

        assert_eq!(archive_size, 64);
    }

    #[tokio::test]
    async fn test_enormous_archive_size() {
        let database = create_in_memory().await.unwrap();

        let insert_files = (0..128)
            .map(|i| {
                File::new(
                    format!("foobar_{}", i),
                    PathBuf::from(format!("/path/to/foo/bar/{}", i)),
                    false,
                    false,
                    format!("test_hash_placeholder_{}", i),
                    1_u64.pow(10), // 10 GB
                )
            })
            .collect::<Vec<File>>();

        let result = File::insert_many(&database, &insert_files).await;
        result.unwrap();
        let archive_size = File::archive_size(&database).await;
        let archive_size = archive_size.unwrap();

        assert_eq!(archive_size, 128 * 1_u64.pow(10));

        let archive_count = File::count(&database).await.unwrap();
        assert_eq!(archive_count, 128);
    }

    #[tokio::test]
    async fn test_enormous_file_insert_and_count() {
        let database = create_in_memory().await.unwrap();

        let insert_files = (0..8192)
            .map(|i| {
                File::new(
                    format!("foobar_{}", i),
                    PathBuf::from(format!("/path/to/foo/bar/{}", i)),
                    false,
                    false,
                    format!("test_hash_placeholder_{}", i),
                    0,
                )
            })
            .collect::<Vec<File>>();

        let result = File::insert_many(&database, &insert_files).await;
        result.unwrap();

        let archive_count = File::count(&database).await.unwrap();
        assert_eq!(archive_count, 8192);
    }
}
