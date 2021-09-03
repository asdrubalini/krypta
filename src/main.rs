#![allow(dead_code, unused_variables)]
mod commands;
mod config;
mod database;
mod storage;
mod sync;

use dotenv::dotenv;

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
    let command = commands::CliCommand::try_parse()
        .unwrap()
        .execute(config.clone(), &database)
        .await;

    database.close().await;
}
