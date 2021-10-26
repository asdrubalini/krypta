use std::fs::{create_dir, remove_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

pub fn generate_random_plaintext_file(file_path: &str, length: usize) {
    let mut plaintext_file = File::create(file_path).unwrap();
    let random_bytes: Vec<u8> = (0..length).map(|_| rand::random::<u8>()).collect();

    plaintext_file.write_all(&random_bytes).unwrap();
    plaintext_file.flush().unwrap();
}

pub fn generate_random_plaintext_file_with_rng(rng: &mut SmallRng, file_path: &str, length: usize) {
    let mut plaintext_file = File::create(file_path).unwrap();
    let mut random_bytes = vec![0; length];

    rng.fill_bytes(&mut random_bytes);

    plaintext_file.write_all(&random_bytes).unwrap();
    plaintext_file.flush().unwrap();
}

pub fn generate_plaintext_with_content(file_path: &str, content: String) {
    let mut plaintext_file = File::create(file_path).unwrap();

    plaintext_file.write_all(content.as_bytes()).unwrap();
    plaintext_file.flush().unwrap();
}

pub fn generate_seeded_key() -> [u8; 32] {
    let mut rng = SmallRng::seed_from_u64(0);
    let mut bytes = [0u8; 32];

    rng.fill_bytes(&mut bytes);

    bytes
}

pub fn init_test_path(tests_path: &str) {
    if PathBuf::from(tests_path).exists() {
        remove_dir_all(tests_path).unwrap();
    }

    create_dir(tests_path).unwrap();
}

pub fn clean_tests_path(tests_path: &str) {
    remove_dir_all(tests_path).unwrap();
}
