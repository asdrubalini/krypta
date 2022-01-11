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
    traits::{ComputeBulk, ComputeUnit, PathPair},
    BUFFER_SIZE,
};

#[derive(Debug, Clone)]
pub struct FileEncryptUnit {
    // The source file
    plaintext_path: PathBuf,
    // The destination file
    encrypted_path: PathBuf,
    key: [u8; 32],
    nonce: [u8; 24],
}

impl From<&FileEncryptUnit> for PathPair {
    fn from(unit: &FileEncryptUnit) -> Self {
        PathPair {
            source: unit.plaintext_path.clone(),
            destination: unit.encrypted_path.clone(),
        }
    }
}

impl FileEncryptUnit {
    pub fn try_new<P: AsRef<Path>>(
        plaintext_path: P,
        encrypted_path: P,
        key: [u8; 32],
        nonce: [u8; 24],
    ) -> Result<FileEncryptUnit, CryptoError> {
        let plaintext_path = plaintext_path.as_ref().to_path_buf();

        // Make sure that plaintext path exists
        File::open(&plaintext_path)?;

        Ok(FileEncryptUnit {
            plaintext_path,
            encrypted_path: encrypted_path.as_ref().to_path_buf(),
            key,
            nonce,
        })
    }
}

impl ComputeUnit for FileEncryptUnit {
    type Output = ();

    /// Try to encrypt a file as specified in struct
    /// TODO: rethink of names
    fn start(self) -> Result<Self::Output, CryptoError> {
        let plaintext_file = File::open(&self.plaintext_path)?;
        let encrypted_file = File::create(&self.encrypted_path)?;

        // Zero-sized files cannot mmapped into memory
        if plaintext_file.metadata()?.len() == 0 {
            panic!("Cannot encrypt zero-sized file");
        }

        let aead = XChaCha20Poly1305::new(self.key.as_ref().into());
        let mut stream_encryptor =
            stream::EncryptorBE32::from_aead(aead, self.nonce.as_ref().into());

        let source_file_map = unsafe { MmapOptions::new().map(&plaintext_file)? };
        let mut encrypted_file_buf = BufWriter::new(encrypted_file);

        let mut plaintext_chunks = source_file_map.chunks(BUFFER_SIZE).peekable();

        // Encrypt and write loop
        let last_chunk = loop {
            // This should never be None since we look ahead with peek and break early if
            // next is the last chunk
            let chunk = plaintext_chunks.next().unwrap();

            if plaintext_chunks.peek().is_none() {
                break chunk;
            }

            // TODO: handle error with something better than unwrap
            let ciphertext = stream_encryptor.encrypt_next(chunk).map_err(|_| {
                CryptoError::CipherOperationError(CipherOperationError::EncryptNext, (&self).into())
            })?;
            encrypted_file_buf.write_all(&ciphertext)?;
        };

        // encrypt_last consume and must be called at the very end
        let ciphertext = stream_encryptor.encrypt_last(last_chunk).map_err(|_| {
            CryptoError::CipherOperationError(CipherOperationError::EncryptLast, (&self).into())
        })?;
        encrypted_file_buf.write_all(&ciphertext)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileEncryptBulk {
    encryptors: Vec<FileEncryptUnit>,
}

impl FileEncryptBulk {
    pub fn try_new<P: AsRef<Path>>(
        paths: &[(P, P)],
        key: [u8; 32],
        nonce: [u8; 24],
    ) -> Result<Box<Self>, CryptoError> {
        let mut encryptors = vec![];

        for (source_path, destination_path) in paths {
            let source_path = source_path.as_ref();
            let destination_path = destination_path.as_ref();

            encryptors.push(FileEncryptUnit::try_new(
                source_path,
                destination_path,
                key,
                nonce,
            )?);
        }

        Ok(Box::new(Self { encryptors }))
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
        unit.plaintext_path.clone()
    }

    fn map_output(
        result: Result<<<Self as ComputeBulk>::Compute as ComputeUnit>::Output, CryptoError>,
    ) -> Self::Output {
        result.is_ok()
    }
}
