use std::path::PathBuf;

use crypto::{hash::Sha256ConcurrentFileHasher, traits::ConcurrentComputable};

use crate::common::{clean_tests_path, generate_plaintext_with_content, init_test_path};

mod common;

const TESTS_PATH: &str = "./hashes_tests/";

#[tokio::test]
async fn empty_equal_files() {
    init_test_path(TESTS_PATH);

    let mut paths = Vec::new();

    for i in 0..8192 {
        let mut plaintext_path = PathBuf::from(TESTS_PATH);
        plaintext_path.push(i.to_string());

        generate_plaintext_with_content(plaintext_path.to_str().unwrap(), "".to_string());

        paths.push(plaintext_path);
    }

    let mut concurrent = Sha256ConcurrentFileHasher::try_new(&paths).unwrap();
    let results = concurrent.start_all().await;

    for result in results {
        assert_eq!(
            result.as_hex(),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    clean_tests_path(TESTS_PATH);
}
