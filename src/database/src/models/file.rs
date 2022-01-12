use std::path::Path;
use std::{fs::Metadata, path::PathBuf};

use chrono::{DateTime, Utc};
use crypto::crypt::generate_random_secure_key_nonce_pair;
use rand::Rng;
use rusqlite::{params, Row};

use crate::{errors::DatabaseResult, Database};

use crate::traits::{Fetch, Insert, InsertMany, Search};

use super::Device;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub id: i64,
    pub title: String,
    pub path: PathBuf,
    pub random_hash: String,
    pub contents_hash: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub key: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl TryFrom<&Row<'_>> for File {
    type Error = rusqlite::Error;

    fn try_from(row: &Row<'_>) -> Result<Self, Self::Error> {
        Ok(File {
            id: row.get(0)?,
            title: row.get(1)?,
            path: PathBuf::from(row.get::<_, String>(2)?),
            random_hash: row.get(3)?,
            contents_hash: row.get(4)?,
            size: row.get(5)?,
            created_at: row.get(6)?,
            updated_at: row.get(7)?,
            key: row.get(8)?,
            nonce: row.get(9)?,
        })
    }
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
    pub key: Vec<u8>,
    pub nonce: Vec<u8>,
}

impl InsertFile {
    /// Build a new `InsertFile` and generate on the fly some stuff
    pub fn new(title: String, path: PathBuf, contents_hash: String, size: u64) -> Self {
        let random_hash = File::pseudorandom_hex_string();
        let now = chrono::Utc::now();

        // Key and nonce generation
        let (key, nonce) = generate_random_secure_key_nonce_pair();
        let key = Vec::from(key);
        let nonce = Vec::from(nonce);

        InsertFile {
            title,
            path: path.to_string_lossy().to_string(),
            random_hash,
            contents_hash,
            size,
            created_at: now,
            updated_at: now,
            key,
            nonce,
        }
    }
}

impl Fetch for File {
    /// Fetch all records from database
    fn fetch_all(db: &Database) -> DatabaseResult<Vec<Self>> {
        let mut stmt = db.prepare(include_str!("sql/file/fetch_all.sql"))?;
        let mut rows = stmt.query([])?;

        let mut files = vec![];
        while let Some(row) = rows.next()? {
            files.push(File::try_from(row)?);
        }

        Ok(files)
    }
}

impl Search for File {
    /// Search files stored in database
    fn search(db: &Database, query: impl AsRef<str>) -> DatabaseResult<Vec<Self>> {
        let mut stmt = db.prepare(include_str!("sql/file/search.sql"))?;
        let mut rows = stmt.query([format!("%{}%", query.as_ref())])?;

        let mut files = vec![];
        while let Some(row) = rows.next()? {
            files.push(File::try_from(row)?);
        }

        Ok(files)
    }
}

impl Insert<File> for InsertFile {
    /// Insert a new file into the database
    fn insert(&self, db: &Database) -> DatabaseResult<File> {
        let device = db.query_row(
            include_str!("sql/file/insert.sql"),
            params![
                self.title,
                self.path,
                self.random_hash,
                self.contents_hash,
                self.size,
                self.created_at,
                self.updated_at,
                self.key,
                self.nonce
            ],
            |row| File::try_from(row),
        )?;

        Ok(device)
    }
}

impl InsertMany<File> for InsertFile {
    fn insert_many(db: &mut Database, items: &[Self]) -> DatabaseResult<Vec<File>> {
        let tx = db.transaction()?;
        let mut files = vec![];

        for file in items {
            files.push(file.insert(&tx)?);
        }

        tx.commit()?;

        Ok(files)
    }
}

impl File {
    /// Generate a pseudorandom hex string with the same length as SHA-256
    fn pseudorandom_hex_string() -> String {
        let mut generator = rand::thread_rng();

        (0..32)
            .into_iter()
            .map(|_| {
                let random_byte: u8 = generator.gen_range(0..=255);
                format!("{:02x}", random_byte)
            })
            .collect()
    }

