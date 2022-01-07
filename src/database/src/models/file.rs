use std::path::Path;
use std::{fs::Metadata, path::PathBuf};

use async_trait::async_trait;
use rand::Rng;
use sqlx::{
    sqlite::SqliteRow,
    types::chrono::{DateTime, Utc},
    FromRow, Row,
};

use crate::{errors::DatabaseError, BigIntAsBlob, Database};

use crate::traits::{Fetch, Insert, InsertMany, Search};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub id: i64,
    pub title: String,
    pub path: String,
    pub random_hash: String,
    pub contents_hash: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertFile {
    pub title: String,
    pub path: String,
    pub random_hash: String,
    pub contents_hash: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl<'r> FromRow<'r, SqliteRow> for File {
    fn from_row(row: &'r SqliteRow) -> Result<Self, sqlx::Error> {
        let id = row.try_get("id")?;
        let title = row.try_get("title")?;
        let path = row.try_get("path")?;
        let random_hash = row.try_get("random_hash")?;
        let contents_hash = row.try_get("contents_hash")?;
        let size: Vec<u8> = row.try_get("size")?;
        let created_at = row.try_get("created_at")?;
        let updated_at = row.try_get("updated_at")?;

        Ok(File {
            id,
            title,
            path,
            random_hash,
            contents_hash,
            size: BigIntAsBlob::from_bytes(&size),
            created_at,
            updated_at,
        })
    }
}

impl InsertFile {
    /// Build a new `InsertFile`
    pub fn new(title: String, path: PathBuf, contents_hash: String, size: u64) -> Self {
        let random_hash = File::pseudorandom_sha256_string();
        let now = chrono::Utc::now();

        InsertFile {
            title,
            path: path.to_string_lossy().to_string(),
            random_hash,
            contents_hash,
            size,
            created_at: now,
            updated_at: now,
        }
    }
}

#[async_trait]
impl Fetch for File {
    /// Fetch all records from database
    async fn fetch_all(database: &Database) -> Result<Vec<Self>, DatabaseError> {
        let files = sqlx::query_as::<_, File>(include_str!("./sql/file/fetch_all.sql"))
            .fetch_all(database)
            .await?;

        Ok(files)
    }
}

#[async_trait]
impl Search for File {
    /// Search files stored in database
    async fn search(database: &Database, query: &str) -> Result<Vec<Self>, DatabaseError> {
        let files = sqlx::query_as::<_, File>(include_str!("./sql/file/search.sql"))
            .bind(format!("%{}%", query))
            .fetch_all(database)
            .await?;

        Ok(files)
    }
}

#[async_trait]
impl Insert<File> for InsertFile {
    /// Insert a new file into the database
    async fn insert(self, database: &Database) -> Result<File, DatabaseError> {
        let file = sqlx::query_as::<_, File>(include_str!("./sql/file/insert.sql"))
            .bind(self.title)
            .bind(self.path)
            .bind(self.random_hash)
            .bind(self.contents_hash)
            .bind(BigIntAsBlob::from_u64(&self.size))
            .bind(self.created_at)
            .bind(self.updated_at)
            .fetch_one(database)
            .await?;

        Ok(file)
    }
}

#[async_trait]
impl InsertMany<File> for InsertFile {
    async fn insert_many(database: &Database, items: &[Self]) -> Result<Vec<File>, DatabaseError> {
        let mut transaction = database.begin().await?;
        let mut inserted_items = vec![];

        for file in items {
            let file = file.to_owned();
            log::trace!("InsertFile::InsertMany: {}", file.title);

            let inserted = sqlx::query_as::<_, File>(include_str!("./sql/file/insert.sql"))
                .bind(file.title)
                .bind(file.path)
                .bind(file.random_hash)
                .bind(file.contents_hash)
                .bind(BigIntAsBlob::from_u64(&file.size))
                .bind(file.created_at)
                .bind(file.updated_at)
                .fetch_one(&mut transaction)
                .await?;

            inserted_items.push(inserted);
        }

        transaction.commit().await?;

        Ok(inserted_items)
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
    pub size: u64,
}

impl MetadataFile {
    /// Convert &Metadata into a MetadataFile
    pub fn new(path: impl AsRef<Path>, metadata: &Metadata) -> Self {
        let path = path.as_ref();

        MetadataFile {
            title: path.to_string_lossy().to_string(),
            path: path.to_path_buf(),
            size: metadata.len(),
        }
    }

    /// Converts a `MetadataFile` into a `File` with some additional fields that are
    /// not present in a `Metadata` struct
    pub fn into_insert_file(self, contents_hash: String) -> InsertFile {
        InsertFile::new(self.title, self.path, contents_hash, self.size)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::traits::{Fetch, Insert, InsertMany};
    use crate::{create_in_memory, models::file::InsertFile};

    use super::File;

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

        let file1 = InsertFile::new(
            "foobar".to_string(),
            PathBuf::from("/path/to/foo7bar"),
            "asdas".to_string(),
            0,
        );

        assert!(file1.insert(&database).await.is_ok());

        let file2 = InsertFile::new(
            "foobar".to_string(),
            PathBuf::from("/path/to/foo7bar"),
            "bfsdfb".to_string(),
            0,
        );

        assert!(file2.insert(&database).await.is_err());
    }

    #[tokio::test]
    async fn test_insert_and_fetch() {
        let database = create_in_memory().await.unwrap();

        let insert_file = InsertFile::new(
            "foobar".to_string(),
            PathBuf::from("/path/to/foo/bar"),
            "sdadfb".to_string(),
            0,
        );

        let inserted_file = insert_file.insert(&database).await.unwrap();

        let files = File::fetch_all(&database).await;

        let files = files.unwrap();

        assert_eq!(files.len(), 1);

        let fetched_file = files.get(0).unwrap().to_owned();

        assert_eq!(inserted_file, fetched_file);
    }

    #[tokio::test]
    async fn test_insert_many() {
        let database = create_in_memory().await.unwrap();

        let insert_files = (0..128)
            .map(|i| {
                InsertFile::new(
                    format!("foobar_{}", i),
                    PathBuf::from(format!("/path/to/foo/bar/{}", i)),
                    format!("test_hash_placeholder_{}", i),
                    0,
                )
            })
            .collect::<Vec<InsertFile>>();

        let result = InsertFile::insert_many(&database, &insert_files).await;
        result.unwrap();
        let files = File::fetch_all(&database).await;
        let files = files.unwrap();

        assert_eq!(files.len(), 128);
    }

    #[tokio::test]
    async fn test_archive_size_and_count() {
        let database = create_in_memory().await.unwrap();

        let file = InsertFile::new(
            format!("foobar"),
            PathBuf::from("/path/to/foo/bar"),
            "test_hash_placeholder".to_string(),
            64,
        );

        file.insert(&database).await.unwrap();
        let archive_size = File::archive_size(&database).await;
        let archive_size = archive_size.unwrap();

        assert_eq!(archive_size, 64);
    }

    #[tokio::test]
    async fn test_enormous_archive_size() {
        let database = create_in_memory().await.unwrap();

        let insert_files = (0..128)
            .map(|i| {
                InsertFile::new(
                    format!("foobar_{}", i),
                    PathBuf::from(format!("/path/to/foo/bar/{}", i)),
                    format!("test_hash_placeholder_{}", i),
                    1_u64.pow(10), // 10 GB
                )
            })
            .collect::<Vec<InsertFile>>();

        InsertFile::insert_many(&database, &insert_files)
            .await
            .unwrap();
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
                InsertFile::new(
                    format!("foobar_{}", i),
                    PathBuf::from(format!("/path/to/foo/bar/{}", i)),
                    format!("test_hash_placeholder_{}", i),
                    0,
                )
            })
            .collect::<Vec<InsertFile>>();

        let result = InsertFile::insert_many(&database, &insert_files).await;
        result.unwrap();

        let archive_count = File::count(&database).await.unwrap();
        assert_eq!(archive_count, 8192);
    }
}
