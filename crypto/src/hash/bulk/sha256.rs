use crate::hash::{single::SingleSha256, traits::BulkHashable, types::Sha256Hash};

use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct BulkSha256 {
    hashers: Vec<SingleSha256>,
}

#[async_trait]
impl<'a> BulkHashable<Sha256Hash> for BulkSha256 {
    fn try_new(hashers: Vec<Sha256Hash>) -> crate::error::CryptoResult<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    async fn start(self) -> crate::error::CryptoResult<Vec<Sha256Hash>> {
        todo!()
    }
}
