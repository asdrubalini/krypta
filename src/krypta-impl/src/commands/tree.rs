use std::path::PathBuf;

use database::{models, traits::FetchAll, Database};
use fs::PathTree;

pub async fn tree(db: &Database) {
    let files = models::File::fetch_all(db).unwrap();

    let paths: PathTree = files
        .into_iter()
        .map(|f| f.path)
        .map(PathBuf::from)
        .collect();

    for path in paths.paths_ordered() {
        println!("{}", path.to_string_lossy());
    }
}
