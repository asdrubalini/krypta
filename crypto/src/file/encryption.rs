use std::path::Path;

use crate::{
    error::{CryptoError, CryptoResult},
    BUFFER_SIZE,
};

use bytes::BytesMut;
use sodiumoxide::crypto::secretstream::{Key, Stream, Tag};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
};

pub struct FileEncryptor<'a> {
    source_path: &'a Path,
    destination_path: &'a Path,
    key: Key,
}

impl FileEncryptor<'_> {
    pub fn new<'a>(
        source_path: &'a Path,
        destination_path: &'a Path,
        key: &[u8; 32],
    ) -> CryptoResult<FileEncryptor<'a>> {
        let key = Key::from_slice(key).ok_or(CryptoError::InvalidKeyLength)?;

        Ok(FileEncryptor {
            source_path,
            destination_path,
            key,
        })
    }

    /// Try to encrypt a file as specified in struct
    pub async fn try_encrypt(self) -> CryptoResult<()> {
        let (mut encryption_stream, header) = Stream::init_push(&self.key).unwrap();

        // The file we are trying to encrypt
        let file_input = File::open(&self.source_path)
            .await
            .map_err(CryptoError::SourceFileNotFound)?;

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
            if size == 0 {
                break;
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
