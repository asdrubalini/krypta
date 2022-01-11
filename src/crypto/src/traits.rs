use std::{collections::HashMap, fmt::Display, hash::Hash, path::PathBuf};

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

        computes
            .into_par_iter()
            .map(|compute| {
                let key = Self::map_key(&compute);
                let result = compute.start();
                let output = Self::map_output(result);

                (key, output)
            })
            .collect()
    }
}

#[derive(Debug)]
pub struct PathPair {
    pub source: PathBuf,
    pub destination: PathBuf,
}

impl Display for PathPair {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
