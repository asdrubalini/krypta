use thiserror::Error;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("SQLx Error: {0}")]
    SQLx(#[from] sqlx::Error),
    #[error("Input/Output error")]
    IOError(#[from] std::io::Error),
}
