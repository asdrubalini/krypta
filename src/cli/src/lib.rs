use std::path::PathBuf;

pub use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(about, version, author)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: CliCommand,
}

#[derive(Subcommand, Debug)]
pub enum CliCommand {
    /// Set the unlocked path for the current device
    #[clap(name = "set-unlocked")]
    SetUnlocked { unlocked_path: PathBuf },

    /// Set the locked path for the current device
    #[clap(name = "set-locked")]
    SetLocked { locked_path: PathBuf },

    /// Get the status of the current database
    Status,

    /// Update the database based on the unlocked path
    Sync,

    /// Encrypt files that require encryption
    Encrypt,

    /// Unlock just the folder structure in the unlocked_path, without creating files
    #[clap(name = "unlock-structure")]
    UnlockStructure,

    /// Unlock specified file in the unlocked_path
    Unlock,

    /// Find something based on file name, path or tag name
    Find { query: String },
}
