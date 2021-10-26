use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Semaphore;

use crate::error::CryptoResult;

/// Something that can be computed asynchronously
#[async_trait]
pub trait Computable {
    type Output: Send;

    async fn start(self) -> CryptoResult<Self::Output>;
}

/// Provide the ability to execute multiple `Computable` objects at once
#[async_trait]
pub trait ConcurrentComputable {
    type Computables: Computable + Send + 'static;
    type Output: Send;

    /// Get a Vec of `Computable`s
    fn computables(&mut self) -> Vec<Self::Computables>;

    /// Map each `Computable` Result to an output
    fn computable_result_to_output(
        result: CryptoResult<<<Self as ConcurrentComputable>::Computables as Computable>::Output>,
    ) -> Self::Output;

    /// Start Computable action in a concurrent manner
    async fn start_all(&mut self) -> Vec<Self::Output> {
        let cpus_count = num_cpus::get();
        let semaphore = Arc::new(Semaphore::new(cpus_count));

        let mut handles = Vec::new();

        let computables = self.computables();

        for computable in computables {
            let permit = semaphore.clone().acquire_owned().await.unwrap();

            let handle = tokio::spawn(async move {
                let result = computable.start().await;

                drop(permit);
                result
            });

            handles.push(handle);
        }

        let mut outputs: Vec<Self::Output> = Vec::new();

        for handle in handles {
            let result = handle.await.unwrap();
            outputs.push(Self::computable_result_to_output(result));
        }

        outputs
    }
}
