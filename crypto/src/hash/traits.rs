use std::path::Path;

use async_trait::async_trait;

use crate::error::CryptoResult;

#[async_trait]
pub trait SingleHashable<T> {
    fn try_new(source_path: &Path) -> CryptoResult<Self>
    where
        Self: Sized;

    async fn start(self) -> CryptoResult<T>;
}

#[async_trait]
pub trait BulkHashable<T> {
    fn try_new(source_path: &Path) -> CryptoResult<Self>
    where
        Self: Sized;

    async fn start(self) -> CryptoResult<Vec<T>>;
}
