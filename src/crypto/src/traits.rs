use std::{any::type_name, collections::HashMap, hash::Hash};

use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::errors::CryptoError;

/// Something that can be computed
pub trait ComputeUnit {
    type Output: Send;

    fn start(self) -> Result<Self::Output, CryptoError>;
}

/// Provide the ability to execute multiple `Compute` objects at once
pub trait ComputeBulk {
    type Compute: ComputeUnit + Send;
    type Key: Hash + Eq + Send;
    type Output: Send;

    /// Get a Vec of `Compute`s
    fn units(&self) -> Vec<Self::Compute>;

    /// Map a `ComputeUnit` to its key
    fn map_key(unit: &<Self as ComputeBulk>::Compute) -> Self::Key;

    /// Map each `ComputeUnit` Result to an output
    fn map_output(
        result: Result<<<Self as ComputeBulk>::Compute as ComputeUnit>::Output, CryptoError>,
    ) -> Self::Output;

    /// Start `ComputeUnit` action in a concurrent manner
    fn start_all(self: Box<Self>) -> HashMap<Self::Key, Self::Output> {
        let computes = self.units();

        log::trace!(
            "[{}]: Starting job of {} computes",
            type_name::<Self::Compute>(),
            computes.len(),
        );

        computes
            .into_par_iter()
            .map(|compute| {
                let key = Self::map_key(&compute);
                let result = compute.start();
                let output = Self::map_output(result);

                log::trace!("[{}]: done", type_name::<Self::Compute>());

                (key, output)
            })
            .collect()
    }
}
