use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use chacha20poly1305::{aead::stream, ChaCha20Poly1305, KeyInit, XChaCha20Poly1305};
use memmap2::MmapOptions;

use crate::{
    errors::{CipherOperationError, CryptoError},
    traits::{ComputeBulk, ComputeUnit},
    BUFFER_SIZE,
};

use super::{KeyArray, NonceArray, PathPair};

#[derive(Debug, Clone)]
pub struct FileEncryptUnit {
    // The source file
    unlocked_path: PathBuf,
    // The destination file
    locked_path: PathBuf,
    key: KeyArray,
    nonce: NonceArray,
}

impl From<&FileEncryptUnit> for PathPair {
    fn from(unit: &FileEncryptUnit) -> Self {
        PathPair {
            source: unit.unlocked_path.clone(),
            destination: unit.locked_path.clone(),
        }
    }
}

impl FileEncryptUnit {
    pub fn try_new<P: AsRef<Path>>(
        unlocked_path: P,
        locked_path: P,
        key: KeyArray,
        nonce: NonceArray,
    ) -> Result<FileEncryptUnit, CryptoError> {
        let unlocked_path = unlocked_path.as_ref().to_path_buf();

        // Make sure that plaintext path exists
        File::open(&unlocked_path)?;

        Ok(FileEncryptUnit {
            unlocked_path,
            locked_path: locked_path.as_ref().to_path_buf(),
            key,
            nonce,
        })
    }
}

impl ComputeUnit for FileEncryptUnit {
    type Output = ();

    /// Try to encrypt a file as specified in struct
    fn start(self) -> Result<Self::Output, CryptoError> {
        let unlocked_file = File::open(&self.unlocked_path)?;
        let locked_file = File::create(&self.locked_path)?;

        let aead = XChaCha20Poly1305::new(&self.key);
        let mut stream_encryptor = stream::EncryptorLE31::from_aead(aead, &self.nonce);

        // Zero-sized files cannot be mmapped into memory
        if unlocked_file.metadata()?.len() == 0 {
            let mut locked_file_buf = BufWriter::new(locked_file);
            let empty: &[u8] = &[];

            let ciphertext = stream_encryptor.encrypt_last(empty).map_err(|_| {
                CryptoError::CipherOperationError(
                    CipherOperationError::EncryptLast,
                    PathPair::from(&self),
                )
            })?;

            locked_file_buf.write_all(&ciphertext)?;

            return Ok(());
        }

        // SAFETY: nobody else is accessing this file
        let unlocked_file_map = unsafe { MmapOptions::new().map(&unlocked_file)? };
        let mut locked_file_buf = BufWriter::new(locked_file);

        let mut unlocked_chunks = unlocked_file_map.chunks(BUFFER_SIZE).peekable();

        // Encrypt and write loop
        let last_chunk = loop {
            // This should never be None since we look ahead with peek and break early if
            // next is the last chunk
            let chunk = unlocked_chunks.next().unwrap();

            if unlocked_chunks.peek().is_none() {
                break chunk;
            }

            let ciphertext = stream_encryptor.encrypt_next(chunk).map_err(|_| {
                CryptoError::CipherOperationError(
                    CipherOperationError::EncryptNext,
                    PathPair::from(&self),
                )
            })?;
            locked_file_buf.write_all(&ciphertext)?;
        };

        // encrypt_last consume and must be called at the very end
        let ciphertext = stream_encryptor.encrypt_last(last_chunk).map_err(|_| {
            CryptoError::CipherOperationError(
                CipherOperationError::EncryptLast,
                PathPair::from(&self),
            )
        })?;
        locked_file_buf.write_all(&ciphertext)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileEncryptBulk {
    encryptors: Vec<FileEncryptUnit>,
}

impl FileEncryptBulk {
    pub fn new(encryptors: impl IntoIterator<Item = FileEncryptUnit>) -> Box<Self> {
        Box::new(Self {
            encryptors: encryptors.into_iter().collect(),
        })
    }
}

impl ComputeBulk for FileEncryptBulk {
    type Compute = FileEncryptUnit;
    type Output = Result<(), CryptoError>;
    type Key = PathBuf;

    fn units(&self) -> Vec<Self::Compute> {
        self.encryptors.clone()
    }

    fn map_key(unit: &<Self as ComputeBulk>::Compute) -> Self::Key {
        unit.unlocked_path.clone()
    }

    fn map_output(
        result: Result<<<Self as ComputeBulk>::Compute as ComputeUnit>::Output, CryptoError>,
    ) -> Self::Output {
        result
    }
}
