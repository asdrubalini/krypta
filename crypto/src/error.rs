use std::path::PathBuf;

pub type CryptoResult<T> = Result<T, CryptoError>;

#[derive(Debug)]
pub enum CryptoError {
    InvalidKeyLength,
    SourceFileNotFound(PathBuf),
    SourceFileIsAPath(PathBuf),
    CannotCreateDestinationFile(std::io::Error),
    FileWriteError(std::io::Error),
    FileReadError(std::io::Error),
    SodiumOxideError,
}
