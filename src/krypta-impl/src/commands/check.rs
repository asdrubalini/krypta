use core::panic;
use std::{collections::HashSet, path::PathBuf};

use database::{models, traits::FetchAll, Database};
use fs::PathFinder;

use crate::utils::config::Config;

pub async fn check(db: &mut Database) {
    let db_files = models::File::fetch_all(db)
        .unwrap()
        .into_iter()
        .map(|file| PathBuf::from(file.locked_hash))
        .collect::<HashSet<_>>(); // HashSet in order to remove duplicates

    let locked_path = Config::get_locked_path();
    let fs_files = PathFinder::from_source_path(locked_path).unwrap().metadatas;

    if db_files.len() != fs_files.len() {
        panic!(
            "consistency error: Database has {} different files, while Fs has {} different files",
            db_files.len(),
            fs_files.len()
        );
    }

    for db_file in &db_files {
        if fs_files.get(db_file).is_none() {
            panic!(
                "consistency error: file with Hash {:?} is in Database but cannot be found in Fs",
                db_file
            );
        }
    }

    for fs_file in fs_files.keys() {
        if db_files.get(fs_file).is_none() {
            panic!(
                "consistency error: file with Hash {:?} is in Fs but cannot be found in Database",
                fs_file
            );
        }
    }

    println!("consistency check: all ok");
}
