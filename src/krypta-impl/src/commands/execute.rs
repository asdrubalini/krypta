use cli::{CliCommand, Parser};
use database::Database;

use super::{debug, sync};

/// Parse and execute command, if valid
pub async fn execute_command(database: &mut Database) -> anyhow::Result<()> {
    match cli::Cli::parse().command {
        CliCommand::SetUnlocked { unlocked_path: _ } => todo!(),
        CliCommand::SetLocked { locked_path: _ } => todo!(),
        CliCommand::Status => todo!(),
        CliCommand::Sync => sync::execute(database).await,
        CliCommand::Encrypt => todo!(),
        CliCommand::UnlockStructure => todo!(),
        CliCommand::Unlock => todo!(),
        CliCommand::Find { query: _ } => todo!(),
        CliCommand::Debug => debug::execute(database).await,
    }?;

    Ok(())
}
