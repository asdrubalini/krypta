use database::{models, traits::FetchAll, Database};
use fs::PathTree;

pub async fn debug(db: &mut Database) {
    let files = models::File::fetch_all(db).unwrap();
    let tree: PathTree = files.into_iter().collect();

    let ciao = tree.directory_structure();

    for dir in ciao {
        println!("{:?}", dir);
    }

    //vfs::KryptaFS::mount("/home/giovanni/krypta/fuse-mount");
}
