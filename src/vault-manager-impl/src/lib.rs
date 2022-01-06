mod actions;
mod commands;

use dotenv::dotenv;

pub async fn start() {
    dotenv().ok();
    pretty_env_logger::init();

    // Initialize database pool
    let database = database::connect_or_create()
        .await
        .expect("Cannot open database");

    // Parse cli arguments and execute requested operation
    commands::execute_command(&database).await;

    database.close().await;
}
