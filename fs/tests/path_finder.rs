use std::{
    fs::{create_dir_all, remove_dir_all},
    path::Path,
};

use fs::PathFinder;

mod common;

#[test]
fn test_path_finder() {
    let source_path = Path::new("./path_finder_tests/");
    create_dir_all(source_path).unwrap();

    common::generate_files(source_path, 128, 0);

    let path_finder = PathFinder::from_source_path(source_path).unwrap();

    for (path, metadata) in path_finder.metadatas {
        assert!(path.to_string_lossy().to_string().starts_with("file_"));
        assert_eq!(metadata.len(), 0);
    }

    remove_dir_all(source_path).unwrap();
}
