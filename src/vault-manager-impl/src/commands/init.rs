use std::{env, path::Path};

use log::info;

use crate::{actions::sync::sync_database_from_source_path, database::Database};

pub async fn execute(database: &Database, path: Option<impl AsRef<Path>>) {
    // Default is pwd, fallback to it if path is None
    let source_path = path
        .map(|p| p.as_ref().to_path_buf())
        .unwrap_or_else(|| env::current_dir().unwrap());

    // On init the database is empty, so sync::sync_database_from_source_path acts
    // like initializing database
    let report = sync_database_from_source_path(database, &source_path)
        .await
        .unwrap();

    info!("Done, report: {:?}", report);
}