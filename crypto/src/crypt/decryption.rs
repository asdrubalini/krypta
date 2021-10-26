use std::path::{Path, PathBuf};

use crate::{
    error::SodiumOxideError,
    traits::{Computable, ConcurrentComputable},
    BUFFER_SIZE,
};

use async_trait::async_trait;
use bytes::BytesMut;
use sodiumoxide::crypto::secretstream::{Header, Key, Stream, ABYTES, HEADERBYTES};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
};

#[derive(Debug, Clone)]
pub struct FileDecryptor {
    source_path: PathBuf,
    destination_path: PathBuf,
    key: Key,
}

impl FileDecryptor {
    pub fn try_new<P: AsRef<Path>>(
        source_path: P,
        destination_path: P,
        key: &[u8; 32],
    ) -> anyhow::Result<FileDecryptor> {
        let key = Key::from_slice(key).ok_or(SodiumOxideError::InvalidKeyLength)?;

        Ok(FileDecryptor {
            source_path: source_path.as_ref().to_path_buf(),
            destination_path: destination_path.as_ref().to_path_buf(),
            key,
        })
    }
}

#[async_trait]
impl Computable for FileDecryptor {
    type Output = ();

    /// Try to decrypt a file as specified in struct
    async fn start(self) -> anyhow::Result<Self::Output> {
        // The file we are trying to decrypt
        let file_input = File::open(&self.source_path).await?;

        // The decrypted file
        let file_output = File::create(&self.destination_path).await?;

        // Source file reader
        let mut reader_input = BufReader::new(file_input);
        let mut buffer_input = BytesMut::with_capacity(BUFFER_SIZE + ABYTES);

        // Destination file writer
        let mut writer_output = BufWriter::new(file_output);

        // Read Header from file
        let mut header_buf = [0u8; HEADERBYTES];
        reader_input.read_exact(&mut header_buf).await?;

        let header = Header::from_slice(&header_buf).unwrap();
        let key = self.key;

        let mut decryption_stream =
            Stream::init_pull(&header, &key).map_err(|_| SodiumOxideError::InitPull)?;

        // Read -> Decrypt -> Write loop
        while let Ok(size) = reader_input.read_buf(&mut buffer_input).await {
            // Loop until both amount of data red into buffer is zero and the buffer is empty
            if size == 0 && buffer_input.is_empty() {
                break;
            }

            // Continue reading until either the buffer is full or the amount of data red is zero
            if buffer_input.len() < buffer_input.capacity() && size > 0 {
                continue;
            }

            // Decrypt
            let (result, _tag) = decryption_stream
                .pull(&buffer_input, None)
                .map_err(|_| SodiumOxideError::Pull)?;

            // Write to output buffer
            writer_output.write_all(&result).await?;

            buffer_input.clear();
        }

        writer_output.flush().await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileConcurrentDecryptor {
    decryptors: Option<Vec<FileDecryptor>>,
}

impl FileConcurrentDecryptor {
    pub fn try_new<P: AsRef<Path>>(source_paths: &[P]) -> anyhow::Result<Self> {
        let mut decryptors = Vec::new();

        todo!();
        // for source_path in source_paths {
        // encryptors.push(FileEncryptor::try_new(source_path)?);
        // }

        Ok(Self {
            decryptors: Some(decryptors),
        })
    }
}

impl ConcurrentComputable for FileConcurrentDecryptor {
    type Computables = FileDecryptor;
    type Output = bool;

    fn computables(&mut self) -> Vec<Self::Computables> {
        self.decryptors.take().expect("Cannot take computables")
    }

    fn computable_result_to_output(
        result: anyhow::Result<<Self::Computables as Computable>::Output>,
    ) -> Self::Output {
        result.is_ok()
    }
}
