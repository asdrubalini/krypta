use std::path::{Path, PathBuf};

use crate::{
    crypt::traits::SingleCryptable,
    error::{CryptoError, CryptoResult},
    BUFFER_SIZE,
};

use async_trait::async_trait;
use bytes::BytesMut;
use sodiumoxide::crypto::secretstream::{Header, Key, Stream, ABYTES, HEADERBYTES};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter},
};

#[derive(Debug)]
pub struct SingleFileDecryptor {
    source_path: PathBuf,
    destination_path: PathBuf,
    key: Key,
}

#[async_trait]
impl SingleCryptable for SingleFileDecryptor {
    fn try_new(
        source_path: &Path,
        destination_path: &Path,
        key: &[u8; 32],
    ) -> CryptoResult<SingleFileDecryptor> {
        let key = Key::from_slice(key).ok_or(CryptoError::InvalidKeyLength)?;

        Ok(SingleFileDecryptor {
            source_path: source_path.to_path_buf(),
            destination_path: destination_path.to_path_buf(),
            key,
        })
    }

    /// Try to decrypt a file as specified in struct
    async fn start(self) -> CryptoResult<()> {
        // The file we are trying to decrypt
        let file_input = File::open(&self.source_path)
            .await
            .map_err(CryptoError::SourceFileNotFound)?;

        // The decrypted file
        let file_output = File::create(&self.destination_path)
            .await
            .map_err(CryptoError::CannotCreateDestinationFile)?;

        // Source file reader
        let mut reader_input = BufReader::new(file_input);
        let mut buffer_input = BytesMut::with_capacity(BUFFER_SIZE + ABYTES);

        // Destination file writer
        let mut writer_output = BufWriter::new(file_output);

        // Read Header from file
        let mut header_buf = [0u8; HEADERBYTES];
        reader_input
            .read_exact(&mut header_buf)
            .await
            .map_err(CryptoError::FileReadError)?;

        let header = Header::from_slice(&header_buf).unwrap();
        let key = self.key;

        let mut decryption_stream =
            Stream::init_pull(&header, &key).map_err(|_| CryptoError::SodiumOxideError)?;

        // Read -> Decrypt -> Write loop
        while let Ok(size) = reader_input.read_buf(&mut buffer_input).await {
            // Loop until both amount of data red into buffer is zero and the buffer is empty
            if size == 0 && buffer_input.len() == 0 {
                break;
            }

            // Continue reading until either the buffer is full or the amount of data red is zero
            if buffer_input.len() < buffer_input.capacity() && size > 0 {
                continue;
            }

            // Decrypt
            let (result, _tag) = decryption_stream
                .pull(&buffer_input, None)
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
