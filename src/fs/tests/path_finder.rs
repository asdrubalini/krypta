use fs::PathFinder;
use tmp::{RandomFill, Tmp};

#[test]
fn test_path_finder() {
    for files_count in 100..150 {
        for files_len in 10..15 {
            println!("files_count: {} files_len: {}", files_count, files_len);

            let tmp = Tmp::new();
            tmp.random_fill(files_count, files_len);

            let path_finder = PathFinder::from_source_path(tmp.path()).unwrap();

            assert_eq!(path_finder.metadatas.len(), files_count);

            for (_path, metadata) in path_finder.metadatas {
                assert_eq!(metadata.len(), files_len as u64);
            }
        }
    }
}
