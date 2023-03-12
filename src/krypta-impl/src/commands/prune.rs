use std::{collections::HashSet, fs::remove_file, path::PathBuf};

use database::{connect_or_create, database_file, models, traits::FetchAll, Database};
use utils::ask_yes_or_no;

use crate::utils::config::Config;

pub async fn prune(db: &mut Database) {
    ask_yes_or_no("Are you sure you want to remove everything? This action is irreversible!");

    let db_files = models::File::fetch_all(db)
        .unwrap()
        .into_iter()
        .map(|file| PathBuf::from(file.locked_hash))
        .collect::<HashSet<_>>(); // HashSet in order to remove duplicates

    let locked_path = Config::get_locked_path();

    println!("deleting {} files...", db_files.len());

    for file in db_files {
        let mut full_path = locked_path.clone();
        full_path.push(file);

        match remove_file(&full_path) {
            Ok(_) => (),
            Err(err) => println!("cannot remove file {:?}: {err}", full_path),
        }
    }

    println!("recreating database...");
    remove_file(database_file()).unwrap();

    drop(db);
    let _ = connect_or_create().unwrap();

    println!("all done");
}