    pub fn get_file_paths(db: &Database) -> DatabaseResult<Vec<PathBuf>> {
        let mut stmt = db.prepare(include_str!("sql/file/find_paths.sql"))?;
        let mut rows = stmt.query([])?;

        let mut paths = vec![];
        while let Some(row) = rows.next()? {
            let path = PathBuf::from(row.get::<_, String>(0)?);
            paths.push(path);
        }

        Ok(paths)
    }

    pub fn archive_size(db: &Database) -> DatabaseResult<u64> {
        let size = db.query_row(include_str!("sql/file/size.sql"), [], |row| row.get(0))?;
        Ok(size)
    }

    pub fn count(db: &Database) -> DatabaseResult<u32> {
        let count = db.query_row(include_str!("sql/file/count.sql"), [], |row| row.get(0))?;
        Ok(count)
    }

    /// Find files that need to be encrypted for the specified device
    pub fn find_need_encryption_files(db: &Database, device: &Device) -> DatabaseResult<Vec<File>> {
        let mut stmt = db.prepare(include_str!("sql/file/need_encryption.sql"))?;
        let mut rows = stmt.query([&device.platform_id])?;

        let mut files = vec![];
        while let Some(row) = rows.next()? {
            files.push(File::try_from(row)?);
        }

        Ok(files)
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
    fn test_pseudorandom_hex_string_is_valid_length_and_contains_valid_chars() {
        let valid_chars = "0123456789abcdfe";

        for _ in 0..10_000 {
            let result = File::pseudorandom_hex_string();
            assert_eq!(result.len(), 64);

            for chr in result.chars() {
                assert!(valid_chars.contains(chr));
            }
        }
    }

    #[test]
    fn test_insert_unique() {
        let database = create_in_memory().unwrap();

        let file1 = InsertFile::new(
            "foobar".to_string(),
            PathBuf::from("/path/to/foo7bar"),
            "asdas".to_string(),
            0,
        );

        assert!(file1.insert(&database).is_ok());

        let file2 = InsertFile::new(
            "foobar".to_string(),
            PathBuf::from("/path/to/foo7bar"),
            "bfsdfb".to_string(),
            0,
        );

        assert!(file2.insert(&database).is_err());
    }

    #[test]
    fn test_insert_and_fetch() {
        let database = create_in_memory().unwrap();

        let insert_file = InsertFile::new(
            "foobar".to_string(),
            PathBuf::from("/path/to/foo/bar"),
            "sdadfb".to_string(),
            0,
        );

        let inserted_file = insert_file.insert(&database).unwrap();

        let files = File::fetch_all(&database);

        let files = files.unwrap();

        assert_eq!(files.len(), 1);

        let fetched_file = files.get(0).unwrap().to_owned();

        assert_eq!(inserted_file, fetched_file);
    }

    #[test]
    fn test_insert_many() {
        let mut database = create_in_memory().unwrap();

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

        InsertFile::insert_many(&mut database, &insert_files).unwrap();
        let files = File::fetch_all(&database).unwrap();

        assert_eq!(files.len(), 128);
    }

    #[test]
    fn test_archive_size_and_count() {
        let database = create_in_memory().unwrap();

        let file = InsertFile::new(
            format!("foobar"),
            PathBuf::from("/path/to/foo/bar"),
            "test_hash_placeholder".to_string(),
            64,
        );

        file.insert(&database).unwrap();
        let archive_size = File::archive_size(&database).unwrap();

        assert_eq!(archive_size, 64);
    }

    #[test]
    fn test_enormous_archive_size() {
        let mut database = create_in_memory().unwrap();

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

        InsertFile::insert_many(&mut database, &insert_files).unwrap();
        let archive_size = File::archive_size(&database);
        let archive_size = archive_size.unwrap();

        assert_eq!(archive_size, 128 * 1_u64.pow(10));

        let archive_count = File::count(&database).unwrap();
        assert_eq!(archive_count, 128);
    }

    #[test]
    fn test_enormous_file_insert_and_count() {
        let mut database = create_in_memory().unwrap();

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

        InsertFile::insert_many(&mut database, &insert_files).unwrap();

        let archive_count = File::count(&database).unwrap();
        assert_eq!(archive_count, 8192);
    }
}
