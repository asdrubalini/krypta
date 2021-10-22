use std::path::{Path, PathBuf};

use crate::{
    error::{CryptoError, CryptoResult},
    hash::{traits::SingleHashable, types::Sha256Hash},
    BUFFER_SIZE,
};

use async_trait::async_trait;
use bytes::BytesMut;
use sha2::{Digest, Sha256};
use tokio::{
    fs::File,
    io::{AsyncReadExt, BufReader},
};

#[derive(Debug, Clone)]
pub struct SingleSha256 {
    source_path: PathBuf,
}

#[async_trait]
impl SingleHashable<Sha256Hash> for SingleSha256 {
    fn try_new(source_path: &Path) -> CryptoResult<Self> {
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

    async fn start(self) -> CryptoResult<Sha256Hash> {
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
            if size == 0 && buffer_input.len() == 0 {
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
