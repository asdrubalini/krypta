use crate::hash::{traits::BulkHashable, types::Sha256Hash};

use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct BulkSha256 {}

#[async_trait]
impl<'a> BulkHashable<Sha256Hash<'a>> for BulkSha256 {
    fn try_new(source_path: &std::path::Path) -> crate::error::CryptoResult<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    async fn start(self) -> crate::error::CryptoResult<Vec<Sha256Hash<'a>>> {
        todo!()
    }
}
