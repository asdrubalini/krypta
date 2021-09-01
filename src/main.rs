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

    database::models::File::insert(
        &database,
        "feet video",
        "feet/pornhub/nice_feet.mp4",
        &false,
        "placeholder",
        "placeholder",
    )
    .await
    .expect("Cannot insert stuff into the database");

    let records = database::models::File::search(&database, "feet")
        .await
        .expect("Cannot search");

    println!("{:#?}", records);
}
