use std::any::type_name;
use std::path::Path;
use std::time::Instant;
use std::{fs::Metadata, path::PathBuf};

use chrono::{DateTime, Utc};
use crypto::crypt::{
    generate_random_secure_key_nonce_pair, FileEncryptUnit, AEAD_KEY_SIZE, AEAD_NONCE_SIZE,
};
use crypto::errors::CryptoError;
use database_macros::{Insert, TableName, TryFromRow};
use fs::PathTree;
use rand::SeedableRng;
use rand_chacha::ChaCha20Rng;
use rusqlite::named_params;
use utils::RandomString;

use crate::{errors::DatabaseResult, Database};

use crate::traits::{Count, FetchAll, InsertMany, Search, TryFromRow, Update, UpdateMany};

use super::Tag;

#[derive(TableName, TryFromRow, Insert, Debug, Clone, PartialEq, Eq)]
pub struct File {
    pub id: Option<i64>,
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

impl Count for File {}

impl FetchAll for File {}

impl Search for File {
    /// Search files stored in database
    fn search(db: &Database, query: impl AsRef<str>) -> DatabaseResult<Vec<Self>> {
        let mut stmt = db.prepare(include_str!("sql/file/search.sql"))?;
        let mut rows = stmt.query(named_params! {
            ":query": format!("%{}%", query.as_ref())
        })?;

        let mut files = vec![];
        while let Some(row) = rows.next()? {
            files.push(File::try_from_row(row)?);
        }

        Ok(files)
    }
}

impl InsertMany for File {}

impl Update for File {
    fn update(mut self, db: &Database) -> DatabaseResult<File> {
        assert_ne!(self.id, None);

        self.update_updated_at(); // Make sure that updated_at is up to date

        let file = db.query_row(
            include_str!("sql/file/update.sql"),
            named_params! {
                ":title": self.title,
                ":path": self.path,
                ":random_hash": self.random_hash,
                ":contents_hash": self.contents_hash,
                ":size": self.size,
                ":created_at": self.created_at,
                ":updated_at": self.updated_at,
                ":key": self.key,
                ":nonce": self.nonce,
                ":id": self.id
            },
            |row| File::try_from_row(row),
        )?;

        Ok(file)
    }
}

impl UpdateMany for File {}

impl From<&File> for PathBuf {
    fn from(file: &File) -> Self {
        PathBuf::from(&file.path)
    }
}

impl File {
    /// Build a new `File` and generate on the fly some stuff
    pub fn new(title: String, path: PathBuf, contents_hash: String, size: u64) -> Self {
        let random_hash = File::random_hash_string();
        let now = chrono::Utc::now();

        // Key and nonce generation
        let (key, nonce) = generate_random_secure_key_nonce_pair();
        let key = Vec::from(key.as_slice());
        let nonce = Vec::from(nonce.as_slice());

        File {
            id: None,
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

    /// Generate a pseudorandom 32 bytes hex string
    fn random_hash_string() -> String {
        let mut generator = ChaCha20Rng::from_entropy();
        RandomString::hex_with_rng(&mut generator, 32)
    }

    /// Update `updated_at` field to now
    fn update_updated_at(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Get file matching path
    fn find_file_from_path(db: &Database, path: &Path) -> DatabaseResult<File> {
        let file = db.query_row(
            include_str!("sql/file/find_file_from_path.sql"),
            named_params! { ":path":path.to_string_lossy() },
            |row| File::try_from_row(row),
        )?;

        Ok(file)
    }

    /// Get files matching paths
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

    /// Get the total size of the archive
    pub fn archive_size(db: &Database) -> DatabaseResult<u64> {
        let size = db.query_row(include_str!("sql/file/size.sql"), [], |row| row.get("size"))?;
        Ok(size)
    }

    /// Convert self into a crypto::Encryptor, if possible
    pub fn try_into_encryptor<P: AsRef<Path>>(
        self,
        locked_path: P,
        source_path: P,
    ) -> Result<FileEncryptUnit, CryptoError> {
        // Build absolute paths
        let mut source = source_path.as_ref().to_owned();
        source.push(self.path);

        let mut locked = locked_path.as_ref().to_owned();
        locked.push(self.random_hash);

        // Should never fail as key and nonce lens are constant
        let key: [u8; AEAD_KEY_SIZE] = self.key.try_into().unwrap();
        let nonce: [u8; AEAD_NONCE_SIZE] = self.nonce.try_into().unwrap();

        FileEncryptUnit::try_new(source, locked, key.into(), nonce.into())
    }

    /// Get a list of tags related to a File
    pub fn tags(&self, db: &Database) -> DatabaseResult<Vec<Tag>> {
        let mut stmt = db.prepare(include_str!("sql/file/tags.sql"))?;
        let mut rows = stmt.query(named_params! {
            ":file_id": self.id.expect("missing file.id")
        })?;

        let mut tags = vec![];
        while let Some(row) = rows.next()? {
            tags.push(Tag::try_from_row(row)?);
        }

        Ok(tags)
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
    pub fn into_file(self, contents_hash: String) -> File {
        File::new(self.title, self.path, contents_hash, self.size)
    }
}

impl FromIterator<File> for PathTree {
    fn from_iter<T: IntoIterator<Item = File>>(files: T) -> Self {
        files
            .into_iter()
            .map(|f| f.path)
            .map(PathBuf::from)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    use chrono::{DateTime, NaiveDateTime, Utc};
    use crypto::crypt::{AEAD_KEY_SIZE, AEAD_NONCE_SIZE};
    use utils::RandomString;

    use crate::create_in_memory;
    use crate::models::{FileTag, Tag};
    use crate::traits::{Count, FetchAll, Insert, InsertMany, Update};

    use super::File;

    fn new_random_file() -> File {
        File::new(
            RandomString::alphanum(10),
            PathBuf::from(format!("foo/bar/{}", RandomString::alphanum(10))),
            File::random_hash_string(),
            1337,
        )
    }

    #[test]
    fn test_pseudorandom_hex_string_is_valid_length_and_contains_valid_chars() {
        let valid_chars = "0123456789abcdfe";

        for _ in 0..10_000 {
            let result = File::random_hash_string();
            assert_eq!(result.len(), 64);

            for chr in result.chars() {
                assert!(valid_chars.contains(chr));
            }
        }
    }

    #[test]
    fn test_insert_unique() {
        let database = create_in_memory().unwrap();

        let file1 = File::new(
            "foobar".to_string(),
            PathBuf::from("/path/to/foo7bar"),
            "asdas".to_string(),
            0,
        );

        assert_eq!(File::count(&database).unwrap(), 0);
        assert!(file1.insert(&database).is_ok());
        assert_eq!(File::count(&database).unwrap(), 1);

        let file2 = File::new(
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

        let insert_file = File::new(
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
                File::new(
                    format!("foobar_{}", i),
                    PathBuf::from(format!("/path/to/foo/bar/{}", i)),
                    format!("test_hash_placeholder_{}", i),
                    0,
                )
            })
            .collect::<Vec<File>>();

        File::insert_many(&mut database, insert_files).unwrap();
        let files = File::fetch_all(&database).unwrap();

        assert_eq!(files.len(), 128);
    }

    #[test]
    fn test_archive_size_and_count() {
        let database = create_in_memory().unwrap();

        let file = File::new(
            "foobar".to_string(),
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
                File::new(
                    format!("foobar_{}", i),
                    PathBuf::from(format!("/path/to/foo/bar/{}", i)),
                    format!("test_hash_placeholder_{}", i),
                    1_u64.pow(10), // 10 GB
                )
            })
            .collect::<Vec<File>>();

        File::insert_many(&mut database, insert_files).unwrap();
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
                File::new(
                    format!("foobar_{}", i),
                    PathBuf::from(format!("/path/to/foo/bar/{}", i)),
                    format!("test_hash_placeholder_{}", i),
                    0,
                )
            })
            .collect::<Vec<File>>();

        File::insert_many(&mut database, insert_files).unwrap();

        let archive_count = File::count(&database).unwrap();
        assert_eq!(archive_count, 8192);
    }

    #[test]
    fn test_file_new() {
        let file = File::new(
            String::from("x.txt"),
            PathBuf::from("foo/bar/x.txt"),
            File::random_hash_string(),
            1337,
        );

        assert_eq!(file.id, None);
        assert_eq!(file.title, "x.txt".to_string());
        assert_eq!(file.path, String::from("foo/bar/x.txt"));
        assert_eq!(PathBuf::from(&file), PathBuf::from("foo/bar/x.txt"));
        assert_eq!(file.random_hash.len(), 64);
        assert_eq!(file.contents_hash.len(), 64);
        assert_eq!(file.size, 1337);
        assert_ne!(
            file.created_at,
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc)
        );
        assert_ne!(
            file.updated_at,
            DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(0, 0), Utc)
        );
        assert_eq!(file.key.len(), AEAD_KEY_SIZE);
        assert_eq!(file.nonce.len(), AEAD_NONCE_SIZE);
    }

