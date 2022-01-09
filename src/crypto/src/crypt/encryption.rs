use std::path::{Path, PathBuf};

use crate::{
    errors::{CryptoError, SodiumOxideError},
    traits::{Compute, ConcurrentCompute},
    BUFFER_SIZE,
};

use async_trait::async_trait;
use bytes::BytesMut;
use sodiumoxide::crypto::secretstream::{Key, Stream, Tag};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
};

#[derive(Debug, Clone)]
pub struct FileEncryptor {
    source_path: PathBuf,
    destination_path: PathBuf,
    key: Key,
}

impl FileEncryptor {
    pub fn try_new<P: AsRef<Path>>(
        source_path: P,
        destination_path: P,
        key: &[u8; 32],
    ) -> Result<FileEncryptor, CryptoError> {
        let key = Key::from_slice(key)
            .ok_or(CryptoError::SodiumOxide(SodiumOxideError::InvalidKeyLength))?;

        Ok(FileEncryptor {
            source_path: source_path.as_ref().to_path_buf(),
            destination_path: destination_path.as_ref().to_path_buf(),
            key,
        })
    }
}

#[async_trait]
impl Compute for FileEncryptor {
    type Output = ();

    /// Try to encrypt a file as specified in struct
    async fn start(self) -> Result<Self::Output, CryptoError> {
        let (mut encryption_stream, header) = Stream::init_push(&self.key)
            .map_err(|_| CryptoError::SodiumOxide(SodiumOxideError::InitPush))?;

        // The file we are trying to encrypt
        let file_input = File::open(&self.source_path).await?;

        // The encrypted file
        let file_output = File::create(&self.destination_path).await?;

        // Source file reader
        let mut reader_input = BufReader::new(file_input);
        let mut buffer_input = BytesMut::with_capacity(BUFFER_SIZE);

        // Destination file writer
        let mut writer_output = BufWriter::new(file_output);

        // Write header to file
        writer_output.write_all(&header.0).await?;

        // Read -> Encrypt -> Write loop
        while let Ok(size) = reader_input.read_buf(&mut buffer_input).await {
            // Loop until both amount of data red into buffer is zero and the buffer is empty
            if size == 0 && buffer_input.is_empty() {
                break;
            }

            // Continue reading until either the buffer is full or the amount of data red is zero
            if buffer_input.len() < buffer_input.capacity() && size > 0 {
                continue;
            }

            // Encrypt
            let result = encryption_stream
                .push(&buffer_input, None, Tag::Message)
                .map_err(|_| CryptoError::SodiumOxide(SodiumOxideError::Push))?;

            // Write to output buffer
            writer_output.write_all(&result).await?;

            buffer_input.clear();
        }

        writer_output.flush().await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileConcurrentEncryptor {
    encryptors: Vec<FileEncryptor>,
}

impl FileConcurrentEncryptor {
    pub fn try_new<P: AsRef<Path>>(paths: &[(P, P)], key: &[u8; 32]) -> Result<Self, CryptoError> {
        let mut encryptors = vec![];

        for (source_path, destination_path) in paths {
            let source_path = source_path.as_ref();
            let destination_path = destination_path.as_ref();

            encryptors.push(FileEncryptor::try_new(source_path, destination_path, key)?);
        }

        Ok(Self { encryptors })
    }
}

impl ConcurrentCompute for FileConcurrentEncryptor {
    type Computable = FileEncryptor;
    type Output = bool;
    type Key = PathBuf;

    fn computables(&self) -> Vec<Self::Computable> {
        self.encryptors.clone()
    }

    fn computable_result_to_output(
        result: Result<<Self::Computable as Compute>::Output, CryptoError>,
    ) -> Self::Output {
        result.is_ok()
    }

    fn computable_to_key(computable: &<Self as ConcurrentCompute>::Computable) -> Self::Key {
        computable.source_path.clone()
    }
}
