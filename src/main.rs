mod database;
mod storage;
mod tui;

#[tokio::main]
async fn main() {
    let db = database::connect_or_create().await.unwrap();
    let res = database::models::File::search(&db, "feet".to_string())
        .await
        .unwrap();

    println!("{:?}", res);
}
