use database::{models, traits::FetchAll, Database};
use fs::PathTree;

pub async fn tree(db: &Database) {
    let files = models::File::fetch_all(db).unwrap();
    let tree: PathTree = files.into_iter().collect();

    for path in tree.paths_ordered() {
        println!("{}", path.to_string_lossy());
    }
}
