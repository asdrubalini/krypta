use std::{collections::HashMap, path::PathBuf};

use database::{models, traits::FetchAll, Database};
use fs::PathTree;

pub async fn tree(db: &mut Database) -> anyhow::Result<()> {
    let files: HashMap<PathBuf, models::File> = models::File::fetch_all(db)?
        .into_iter()
        .map(|file| (PathBuf::from(&file), file))
        .collect();

    let paths_tree: PathTree = files.iter().map(|(path, _file)| path.to_owned()).collect();
    let paths_ordered = paths_tree.paths_ordered();

    for path in paths_ordered {
        let tags = files.get(&path).unwrap().tags(db)?;

        let tags_pretty: String = if tags.is_empty() {
            "(no tags)".to_string()
        } else {
            tags.into_iter()
                .map(|tag| format!("{} ", tag.name))
                .collect()
        };

        println!("{} -> {}", path.to_string_lossy(), tags_pretty);
    }

    Ok(())
}
