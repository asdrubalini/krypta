use cli::{CliCommand, Parser};
use database::Database;

#[cfg(debug_assertions)]
use super::prune;

use super::{add, check, config, debug, find, list, status, tree};

/// Parse and execute command, if valid
pub async fn execute_command(database: &mut Database) -> anyhow::Result<()> {
    match cli::Cli::parse().command {
        CliCommand::Config { key, value } => config::config(key, value).await,
        CliCommand::Status => status::status(database).await,
        CliCommand::Find { query } => find::find(database, query).await,
        CliCommand::Tree => tree::tree(database).await,
        CliCommand::List => list::list(database).await,
        CliCommand::Debug => debug::debug(database).await,
        CliCommand::Add {
            target_path: source_path,
            prefix,
        } => add::add(database, source_path, prefix).await,
        CliCommand::Check => check::check(database).await,

        #[cfg(debug_assertions)]
        CliCommand::Prune => prune::prune(database).await,
    };

    Ok(())
}

//#[cfg(test)]
//mod tests {
//use rand::{prelude::SmallRng, Rng, SeedableRng};
//use tmp::{RandomFill, Tmp};

// fn init_paths_with_rng(rng: &mut impl Rng) -> (Tmp, Tmp) {
// let locked_path = Tmp::random_with_rng(rng);
// let unlocked_path = Tmp::random_with_rng(rng);

// unlocked_path.random_fill(25_000, rng).unwrap();

// (locked_path, unlocked_path)
// }

//#[tokio::test]
//async fn test_integration() {
//// TODO: do something useful here
//let mut db = database::create_in_memory().unwrap();

//let mut rng = SmallRng::seed_from_u64(1);

//let (locked_tmp, unlocked_tmp) = init_paths_with_rng(&mut rng);
//}
//}
