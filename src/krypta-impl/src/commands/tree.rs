use std::path::PathBuf;

use database::{models, traits::FetchAll, Database};
use fs::PathTree;

pub async fn tree(db: &mut Database) -> anyhow::Result<()> {
    let tree: PathTree = models::File::fetch_all(db)?
        .iter()
        .map(PathBuf::from)
        .collect();

    tree.print_ordered_pretty();

    Ok(())
}
