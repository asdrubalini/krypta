mod database;
mod storage;
mod tui;

use dotenv::dotenv;
use pretty_env_logger;

#[tokio::main]
async fn main() {
    dotenv().ok();
    pretty_env_logger::init();

    let db = database::connect_or_create().await.unwrap();
    let res = database::models::File::search(&db, "feet".to_string())
        .await
        .unwrap();

    println!("{:?}", res);
}
