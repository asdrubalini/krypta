use cli::{CliCommand, Parser};
use database::Database;

use super::sync;

/// Parse and execute command, if valid
pub async fn execute_command(database: &mut Database) -> anyhow::Result<()> {
    match cli::Cli::parse().command {
        CliCommand::SetUnlocked { unlocked_path } => todo!(),
        CliCommand::SetLocked { locked_path } => todo!(),
        CliCommand::Status => todo!(),
        CliCommand::Sync => sync::execute(database),
        CliCommand::Encrypt => todo!(),
        CliCommand::UnlockStructure => todo!(),
        CliCommand::Unlock => todo!(),
        CliCommand::Find { query } => todo!(),
    }
    .await?;

    Ok(())
}
