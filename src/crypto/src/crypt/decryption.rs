use std::{
    fs::File,
    path::{Path, PathBuf},
};

use crate::{
    errors::CryptoError,
    traits::{ComputeBulk, ComputeUnit},
    BUFFER_SIZE,
};

#[derive(Debug, Clone)]
pub struct FileDecryptUnit {
    encrypted_path: PathBuf,
    plaintext_path: PathBuf,
    key: [u8; 32],
}

impl FileDecryptUnit {
    pub fn try_new<P: AsRef<Path>>(
        encrypted_path: P,
        plaintext_path: P,
        key: [u8; 32],
    ) -> Result<FileDecryptUnit, CryptoError> {
        File::open(&encrypted_path)?;

        Ok(FileDecryptUnit {
            encrypted_path: encrypted_path.as_ref().to_path_buf(),
            plaintext_path: plaintext_path.as_ref().to_path_buf(),
            key,
        })
    }
}

impl ComputeUnit for FileDecryptUnit {
    type Output = ();

    /// Try to encrypt a file as specified in struct
    fn start(self) -> Result<Self::Output, CryptoError> {
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileDecryptBulk {
    encryptors: Vec<FileDecryptUnit>,
}

impl FileDecryptBulk {
    pub fn try_new<P: AsRef<Path>>(paths: &[(P, P)], key: [u8; 32]) -> Result<Self, CryptoError> {
        let mut encryptors = vec![];

        for (source_path, destination_path) in paths {
            let source_path = source_path.as_ref();
            let destination_path = destination_path.as_ref();

            encryptors.push(FileDecryptUnit::try_new(
                source_path,
                destination_path,
                key,
            )?);
        }

        Ok(Self { encryptors })
    }
}

impl ComputeBulk for FileDecryptBulk {
    type Compute = FileDecryptUnit;
    type Output = bool;
    type Key = PathBuf;

    fn units(&self) -> Vec<Self::Compute> {
        self.encryptors.clone()
    }

    fn map_key(unit: &<Self as ComputeBulk>::Compute) -> Self::Key {
        unit.encrypted_path.clone()
    }

    fn map_output(
        result: Result<<<Self as ComputeBulk>::Compute as ComputeUnit>::Output, CryptoError>,
    ) -> Self::Output {
        result.is_ok()
    }
}
