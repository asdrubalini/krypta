mod actions;
mod commands;

use dotenv::dotenv;

pub async fn start() {
    dotenv().ok();
    pretty_env_logger::init();

    // Initialize database pool
    let mut database = database::connect_or_create().expect("Cannot open database");

    // Parse cli arguments and execute requested operation
    commands::execute_command(&mut database).await;

    database.close().unwrap();
}
