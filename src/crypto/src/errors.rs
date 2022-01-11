use thiserror::Error;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Input/Output error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("ChaChaError: {0}")]
    ChaChaError(String),
}
