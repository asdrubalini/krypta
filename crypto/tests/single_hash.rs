use std::{fs::remove_file, path::Path};

use crypto::hash::{single::SingleSha256, traits::SingleHashable};

mod common;

const TESTS_PATH: &str = "./hash_tests/";
const PLAINTEXT_FILE: &str = "./hash_tests/file";

async fn plaintext_hash(content: &str) -> String {
    common::generate_plaintext_with_content(PLAINTEXT_FILE, content.to_string());

    let plaintext_path = Path::new(PLAINTEXT_FILE);
    let hasher = SingleSha256::try_new(plaintext_path).unwrap();
    let hash = hasher.start().await.unwrap();

    remove_file(PLAINTEXT_FILE).unwrap();

    hash.as_hex()
}

#[tokio::test]
async fn small_file() {
    common::init_test_path(TESTS_PATH);

    // Empty hash
    assert_eq!(
        plaintext_hash("").await,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );

    // Short ascii hash
    assert_eq!(
        plaintext_hash("abc").await,
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );

    common::clean_test_path(TESTS_PATH);
}
