use crate::hash::{traits::SingleHashable, types::Sha256Hash};

use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct SingleSha256 {}

#[async_trait]
impl SingleHashable<Sha256Hash> for SingleSha256 {
    fn try_new(source_path: &std::path::Path) -> crate::error::CryptoResult<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    async fn start(self) -> crate::error::CryptoResult<Sha256Hash> {
        todo!()
    }
}
