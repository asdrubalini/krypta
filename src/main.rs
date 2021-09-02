#![allow(dead_code, unused_variables)]
mod cli;
mod config;
mod database;
mod storage;
mod sync;

use dotenv::dotenv;
use pretty_env_logger;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    // Initialize database pool
    let database = database::connect_or_create()
        .await
        .expect("Cannot open database");

    // Load config from file
    let config = config::Configuration::read_from_file();

    // Parse CLI arguments
    let command = cli::CliCommand::try_parse().unwrap();

    match command {
        cli::CliCommand::Sync { path } => sync::sync_database_from_source_folder(&database, path)
            .await
            .unwrap(),
    };
}
