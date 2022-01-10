use std::{
    fmt::Debug,
    fs::File,
    io::{self, BufRead, BufReader, Read},
    path::{Path, PathBuf},
};

use crate::{
    errors::CryptoError,
    traits::{Compute, ConcurrentCompute},
    BUFFER_SIZE,
};

use bytes::BytesMut;
use sha2::{Digest, Sha256};

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

impl Debug for Sha256Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_hex())
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
    pub fn try_new<P: AsRef<Path>>(source_path: P) -> Result<Self, CryptoError> {
        let source_path = source_path.as_ref();
        let source_path_buf = source_path.to_path_buf();

        // Error out if source path does not exists or if is a directory
        if !source_path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                source_path.as_os_str().to_str().unwrap(),
            )
            .into());
        } else if source_path.is_dir() {
            return Err(
                io::Error::new(io::ErrorKind::Other, "Expected file, found directory").into(),
            );
        }

        Ok(Self {
            source_path: source_path_buf,
        })
    }
}

impl Compute for Sha256FileHasher {
    type Output = Sha256Hash;

    fn start(self) -> Result<Self::Output, CryptoError> {
        let file_input = File::open(&self.source_path)?;

        // Source file reader
        let mut reader_input = BufReader::with_capacity(BUFFER_SIZE, file_input);

        let mut hasher = Sha256::new();

        // Hash loop
        loop {
            let buffer = reader_input.fill_buf()?;
            hasher.update(buffer);

            let length = buffer.len();

            if length == 0 {
                break;
            }

            reader_input.consume(length);
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
    pub fn try_new<P: AsRef<Path>>(source_paths: &[P]) -> Result<Box<Self>, CryptoError> {
        let mut hashers = Vec::new();

        for source_path in source_paths {
            hashers.push(Sha256FileHasher::try_new(source_path)?);
        }

        Ok(Box::new(Self { hashers }))
    }
}

impl ConcurrentCompute for Sha256ConcurrentFileHasher {
    type Compute = Sha256FileHasher;
    type Output = Sha256Hash;
    type Key = PathBuf;

    fn computables(&self) -> Vec<Self::Compute> {
        self.hashers.clone()
    }

    fn computable_result_to_output(
        result: Result<<Self::Compute as Compute>::Output, CryptoError>,
    ) -> Self::Output {
        result.unwrap()
    }

    fn computable_to_key(computable: &<Self as ConcurrentCompute>::Compute) -> Self::Key {
        computable.source_path.clone()
    }
}
