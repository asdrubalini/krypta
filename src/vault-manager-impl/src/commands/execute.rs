use cli::{CliCommand, Parser};

use crate::database::Database;

use super::{init, status, sync};

/// Parse and execute command, if valid
pub async fn execute_command(database: &Database) {
    match cli::Cli::parse().command {
        CliCommand::Init { path } => init::execute(database, path).await,
        CliCommand::Sync => sync::execute(database).await,
        CliCommand::Status => status::execute(database).await,
    };
}