use std::path::{Path, PathBuf};

use crate::{
    error::{CryptoError, CryptoResult},
    traits::{Computable, ConcurrentComputable},
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
    fn try_new<P: AsRef<Path>>(
        source_path: P,
        destination_path: P,
        key: &[u8; 32],
    ) -> CryptoResult<FileEncryptor> {
        let key = Key::from_slice(key).ok_or(CryptoError::InvalidKeyLength)?;

        Ok(FileEncryptor {
            source_path: source_path.as_ref().to_path_buf(),
            destination_path: destination_path.as_ref().to_path_buf(),
            key,
        })
    }
}

#[async_trait]
impl Computable for FileEncryptor {
    type Output = ();

    /// Try to encrypt a file as specified in struct
    async fn start(self) -> CryptoResult<Self::Output> {
        let (mut encryption_stream, header) =
            Stream::init_push(&self.key).map_err(|_| CryptoError::SodiumOxideError)?;

        // The file we are trying to encrypt
        let file_input = File::open(&self.source_path)
            .await
            .map_err(|_| CryptoError::SourceFileNotFound(self.source_path))?;

        // The encrypted file
        let file_output = File::create(&self.destination_path)
            .await
            .map_err(CryptoError::CannotCreateDestinationFile)?;

        // Source file reader
        let mut reader_input = BufReader::new(file_input);
        let mut buffer_input = BytesMut::with_capacity(BUFFER_SIZE);

        // Destination file writer
        let mut writer_output = BufWriter::new(file_output);

        // Write header to file
        writer_output
            .write_all(&header.0)
            .await
            .map_err(CryptoError::FileWriteError)?;

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
                .map_err(|_| CryptoError::SodiumOxideError)?;

            // Write to output buffer
            writer_output
                .write_all(&result)
                .await
                .map_err(CryptoError::FileWriteError)?;

            buffer_input.clear();
        }

        writer_output
            .flush()
            .await
            .map_err(CryptoError::FileWriteError)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileConcurrentEncryptor {
    encryptors: Option<Vec<FileEncryptor>>,
}

impl FileConcurrentEncryptor {
    pub fn try_new<P: AsRef<Path>>(source_paths: &[P]) -> CryptoResult<Self> {
        let mut encryptors = Vec::new();

        // for source_path in source_paths {
        // encryptors.push(FileEncryptor::try_new(source_path)?);
        // }

        Ok(Self {
            encryptors: Some(encryptors),
        })
    }
}

impl ConcurrentComputable for FileConcurrentEncryptor {
    type Computables = FileEncryptor;
    type Output = bool;

    fn computables(&mut self) -> Vec<Self::Computables> {
        self.encryptors.take().expect("Cannot take computables")
    }

    fn computable_result_to_output(
        result: CryptoResult<<Self::Computables as Computable>::Output>,
    ) -> Self::Output {
        result.is_ok()
    }
}
