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
    #[clap(name = "set-unlocked")]
    SetUnlocked {
        unlocked_path: PathBuf,
    },
    #[clap(name = "set-locked")]
    SetLocked {
        locked_path: PathBuf,
    },
    Status,
    Add,
    Encrypt,
    #[clap(name = "unlock-structure")]
    UnlockStructure,
    Unlock,
    Find {
        query: String,
    },
}
