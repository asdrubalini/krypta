use thiserror::Error;

#[derive(Error, Debug)]
pub enum FsError {
    #[error("Input/Output error")]
    IoError(#[from] std::io::Error),
}
