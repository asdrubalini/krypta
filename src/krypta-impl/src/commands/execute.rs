use cli::{CliCommand, Parser};
use database::Database;

use super::{config, debug, encrypt, force_sync, list, status, sync, unlock_structure};

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
        CliCommand::Tree => todo!(),
        CliCommand::List => list::list(database).await,
        CliCommand::Debug => debug::debug(database).await,
    }?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use rand::{prelude::SmallRng, Rng, SeedableRng};
    use tmp::{RandomFill, Tmp};

    use super::{config, encrypt, sync};

    fn init_paths_with_rng(rng: &mut impl Rng) -> (Tmp, Tmp) {
        let locked_path = Tmp::random_with_rng(rng);
        let unlocked_path = Tmp::random_with_rng(rng);

        unlocked_path.random_fill(25_000, rng).unwrap();

        (locked_path, unlocked_path)
    }

    #[tokio::test]
    async fn test_integration() {
        // TODO: do something useful here
        let mut db = database::create_in_memory().unwrap();

        let mut rng = SmallRng::seed_from_u64(1);

        let (locked_tmp, unlocked_tmp) = init_paths_with_rng(&mut rng);

        config::set_locked(&db, locked_tmp.base_path())
            .await
            .unwrap();
        config::set_unlocked(&db, unlocked_tmp.base_path())
            .await
            .unwrap();

        sync::sync(&mut db).await.unwrap();
        sync::sync(&mut db).await.unwrap();

        encrypt::encrypt(&mut db).await.unwrap();
        encrypt::encrypt(&mut db).await.unwrap();
    }
}
