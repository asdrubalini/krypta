use std::path::{Path, PathBuf};

use crypto::{
    hash::Sha256ConcurrentFileHasher,
    traits::{Computable, ConcurrentComputable},
};

use crate::common::{clean_tests_path, generate_plaintext_with_content, init_test_path};

mod common;

const TESTS_PATH: &str = "./hashes_tests/";

// async fn plaintext_hash(content: &str) -> String {
// generate_plaintext_with_content(PLAINTEXT_FILE, content.to_string());

// let hasher = Sha256FileHasher::try_new(Path::new(PLAINTEXT_FILE)).unwrap();
// let hash = hasher.start().await.unwrap();

// remove_file(PLAINTEXT_FILE).unwrap();

// hash.as_hex()
// }

#[tokio::test]
async fn empty_files() {
    init_test_path(TESTS_PATH);

    let mut paths = Vec::new();

    for i in 0..8192 {
        let mut plaintext_path = PathBuf::from(TESTS_PATH);
        plaintext_path.push(i.to_string());

        generate_plaintext_with_content(plaintext_path.to_str().unwrap(), "".to_string());

        paths.push(plaintext_path);
    }

    let concurrent = Sha256ConcurrentFileHasher::try_new(&paths).unwrap();

    clean_tests_path(TESTS_PATH);
}
