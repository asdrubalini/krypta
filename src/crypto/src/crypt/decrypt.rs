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
    encrypted_path: PathBuf,
    // The destination file
    plaintext_path: PathBuf,
    key: Box<[u8; AEAD_KEY_SIZE]>,
    nonce: Box<[u8; AEAD_NONCE_SIZE]>,
}

impl From<&FileDecryptUnit> for PathPair {
    fn from(unit: &FileDecryptUnit) -> Self {
        PathPair {
            source: unit.encrypted_path.clone(),
            destination: unit.plaintext_path.clone(),
        }
    }
}

impl FileDecryptUnit {
    pub fn try_new<P: AsRef<Path>>(
        encrypted_path: P,
        plaintext_path: P,
        key: [u8; AEAD_KEY_SIZE],
        nonce: [u8; AEAD_NONCE_SIZE],
    ) -> Result<FileDecryptUnit, CryptoError> {
        let encrypted_path = encrypted_path.as_ref().to_path_buf();

        // Make sure that plaintext path exists
        File::open(&encrypted_path)?;

        Ok(FileDecryptUnit {
            encrypted_path,
            plaintext_path: plaintext_path.as_ref().to_path_buf(),
            key: Box::new(key),
            nonce: Box::new(nonce),
        })
    }
}

impl ComputeUnit for FileDecryptUnit {
    type Output = ();

    /// Try to encrypt a file as specified in struct
    fn start(self) -> Result<Self::Output, CryptoError> {
        let encrypted_file = File::open(&self.encrypted_path)?;
        let plaintext_file = File::create(&self.plaintext_path)?;

        // Zero-sized files cannot mmapped into memory
        if encrypted_file.metadata()?.len() == 0 {
            return Err(CryptoError::ZeroLength(self.encrypted_path));
        }

        let aead = XChaCha20Poly1305::new(self.key.as_ref().into());
        let mut stream_decryptor =
            stream::DecryptorBE32::from_aead(aead, self.nonce.as_ref().into());

        let encrypted_file_map = unsafe { MmapOptions::new().map(&encrypted_file)? };
        let mut plaintext_file_buf = BufWriter::new(plaintext_file);

        let mut encrypted_chunks = encrypted_file_map
            .chunks(BUFFER_SIZE + AEAD_TAG_SIZE)
            .peekable();

        // Encrypt and write loop
        let last_chunk = loop {
            // This should never be None since we look ahead with peek and break early if
            // next is the last chunk
            let chunk = encrypted_chunks.next().unwrap();

            if encrypted_chunks.peek().is_none() {
                break chunk;
            }

            let plaintext = stream_decryptor.decrypt_next(chunk).map_err(|_| {
                CryptoError::CipherOperationError(CipherOperationError::DecryptNext, (&self).into())
            })?;
            plaintext_file_buf.write_all(&plaintext)?;
        };

        // decrypt_last consume and must be called at the very end
        let plaintext = stream_decryptor.decrypt_last(last_chunk).map_err(|_| {
            CryptoError::CipherOperationError(CipherOperationError::DecryptLast, (&self).into())
        })?;
        plaintext_file_buf.write_all(&plaintext)?;

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
        unit.plaintext_path.clone()
    }

    fn map_output(
        result: Result<<<Self as ComputeBulk>::Compute as ComputeUnit>::Output, CryptoError>,
    ) -> Self::Output {
        result.is_ok()
    }
}
