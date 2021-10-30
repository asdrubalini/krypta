/// `PathFinder` is a module that is able to, given a source_directory, recursively
/// find files and strip the host-specific bits from them, obtaining something
/// that can be safely inserted into a database.
///
/// A `PathFinder` instance holds the found paths which can be filtered to remove unwanted ones.
///
/// A `PathFinder` instance can be turned into a `MetadataCollection` which asynchronously
/// starts obtaining `Metadata` for each file, `MAX_CONCURRENT_FILE_OPERATIONS` at the same time.
// #![allow(dead_code, unused_variables)]

// const MAX_CONCURRENT_FILE_OPERATIONS: usize = 256;

// mod metadata;
// pub use metadata::{Metadata, MetadataCollection};
mod path_finder;
pub use path_finder::PathFinder;
