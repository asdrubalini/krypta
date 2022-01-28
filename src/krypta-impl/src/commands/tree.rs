use database::{models, traits::FetchAll, Database};
use fs::PathTree;

pub async fn tree(db: &mut Database) -> anyhow::Result<()> {
    let tree: PathTree = models::File::fetch_all(db)?
        .into_iter()
        .map(|f| f.as_path_buf())
        .collect();

    tree.print_ordered();

    Ok(())
}
