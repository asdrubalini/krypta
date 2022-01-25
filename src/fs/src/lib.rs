/// `PathFinder` is a module that is able to, given a source_directory, recursively
/// find files and strip the host-specific bits from them, obtaining something
/// that can be safely inserted, for example, into a database.
///
/// A `PathFinder` instance holds the found paths which can be filtered to remove unwanted ones.
mod path_finder;
pub use path_finder::PathFinder;

mod tree;
pub use tree::PathTree;

pub mod errors;
