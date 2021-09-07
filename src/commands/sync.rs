use crate::actions::sync::sync_database_from_source_folder;
use crate::database::Database;

pub async fn execute(database: &Database, path: String) {
    let result = sync_database_from_source_folder(database, path)
        .await
        .expect("Fatal while executing command");

    println!("{:#?}", result);
}
