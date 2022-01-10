use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read},
    path::{Path, PathBuf},
};

use crate::{
    errors::CryptoError,
    traits::{Compute, ConcurrentCompute},
    BUFFER_SIZE,
};

use bytes::BytesMut;
use sodiumoxide::crypto::secretstream::{Header, Key, Stream, ABYTES, HEADERBYTES};

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
    ) -> Result<FileDecryptor, CryptoError> {
        todo!()
        // let key = Key::from_slice(key)
        // .ok_or(CryptoError::SodiumOxide(SodiumOxideError::InvalidKeyLength))?;

        // Ok(FileDecryptor {
        // source_path: source_path.as_ref().to_path_buf(),
        // destination_path: destination_path.as_ref().to_path_buf(),
        // key,
        // })
    }
}

impl Compute for FileDecryptor {
    type Output = ();

    /// Try to decrypt a file as specified in struct
    fn start(self) -> Result<Self::Output, CryptoError> {
        // The file we are trying to decrypt
        let file_input = File::open(&self.source_path)?;

        // The decrypted file
        let file_output = File::create(&self.destination_path)?;

        // Source file reader
        let mut reader_input = BufReader::new(file_input);
        let mut buffer_input = BytesMut::with_capacity(BUFFER_SIZE + ABYTES);

        // Destination file writer
        let mut writer_output = BufWriter::new(file_output);

        // Read Header from file
        let mut header_buf = [0u8; HEADERBYTES];
        reader_input.read_exact(&mut header_buf)?;

        // let header = Header::from_slice(&header_buf).unwrap();
        // let key = self.key;

        // let mut decryption_stream = Stream::init_pull(&header, &key)
        // .map_err(|_| CryptoError::SodiumOxide(SodiumOxideError::InitPull))?;

        // Read -> Decrypt -> Write loop
        loop {
            let buffer = reader_input.fill_buf()?;
            // TODO: decrypt here

            let length = buffer.len();

            if length == 0 {
                break;
            }

            reader_input.consume(length);
        }

        // writer_output.flush().await?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct FileConcurrentDecryptor {
    decryptors: Vec<FileDecryptor>,
}

impl FileConcurrentDecryptor {
    pub fn try_new<P: AsRef<Path>>(paths: &[(P, P)], key: &[u8; 32]) -> Result<Self, CryptoError> {
        let mut decryptors = vec![];

        for (source_path, destination_path) in paths {
            let source_path = source_path.as_ref();
            let destination_path = destination_path.as_ref();

            decryptors.push(FileDecryptor::try_new(source_path, destination_path, key)?);
        }

        Ok(Self { decryptors })
    }
}

impl ConcurrentCompute for FileConcurrentDecryptor {
    type Compute = FileDecryptor;
    type Output = bool;
    type Key = PathBuf;

    fn computables(&self) -> Vec<Self::Compute> {
        self.decryptors.clone()
    }

    fn computable_result_to_output(
        result: Result<<Self::Compute as Compute>::Output, CryptoError>,
    ) -> Self::Output {
        result.is_ok()
    }

    fn computable_to_key(computable: &<Self as ConcurrentCompute>::Compute) -> Self::Key {
        computable.source_path.clone()
    }
}
