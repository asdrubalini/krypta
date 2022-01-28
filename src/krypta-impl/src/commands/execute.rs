use cli::{CliCommand, Parser};
use database::Database;

use super::{config, debug, encrypt, force_sync, status, sync, tree, unlock_structure};

/// Parse and execute command, if valid
pub async fn execute_command(database: &mut Database) -> anyhow::Result<()> {
    match cli::Cli::parse().command {
        CliCommand::SetUnlocked { unlocked_path } => {
            config::set_unlocked(database, unlocked_path).await
        }
        CliCommand::SetLocked { locked_path } => config::set_locked(database, locked_path).await,
        CliCommand::Status => status::status(database).await,
        CliCommand::Sync => sync::sync(database).await,
        CliCommand::Encrypt => encrypt::encrypt(database).await,
        CliCommand::ForceSync => force_sync::force_sync(database).await,
        CliCommand::UnlockStructure => unlock_structure::unlock_structure(database).await,
        CliCommand::Unlock => todo!(),
        CliCommand::Find { query: _ } => todo!(),
        CliCommand::Tree => tree::tree(database).await,
        CliCommand::Debug => debug::debug(database).await,
    }?;

    Ok(())
}
