use std::{
    fs::{create_dir_all, remove_dir_all},
    path::Path,
};

use metadata_fs::PathFinder;

mod common;

#[test]
fn test_path_finder() {
    let source_path = Path::new("/tmp/test_dir/path_finder/foo/bar/");
    create_dir_all(source_path).unwrap();

    common::generate_files(source_path, 128, 0);

    let path_finder = PathFinder::with_source_path(source_path);

    for path in path_finder.paths {
        assert!(path.to_string_lossy().to_string().starts_with("file_"));
    }

    remove_dir_all(source_path).unwrap();
}
