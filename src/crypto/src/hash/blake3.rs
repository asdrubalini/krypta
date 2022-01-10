use std::{
    fmt::Debug,
    fs::File,
    path::{Path, PathBuf},
};

use memmap::MmapOptions;

use crate::{
    errors::CryptoError,
    traits::{ComputeBulk, ComputeUnit},
    BUFFER_SIZE,
};

#[derive(Debug, Clone)]
pub struct Blake3File {
    source_path: PathBuf,
}

impl Blake3File {
    /// Build a new Blake3File instance with file's path
    pub fn try_new<P: AsRef<Path>>(source_path: P) -> Result<Self, CryptoError> {
        let source_path = source_path.as_ref().to_path_buf();

        // Attempt to open file first
        File::open(&source_path)?;

        Ok(Blake3File { source_path })
    }
}

impl ComputeUnit for Blake3File {
    type Output = blake3::Hash;

    fn start(self) -> Result<Self::Output, CryptoError> {
        let file_input = File::open(&self.source_path)?;

        // Handle zero-sized files without mmaping them into memory
        if file_input.metadata()?.len() == 0 {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&[]);
            return Ok(hasher.finalize());
        }

        let mmap = unsafe { MmapOptions::new().map(&file_input).unwrap() };

        let mut hasher = blake3::Hasher::new();

        // Hash loop
        for chunk in mmap.chunks(BUFFER_SIZE) {
            hasher.update(chunk);
        }

        Ok(hasher.finalize())
    }
}

#[derive(Debug, Clone)]
pub struct Blake3Concurrent {
    hashers: Vec<Blake3File>,
}

impl Blake3Concurrent {
    pub fn try_new<P: AsRef<Path>>(source_paths: &[P]) -> Result<Box<Self>, CryptoError> {
        let mut hashers = Vec::new();

        for source_path in source_paths {
            hashers.push(Blake3File::try_new(source_path)?);
        }

        Ok(Box::new(Self { hashers }))
    }
}

impl ComputeBulk for Blake3Concurrent {
    type Compute = Blake3File;
    type Output = blake3::Hash;
    type Key = PathBuf;

    fn units(&self) -> Vec<Self::Compute> {
        self.hashers.clone()
    }

    fn map_key(computable: &<Self as ComputeBulk>::Compute) -> Self::Key {
        computable.source_path.clone()
    }

    fn map_output(
        result: Result<<Self::Compute as ComputeUnit>::Output, CryptoError>,
    ) -> Self::Output {
        result.unwrap()
    }
}
