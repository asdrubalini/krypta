use async_trait::async_trait;

use crate::error::CryptoResult;

/// Something that can be computed asynchronously
#[async_trait]
pub trait Computable {
    type Output: Send;

    async fn start(self) -> CryptoResult<Self::Output>;
}
