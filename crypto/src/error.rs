use thiserror::Error;

#[derive(Error, Debug)]
pub enum SodiumOxideError {
    #[error("InitPull")]
    InitPull,
    #[error("InitPush")]
    InitPush,
    #[error("Pull")]
    Pull,
    #[error("Push")]
    Push,
    #[error("InvalidKeyLength")]
    InvalidKeyLength,
}

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("SodiumOxide failed: {0}")]
    SodiumOxide(SodiumOxideError),
    #[error("Input/Output error: {0}")]
    IoError(#[from] std::io::Error),
}
