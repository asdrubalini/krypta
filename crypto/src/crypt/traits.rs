use std::path::Path;

use async_trait::async_trait;

use crate::error::CryptoResult;

#[async_trait]
pub trait SingleCryptable {
    fn try_new(source_path: &Path, destination_path: &Path, key: &[u8; 32]) -> CryptoResult<Self>
    where
        Self: Sized;

    async fn start(self) -> CryptoResult<()>;
}
