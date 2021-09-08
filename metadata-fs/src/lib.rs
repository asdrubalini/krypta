#![allow(dead_code, unused_variables)]

mod metadata;

mod path_finder;
pub use path_finder::PathFinder;

const MAX_CONCURRENT_FILE_OPERATIONS: usize = 256;
