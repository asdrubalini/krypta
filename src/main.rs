mod database;
mod storage;
mod tui;
mod utils;

use dotenv::dotenv;
use pretty_env_logger;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let database = database::connect_or_create()
        .await
        .expect("Cannot open database");
}
