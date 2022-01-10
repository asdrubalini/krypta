use std::{collections::HashMap, hash::Hash};

use crossbeam::thread;

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
    type Output;

    /// Get a Vec of `Compute`s
    fn computables(&self) -> Vec<Self::Compute>;

    /// Map each `Compute` Result to an output
    fn computable_result_to_output(
        result: Result<<<Self as ConcurrentCompute>::Compute as Compute>::Output, CryptoError>,
    ) -> Self::Output;

    /// Map a `Compute` to its key
    fn computable_to_key(computable: &<Self as ConcurrentCompute>::Compute) -> Self::Key;

    /// Start Compute action in a concurrent manner
    fn start_all(self: Box<Self>) -> HashMap<Self::Key, Self::Output> {
        let cpus_count = num_cpus::get();
        // TODO: restore semaphore here
        // let semaphore = Arc::new(Semaphore::new(cpus_count));

        let computables = self.computables();

        let scope = thread::scope(|s| {
            let mut handles = vec![];

            for computable in computables {
                // let permit = semaphore.clone().acquire_owned().await.unwrap();

                let handle = s.spawn(|_| {
                    let key = Self::computable_to_key(&computable);
                    let result = computable.start();

                    // drop(permit);
                    (key, result)
                });

                handles.push(handle);
            }

            let mut output_map: HashMap<Self::Key, Self::Output> = HashMap::new();

            // Collect results
            for handle in handles {
                // TODO: handle thread error more gracefully
                let task_ret = handle.join().unwrap();

                let key = task_ret.0;
                let output = Self::computable_result_to_output(task_ret.1);

                output_map.insert(key, output);
            }

            output_map
        });

        // TODO: handle thread error more gracefully
        scope.unwrap()
    }
}
