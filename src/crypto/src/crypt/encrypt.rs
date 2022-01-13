use std::{
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use chacha20poly1305::{
    aead::{stream, NewAead},
    XChaCha20Poly1305,
};
use memmap2::MmapOptions;

use crate::{
    errors::{CipherOperationError, CryptoError},
    traits::{ComputeBulk, ComputeUnit},
    BUFFER_SIZE,
};

use super::{PathPair, AEAD_KEY_SIZE, AEAD_NONCE_SIZE};

#[derive(Debug, Clone)]
pub struct FileEncryptUnit {
    // The source file
    unlocked_path: PathBuf,
    // The destination file
    locked_path: PathBuf,
    key: Box<[u8; AEAD_KEY_SIZE]>,
    nonce: Box<[u8; AEAD_NONCE_SIZE]>,
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
        key: [u8; AEAD_KEY_SIZE],
        nonce: [u8; AEAD_NONCE_SIZE],
    ) -> Result<FileEncryptUnit, CryptoError> {
        let unlocked_path = unlocked_path.as_ref().to_path_buf();

        // Make sure that plaintext path exists
        File::open(&unlocked_path)?;

        Ok(FileEncryptUnit {
            unlocked_path,
            locked_path: locked_path.as_ref().to_path_buf(),
            key: Box::new(key),
            nonce: Box::new(nonce),
        })
    }
}

impl ComputeUnit for FileEncryptUnit {
    type Output = ();

    /// Try to encrypt a file as specified in struct
    fn start(self) -> Result<Self::Output, CryptoError> {
        let unlocked_file = File::open(&self.unlocked_path)?;
        let locked_file = File::create(&self.locked_path)?;

        // Zero-sized files cannot mmapped into memory
        if unlocked_file.metadata()?.len() == 0 {
            return Err(CryptoError::ZeroLength(self.locked_path));
        }

        let aead = XChaCha20Poly1305::new(self.key.as_ref().into());
        let mut stream_encryptor =
            stream::EncryptorBE32::from_aead(aead, self.nonce.as_ref().into());

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
                CryptoError::CipherOperationError(CipherOperationError::EncryptNext, (&self).into())
            })?;
            locked_file_buf.write_all(&ciphertext)?;
        };

        // encrypt_last consume and must be called at the very end
        let ciphertext = stream_encryptor.encrypt_last(last_chunk).map_err(|_| {
            CryptoError::CipherOperationError(CipherOperationError::EncryptLast, (&self).into())
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
    type Output = bool;
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
        result.is_ok()
    }
}
