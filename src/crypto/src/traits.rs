use std::{collections::HashMap, hash::Hash, sync::Arc};

use parking_lot::Mutex;
use rayon::ThreadPoolBuilder;

use crate::errors::CryptoError;

/// Something that can be computed asynchronously
pub trait Compute {
    type Output: Send;

    fn start(self) -> Result<Self::Output, CryptoError>;
}

/// Provide the ability to execute multiple `Computable` objects at once
pub trait ConcurrentCompute {
    type Compute: Compute + Send;
    type Key: Hash + Eq + Send;
    type Output: Send;

    /// Get a Vec of `Compute`s
    fn computables(&self) -> Vec<Self::Compute>;

    /// Map each `Compute` Result to an output
    fn computable_result_to_output(
        result: Result<<<Self as ConcurrentCompute>::Compute as Compute>::Output, CryptoError>,
    ) -> Self::Output;

    /// Map a `Compute` to its key
    fn computable_to_key(computable: &<Self as ConcurrentCompute>::Compute) -> Self::Key;

    fn concurrent_count() -> usize;

    /// Start Compute action in a concurrent manner
    fn start_all(self: Box<Self>) -> HashMap<Self::Key, Self::Output> {
        let concurrent_count = Self::concurrent_count();
        let pool = ThreadPoolBuilder::new()
            .num_threads(concurrent_count)
            .build()
            .unwrap();

        let computables = self.computables();
        // let mut output_map: Arc<Mutex<HashMap<Self::Key, Self::Output>>> =
        // Arc::new(Mutex::from(HashMap::new()));

        let output: Arc<Vec<Mutex<(Self::Key, Self::Output)>>>;

        let scope = pool.scope(|s| {
            for computable in computables {
                let output = Arc::clone(&output);

                let handle = s.spawn(|_| {
                    let key = Self::computable_to_key(&computable);
                    let result = computable.start();
                    let output = Self::computable_result_to_output(result);
                });
            }

            // Collect results
            for handle in handles {
                // TODO: handle thread error more gracefully
                let task_ret = handle.join().unwrap();

                output_map.insert(key, output);
            }

            output_map
        });

        // TODO: handle thread error more gracefully
        scope.unwrap()
    }
}
