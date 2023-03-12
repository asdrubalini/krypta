use std::{collections::HashMap, path::PathBuf};

use database::{
    models,
    traits::{FetchAll, Get},
    Database,
};
use fs::PathFinder;

use crate::utils::config::Config;

pub async fn check(db: &mut Database) {
    let db_files = models::File::fetch_all(db)
        .unwrap()
        .into_iter()
        .map(|file| (PathBuf::from(file.locked_hash), file.id.unwrap()))
        .collect::<HashMap<_, _>>(); // HashMap in order to remove duplicates

    let locked_path = Config::get_locked_path();
    let fs_files = PathFinder::from_source_path(locked_path).unwrap().metadatas;

    let mut errors_count = 0;

    if db_files.len() != fs_files.len() {
        println!(
            "consistency error: Database has {} different files, while Fs has {} different files",
            db_files.len(),
            fs_files.len()
        );
    }

    for (db_file, id) in &db_files {
        if fs_files.get(db_file).is_none() {
            let file_record = models::File::get(db, *id).unwrap().unwrap();

            println!(
                "consistency error: file with Hash {:?} is in Database but cannot be found in Fs\nthe file is: {:#?}",
                db_file, file_record
            );

            errors_count += 1;
        }
    }

    for fs_file in fs_files.keys() {
        if db_files.get(fs_file).is_none() {
            println!(
                "consistency error: file with Hash {:?} is in Fs but cannot be found in Database",
                fs_file
            );

            errors_count += 1;
        }
    }

    if errors_count == 0 {
        println!("consistency check: all ok");
    } else {
        println!("found a total of {errors_count} errors");
    }
}
