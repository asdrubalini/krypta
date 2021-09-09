use std::{
    fs::{create_dir_all, remove_dir_all},
    path::Path,
};

use metadata_fs::{MetadataCollection, PathFinder};

mod common;

#[tokio::test]
async fn test_metadata_size() {
    let source_path = Path::new("/tmp/test_dir/metadata/foo/bar/");
    create_dir_all(source_path).unwrap();

    common::generate_files(source_path, 128, 100);

    let path_finder = PathFinder::with_source_path(source_path);
    let metadatas = MetadataCollection::from_path_finder(path_finder).await;

    for metadata in metadatas.metadatas {
        assert_eq!(metadata.size, Some(100));
    }

    remove_dir_all(source_path).unwrap();
}
