use std::{fs::remove_file, path::Path};

use file_diff::diff;

use crate::common::{
    clean_test_path, generate_random_key, generate_random_plaintext_file, init_test_path,
};

mod common;

const TESTS_PATH: &str = "./crypt_small_file_tests/";
const PLAINTEXT_FILE: &str = "./crypt_small_file_tests/plaintext";
const ENCRYPTED_FILE: &str = "./crypt_small_file_tests/encrypted";
const PLAINTEXT_RECOVERED_FILE: &str = "./crypt_small_file_tests/plaintext-recovered";

// async fn encrypt_decrypt_with_key(key: &[u8; 32]) {
// let plaintext_path = Path::new(PLAINTEXT_FILE);
// let encrypted_path = Path::new(ENCRYPTED_FILE);
// let plaintext_recovered_path = Path::new(PLAINTEXT_RECOVERED_FILE);

// let encryptor = SingleFileEncryptor::try_new(plaintext_path, encrypted_path, key).unwrap();

// encryptor.start().await.unwrap();

// let decryptor =
// SingleFileDecryptor::try_new(encrypted_path, plaintext_recovered_path, key).unwrap();

// decryptor.start().await.unwrap();

// assert!(diff(PLAINTEXT_FILE, PLAINTEXT_RECOVERED_FILE));

// remove_file(PLAINTEXT_FILE).unwrap();
// remove_file(ENCRYPTED_FILE).unwrap();
// remove_file(PLAINTEXT_RECOVERED_FILE).unwrap();
// }

// #[tokio::test]
// async fn small_file_zero_key() {
// init_test_path(TESTS_PATH);

// let tests_file_size = vec![
// 0, 1, 2, 3, 8, 9, 200, 256, 512, 893, 1024, 8192, 100_000, 250_000, 1_000_000,
// ];

// for length in tests_file_size {
// println!("Testing with plaintext len = {}", length);
// generate_random_plaintext_file(PLAINTEXT_FILE, length);
// encrypt_decrypt_with_key(&generate_random_key()).await;
// }

// clean_test_path(TESTS_PATH);
// }
