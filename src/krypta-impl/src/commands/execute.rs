use cli::{CliCommand, Parser};
use database::Database;

use super::{init, status, sync};

/// Parse and execute command, if valid
pub async fn execute_command(database: &mut Database) -> anyhow::Result<()> {
    match cli::Cli::parse().command {
        CliCommand::Init {
            source_path,
            destination_path,
        } => init::execute(database, source_path, destination_path).await?,
        CliCommand::Sync => sync::execute(database).await?,
        CliCommand::Status => status::execute(database).await?,
    };

    Ok(())
}
