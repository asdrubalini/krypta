use std::path::PathBuf;

use thiserror::Error;

use crate::traits::PathPair;

#[derive(Error, Debug)]
pub enum CipherOperationError {
    #[error("EncryptNext")]
    EncryptNext,
    #[error("EncryptLast")]
    EncryptLast,
    #[error("DecryptNext")]
    DecryptNext,
    #[error("DecryptLast")]
    DecryptLast,
}

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Input/Output error: {0}")]
    InputOutput(#[from] std::io::Error),
    #[error("Error while performing {0} cipher operation {:?} {:?}", .1.source, .1.destination)]
    CipherOperationError(CipherOperationError, PathPair),
    #[error("Length of {0:?} cannot be zero")]
    ZeroLength(PathBuf),
}
