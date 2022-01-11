use std::{
    fs::File,
    path::{Path, PathBuf},
};

use chacha20poly1305::{
    aead::{stream, NewAead},
    XChaCha20Poly1305,
};
use memmap::MmapOptions;

use crate::{
    errors::CryptoError,
    traits::{ComputeBulk, ComputeUnit},
    BUFFER_SIZE,
};

#[derive(Debug, Clone)]
pub struct FileEncryptUnit {
    plaintext_path: PathBuf,
    encrypted_path: PathBuf,
    key: [u8; 32],
    nonce: [u8; 24],
}

impl FileEncryptUnit {
    pub fn try_new<P: AsRef<Path>>(
        plaintext_path: P,
        destination_path: P,
        key: [u8; 32],
        nonce: [u8; 24],
    ) -> Result<FileEncryptUnit, CryptoError> {
        File::open(&plaintext_path)?;

        Ok(FileEncryptUnit {
            plaintext_path: plaintext_path.as_ref().to_path_buf(),
            encrypted_path: destination_path.as_ref().to_path_buf(),
            key,
            nonce,
        })
    }
}

impl ComputeUnit for FileEncryptUnit {
    type Output = ();

    /// Try to encrypt a file as specified in struct
    fn start(self) -> Result<Self::Output, CryptoError> {
        let source_file = File::open(&self.plaintext_path)?;
        let encrypted_file = File::create(&self.encrypted_path)?;

        // Handle zero-sized files without mmaping them into memory
        if source_file.metadata()?.len() == 0 {
            panic!("Cannot encrypt zero-sized file");
        }

        let aead = XChaCha20Poly1305::new(self.key.as_ref().into());
        let mut stream_encryptor =
            stream::EncryptorBE32::from_aead(aead, self.nonce.as_ref().into());

        let source_file_map = unsafe { MmapOptions::new().map(&source_file).unwrap() };

        // Encrypt loop
        // for (i, chunk) in source_file_map.chunks(BUFFER_SIZE).enumerate() {
        // let ciphertext = if i == last_chunk_index {
        // // Last chunk
        // let c = stream_encryptor.encrypt_last(chunk).map_err(|_err| {
        // CryptoError::ChaChaError("Error during encryption".to_string())
        // })?;
        // break;
        // c
        // } else {
        // stream_encryptor.encrypt_next(chunk).map_err(|_err| {
        // CryptoError::ChaChaError("Error during encryption".to_string())
        // })?
        // };
        // }

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
    ) -> Result<Self, CryptoError> {
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

        Ok(Self { encryptors })
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
