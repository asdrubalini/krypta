use crate::database::Database;
use crate::sync::sync_database_from_source_folder;

pub async fn execute(database: &Database, path: String) {
    let result = sync_database_from_source_folder(database, path)
        .await
        .expect("Fatal while executing command");

    println!("{:#?}", result);
}
