use thiserror::Error;

pub type DatabaseResult<T> = Result<T, DatabaseError>;

#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("Sqlite error: {0}")]
    SQLx(#[from] rusqlite::Error),
    #[error("Input/Output error")]
    IOError(#[from] std::io::Error),
}
