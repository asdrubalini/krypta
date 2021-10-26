use std::{fs::remove_file, path::Path};

use crypto::{hash::Sha256FileHasher, traits::Computable};

use crate::common::{clean_tests_path, generate_plaintext_with_content, init_test_path};

mod common;

const TESTS_PATH: &str = "./hash_tests/";
const PLAINTEXT_FILE: &str = "./hash_tests/file";

async fn plaintext_hash(content: &str) -> String {
    generate_plaintext_with_content(PLAINTEXT_FILE, content.to_string());

    let hasher = Sha256FileHasher::try_new(Path::new(PLAINTEXT_FILE)).unwrap();
    let hash = hasher.start().await.unwrap();

    remove_file(PLAINTEXT_FILE).unwrap();

    hash.as_hex()
}

#[tokio::test]
async fn small_file() {
    init_test_path(TESTS_PATH);

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

    clean_tests_path(TESTS_PATH);
}
