use std::{collections::HashMap, hash::Hash, sync::Arc};

use async_trait::async_trait;
use tokio::sync::Semaphore;

/// Something that can be computed asynchronously
#[async_trait]
pub trait Computable {
    type Output: Send;

    async fn start(self) -> anyhow::Result<Self::Output>;
}

/// Provide the ability to execute multiple `Computable` objects at once
#[async_trait]
pub trait ConcurrentComputable {
    type Computable: Computable + Send + 'static;
    type Key: Hash + Eq + Send + 'static;
    type Output: Send;

    /// Get a Vec of `Computable`s
    fn computables(&mut self) -> Vec<Self::Computable>;

    /// Map each `Computable` Result to an output
    fn computable_result_to_output(
        result: anyhow::Result<<<Self as ConcurrentComputable>::Computable as Computable>::Output>,
    ) -> Self::Output;

    /// Map a computable to its key
    fn computable_to_key(computable: &<Self as ConcurrentComputable>::Computable) -> Self::Key;

    /// Start Computable action in a concurrent manner
    async fn start_all(&mut self) -> HashMap<Self::Key, Self::Output> {
        let cpus_count = num_cpus::get();
        let semaphore = Arc::new(Semaphore::new(cpus_count));

        let mut handles = Vec::new();

        let computables = self.computables();

        for computable in computables {
            let permit = semaphore.clone().acquire_owned().await.unwrap();

            let handle = tokio::spawn(async move {
                let key = Self::computable_to_key(&computable);
                let result = computable.start().await;

                drop(permit);
                (key, result)
            });

            handles.push(handle);
        }

        let mut output_map: HashMap<Self::Key, Self::Output> = HashMap::new();

        for handle in handles {
            let task_ret = handle.await.unwrap();

            let key = task_ret.0;
            let output = Self::computable_result_to_output(task_ret.1);

            output_map.insert(key, output);
        }

        output_map
    }
}
