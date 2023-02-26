pub use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(about, long_about = None, version, author)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Get the status of the current database
    Status,

    /// Find something based on file name, path or tag name
    Find {
        query: String,
    },

    /// Display files tree
    Tree,

    /// List all the files
    List,

    Debug,
}
