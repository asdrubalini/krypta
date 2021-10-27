#![allow(dead_code, unused_variables)]

const MAX_CONCURRENT_FILE_OPERATIONS: usize = 256;

mod metadata;
pub use metadata::{Metadata, MetadataCollection};

mod path_finder;
pub use path_finder::{CuttablePathBuf, PathFinder};
