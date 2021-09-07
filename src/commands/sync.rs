use std::path::PathBuf;

use crate::actions::sync::sync_database_from_source_path;
use crate::database::Database;

pub async fn execute(database: &Database, path: String) {
    let source_path = PathBuf::from(path);
    let result = sync_database_from_source_path(database, &source_path)
        .await
        .expect("Fatal while executing command");

    println!("{:#?}", result);
}
