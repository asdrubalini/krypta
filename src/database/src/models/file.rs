use std::any::type_name;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use std::{fs::Metadata, path::PathBuf};

use chrono::{DateTime, Utc};
use crypto::crypt::{
    generate_random_secure_key_nonce_pair, FileEncryptUnit, AEAD_KEY_SIZE, AEAD_NONCE_SIZE,
};
use crypto::errors::CryptoError;
use rand::Rng;
use rusqlite::{params, Row};

use crate::{errors::DatabaseResult, Database};

use crate::traits::{Count, Fetch, Insert, InsertMany, Search, Update, UpdateMany};

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
        let file = db.query_row(
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

        Ok(file)
    }
}

impl InsertMany<File> for InsertFile {
    fn insert_many(db: &mut Database, items: &[Self]) -> DatabaseResult<Vec<File>> {
        let tx = db.transaction()?;
        let mut files = vec![];

        log::trace!(
            "[{}] Start inserting {} File",
            type_name::<Self>(),
            items.len()
        );

        let start = Instant::now();

        for file in items {
            files.push(file.insert(&tx)?);
        }

        tx.commit()?;

        log::trace!(
            "[{}] Took {:?} for inserting {} items",
            type_name::<Self>(),
            start.elapsed(),
            items.len()
        );

        Ok(files)
    }
}

impl Count for File {
    fn count(db: &Database) -> DatabaseResult<i64> {
        let count = db.query_row(include_str!("sql/file/count.sql"), [], |row| row.get(0))?;
        Ok(count)
    }
}

impl Update for File {
    fn update(&self, db: &Database) -> DatabaseResult<Self> {
        let file = db.query_row(
            include_str!("sql/file/update.sql"),
            params![
                self.title,
                self.path.to_string_lossy(),
                self.random_hash,
                self.contents_hash,
                self.size,
                self.created_at,
                self.updated_at,
                self.key,
                self.nonce,
                self.id
            ],
            |row| File::try_from(row),
        )?;

        Ok(file)
    }
}

impl UpdateMany for File {
    fn update_many(db: &mut Database, updatables: &[Self]) -> DatabaseResult<Vec<Self>> {
        let tx = db.transaction()?;
        let mut results = vec![];

        log::trace!(
            "[{}] Start updading {} File",
            type_name::<Self>(),
            updatables.len()
        );

        let start = Instant::now();

        for updatable in updatables {
            results.push(updatable.update(&tx)?);
        }

        tx.commit()?;

        log::trace!(
            "[{}] Took {:?} for updating {} items",
            type_name::<Self>(),
            start.elapsed(),
            updatables.len()
        );

        Ok(results)
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

    pub fn find_known_paths_with_last_modified(
        db: &Database,
        device: &Device,
    ) -> DatabaseResult<HashMap<PathBuf, i64>> {
        let mut stmt = db.prepare(include_str!(
            "sql/file/find_known_paths_with_last_modified.sql"
        ))?;
        let mut rows = stmt.query([device.to_owned().platform_id])?;

        let mut items = HashMap::new();
        while let Some(row) = rows.next()? {
            let path = PathBuf::from(row.get::<_, String>(0)?);
            let last_modified = row.get::<_, i64>(1)?;

            items.insert(path, last_modified);
        }

        Ok(items)
    }

    fn find_file_from_path(db: &Database, path: &Path) -> DatabaseResult<File> {
        let file = db.query_row(
            include_str!("sql/file/find_file_from_path.sql"),
            params![path.to_string_lossy()],
            |row| File::try_from(row),
        )?;

        Ok(file)
    }

    pub fn find_files_from_paths(
        db: &mut Database,
        paths: &[impl AsRef<Path>],
    ) -> DatabaseResult<Vec<Self>> {
        let tx = db.transaction()?;
        let mut results = vec![];

        log::trace!(
            "[{}] Start finding {} paths",
            type_name::<Self>(),
            paths.len()
        );

        let start = Instant::now();

        for path in paths {
            let path = path.as_ref();
            results.push(Self::find_file_from_path(&tx, path)?);
        }

        tx.commit()?;

        log::trace!(
            "[{}] Took {:?} for finding {} items",
            type_name::<Self>(),
            start.elapsed(),
            paths.len()
        );

        Ok(results)
    }

    pub fn archive_size(db: &Database) -> DatabaseResult<u64> {
        let size = db.query_row(include_str!("sql/file/size.sql"), [], |row| row.get(0))?;
        Ok(size)
    }

    /// Find files that need to be encrypted for the specified device
    pub fn find_need_encryption(db: &Database, device: &Device) -> DatabaseResult<Vec<File>> {
        let mut stmt = db.prepare(include_str!("sql/file/need_encryption.sql"))?;
        let mut rows = stmt.query([&device.platform_id])?;

        let mut files = vec![];
        while let Some(row) = rows.next()? {
            files.push(File::try_from(row)?);
        }

        Ok(files)
    }

    pub fn try_into_encryptor<P: AsRef<Path>>(
        self,
        locked_path: P,
        unlocked_path: P,
    ) -> Result<FileEncryptUnit, CryptoError> {
        let mut unlocked = unlocked_path.as_ref().to_owned();
        unlocked.push(self.path);

        let mut locked = locked_path.as_ref().to_owned();
        locked.push(self.random_hash);

        let key: [u8; AEAD_KEY_SIZE] = self.key.try_into().unwrap();
        let nonce: [u8; AEAD_NONCE_SIZE] = self.nonce.try_into().unwrap();

        FileEncryptUnit::try_new(unlocked, locked, key, nonce)
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

    use crate::traits::{Count, Fetch, Insert, InsertMany};
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

        assert_eq!(File::count(&database).unwrap(), 0);
        assert!(file1.insert(&database).is_ok());
        assert_eq!(File::count(&database).unwrap(), 1);

        let file2 = InsertFile::new(
            "foobar".to_string(),
            PathBuf::from("/path/to/foo7bar"),
            "bfsdfb".to_string(),
            0,
        );

        assert!(file2.insert(&database).is_err());
        assert_eq!(File::count(&database).unwrap(), 1);
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
        assert_eq!(File::count(&database).unwrap(), 1);

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
