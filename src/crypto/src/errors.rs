use thiserror::Error;

use crate::crypt::PathPair;

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
    #[error("Key with length of {0} bytes is not valid")]
    InvalidKeyLength(usize),
    #[error("Nonce with length of {0} bytes is not valid")]
    InvalidNonceLength(usize),
}