    #[test]
    fn test_insert_and_update() {
        let database = create_in_memory().unwrap();

        let file = new_random_file();

        let inserted = file.insert(&database).unwrap();
        thread::sleep(Duration::from_millis(10));
        let updated = inserted.clone().update(&database).unwrap();

        assert_ne!(updated.id, None);
        assert_eq!(inserted.created_at, updated.created_at);
        assert_ne!(inserted.updated_at, updated.updated_at);
    }

    #[test]
    #[should_panic]
    fn test_update_non_existing() {
        let database = create_in_memory().unwrap();

        let file = new_random_file();
        file.update(&database).unwrap();
    }

    #[test]
    fn test_file_tags() {
        let database = create_in_memory().unwrap();

        let file = new_random_file().insert(&database).unwrap();
        let tag1 = Tag::new("random-tag").insert(&database).unwrap();
        let tag2 = Tag::new("other-tag").insert(&database).unwrap();

        FileTag::new(&file, &tag1).insert(&database).unwrap();
        FileTag::new(&file, &tag2).insert(&database).unwrap();

        let found_tags = file.tags(&database).unwrap();

        assert_eq!(found_tags.len(), 2);
        assert_eq!(found_tags.get(0).unwrap().name, String::from("random-tag"));
        assert_eq!(found_tags.get(1).unwrap().name, String::from("other-tag"));
    }

    #[test]
    fn test_find_file_from_path() {
        let database = create_in_memory().unwrap();

        let inserted_file = new_random_file().insert(&database).unwrap();
        let found_file =
            File::find_file_from_path(&database, &PathBuf::from(&inserted_file)).unwrap();

        assert_eq!(inserted_file, found_file);
    }
}
