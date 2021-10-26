use std::path::{Path, PathBuf};

use crate::{
    error::{CryptoError, CryptoResult},
    traits::{Computable, ConcurrentComputable},
    BUFFER_SIZE,
};

use async_trait::async_trait;
use bytes::BytesMut;
use sha2::{Digest, Sha256};
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};

/// Represents a Sha256 hash
#[derive(Default)]
pub struct Sha256Hash {
    hash: [u8; 32],
}

impl From<&[u8]> for Sha256Hash {
    /// Sha256 hash from bytes
    fn from(slice: &[u8]) -> Self {
        let mut hash = Self::default();
        hash.hash.copy_from_slice(slice);
        hash
    }
}

impl Sha256Hash {
    /// Convert self as an hex string
    pub fn as_hex(&self) -> String {
        self.hash.iter().map(|n| format!("{:02x}", n)).collect()
    }
}

#[derive(Debug, Clone)]
pub struct Sha256FileHasher {
    source_path: PathBuf,
}

impl Sha256FileHasher {
    /// Build a new SingleSha256 instance with file's source_path
    pub fn try_new<P: AsRef<Path>>(source_path: P) -> CryptoResult<Self> {
        let source_path = source_path.as_ref();
        let source_path_buf = source_path.to_path_buf();

        // Error out if source path does not exists or if is a directory
        if !source_path.exists() {
            return Err(CryptoError::SourceFileNotFound(source_path_buf));
        } else if source_path.is_dir() {
            return Err(CryptoError::SourceFileIsAPath(source_path_buf));
        }

        Ok(Self {
            source_path: source_path_buf,
        })
    }
}

#[async_trait]
impl Computable for Sha256FileHasher {
    type Output = Sha256Hash;

    async fn start(self) -> CryptoResult<Self::Output> {
        let file_input = File::open(&self.source_path)
            .await
            .or(Err(CryptoError::SourceFileNotFound(self.source_path)))?;

        // Source file reader
        let mut reader_input = BufReader::new(file_input);
        let mut buffer_input = BytesMut::with_capacity(BUFFER_SIZE);

        let mut hasher = Sha256::new();

        // Hash loop
        while let Ok(size) = reader_input.read_buf(&mut buffer_input).await {
            // Loop until both amount of data red into buffer is zero and the buffer is empty
            if size == 0 && buffer_input.is_empty() {
                break;
            }

            // Continue reading until either the buffer is full or the amount of data red is zero
            if buffer_input.len() < buffer_input.capacity() && size > 0 {
                continue;
            }

            hasher.update(&buffer_input);
            buffer_input.clear();
        }

        let hash: Sha256Hash = hasher.finalize().as_slice().into();

        Ok(hash)
    }
}

#[derive(Debug, Clone)]
pub struct Sha256ConcurrentFileHasher {
    hashers: Vec<Sha256FileHasher>,
}

impl Sha256ConcurrentFileHasher {
    pub fn try_new<P: AsRef<Path>>(source_paths: &[P]) -> CryptoResult<Self> {
        let mut hashers = Vec::new();

        for source_path in source_paths {
            hashers.push(Sha256FileHasher::try_new(source_path)?);
        }

        Ok(Self { hashers: hashers })
    }
}

impl ConcurrentComputable for Sha256ConcurrentFileHasher {
    type Computables = Sha256FileHasher;
    type Output = Sha256Hash;

    fn computables(&self) -> Vec<Self::Computables> {
        self.hashers.clone()
    }

    fn computable_result_to_output(
        result: CryptoResult<<Self::Computables as Computable>::Output>,
    ) -> Self::Output {
        result.unwrap()
    }
}
