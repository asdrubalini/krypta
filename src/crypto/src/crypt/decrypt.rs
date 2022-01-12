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

use super::{PathPair, AEAD_KEY_SIZE, AEAD_NONCE_SIZE, AEAD_TAG_SIZE};

#[derive(Debug, Clone)]
pub struct FileDecryptUnit {
    // The source file
    locked_path: PathBuf,
    // The destination file
    unlocked_path: PathBuf,
    key: Box<[u8; AEAD_KEY_SIZE]>,
    nonce: Box<[u8; AEAD_NONCE_SIZE]>,
}

impl From<&FileDecryptUnit> for PathPair {
    fn from(unit: &FileDecryptUnit) -> Self {
        PathPair {
            source: unit.locked_path.clone(),
            destination: unit.unlocked_path.clone(),
        }
    }
}

impl FileDecryptUnit {
    pub fn try_new<P: AsRef<Path>>(
        locked_path: P,
        unlocked_path: P,
        key: [u8; AEAD_KEY_SIZE],
        nonce: [u8; AEAD_NONCE_SIZE],
    ) -> Result<FileDecryptUnit, CryptoError> {
        let locked_path = locked_path.as_ref().to_path_buf();

        // Make sure that plaintext path exists
        File::open(&locked_path)?;

        Ok(FileDecryptUnit {
            locked_path,
            unlocked_path: unlocked_path.as_ref().to_path_buf(),
            key: Box::new(key),
            nonce: Box::new(nonce),
        })
    }
}

impl ComputeUnit for FileDecryptUnit {
    type Output = ();

    /// Try to encrypt a file as specified in struct
    fn start(self) -> Result<Self::Output, CryptoError> {
        let locked_file = File::open(&self.locked_path)?;
        let unlocked_file = File::create(&self.unlocked_path)?;

        // Zero-sized files cannot mmapped into memory
        if locked_file.metadata()?.len() == 0 {
            return Err(CryptoError::ZeroLength(self.locked_path));
        }

        let aead = XChaCha20Poly1305::new(self.key.as_ref().into());
        let mut stream_decryptor =
            stream::DecryptorBE32::from_aead(aead, self.nonce.as_ref().into());

        let locked_file_map = unsafe { MmapOptions::new().map(&locked_file)? };
        let mut unlocked_file_buf = BufWriter::new(unlocked_file);

        let mut locked_chunks = locked_file_map
            .chunks(BUFFER_SIZE + AEAD_TAG_SIZE)
            .peekable();

        // Encrypt and write loop
        let last_chunk = loop {
            // This should never be None since we look ahead with peek and break early if
            // next is the last chunk
            let chunk = locked_chunks.next().unwrap();

            if locked_chunks.peek().is_none() {
                break chunk;
            }

            let plaintext = stream_decryptor.decrypt_next(chunk).map_err(|_| {
                CryptoError::CipherOperationError(CipherOperationError::DecryptNext, (&self).into())
            })?;
            unlocked_file_buf.write_all(&plaintext)?;
        };

        // decrypt_last consume and must be called at the very end
        let plaintext = stream_decryptor.decrypt_last(last_chunk).map_err(|_| {
            CryptoError::CipherOperationError(CipherOperationError::DecryptLast, (&self).into())
        })?;
        unlocked_file_buf.write_all(&plaintext)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileDecryptBulk {
    decryptors: Vec<FileDecryptUnit>,
}

impl FileDecryptBulk {
    pub fn new(decryptors: Vec<FileDecryptUnit>) -> Box<Self> {
        Box::new(Self { decryptors })
    }
}

impl ComputeBulk for FileDecryptBulk {
    type Compute = FileDecryptUnit;
    type Output = bool;
    type Key = PathBuf;

    fn units(&self) -> Vec<Self::Compute> {
        self.decryptors.clone()
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
