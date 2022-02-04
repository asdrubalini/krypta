use std::{
    io::{stdout, BufWriter, Write},
    path::PathBuf,
    time::Instant,
};

use database::{
    models,
    traits::{Count, Search},
    Database,
};
use fs::PathTree;

pub async fn find(db: &mut Database, query: String) -> anyhow::Result<()> {
    let start = Instant::now();

    let query_result = models::File::search(db, query)?;

    let paths_tree: PathTree = query_result.iter().map(PathBuf::from).collect();
    let paths_ordered = paths_tree.paths_ordered();

    let mut stdout = BufWriter::new(stdout());

    for path in paths_ordered {
        let line = format!("{}\n", path.to_string_lossy());
        stdout.write_all(line.as_bytes())?;
    }

    stdout.flush()?;

    let files_count = models::File::count(db)?;
    println!("Took {:?} for finding {files_count} files", start.elapsed());

    Ok(())
}
