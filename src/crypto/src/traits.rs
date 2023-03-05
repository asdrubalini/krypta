use std::{any::type_name, collections::HashMap, hash::Hash, time::Instant};

use indicatif::{ParallelProgressIterator, ProgressStyle};
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

        if computes.is_empty() {
            log::trace!(
                "[{}]: zero computes found, not starting job",
                type_name::<Self::Compute>(),
            );

            return HashMap::new();
        }

        log::trace!(
            "[{}]: Starting job of {} computes",
            type_name::<Self::Compute>(),
            computes.len(),
        );

        let start = Instant::now();

        let progress_bar_style = ProgressStyle::with_template(
            "[{elapsed_precise}] {spinner} {bar:40.cyan/blue} {pos:>7}/{len:7}",
        )
        .unwrap();

        let results = computes
            .into_par_iter()
            .progress_with_style(progress_bar_style)
            .map(|compute| {
                let key = Self::map_key(&compute);
                let result = compute.start();
                let output = Self::map_output(result);

                (key, output)
            })
            .collect::<HashMap<Self::Key, Self::Output>>();

        log::trace!(
            "[{}] Took {:?} for processing {} items",
            type_name::<Self::Compute>(),
            start.elapsed(),
            results.len()
        );

        results
    }
}
