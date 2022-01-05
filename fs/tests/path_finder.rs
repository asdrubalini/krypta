use fs::PathFinder;
use temp_path::TempPath;

mod common;

#[test]
fn test_path_finder() {
    let tmp = TempPath::new();

    common::generate_files(tmp.path(), 128, 0);

    let path_finder = PathFinder::from_source_path(tmp.path()).unwrap();

    for (path, metadata) in path_finder.metadatas {
        assert!(path.to_string_lossy().to_string().starts_with("file_"));
        assert_eq!(metadata.len(), 0);
    }
}
