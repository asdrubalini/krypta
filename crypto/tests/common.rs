use rand::Rng;
use std::fs::{create_dir, remove_dir_all, File};
use std::io::Write;

pub fn generate_random_plaintext_file(file_path: &str, length: usize) {
    let mut plaintext_file = File::create(file_path).unwrap();
    let random_bytes: Vec<u8> = (0..length).map(|_| rand::random::<u8>()).collect();

    plaintext_file.write_all(&random_bytes).unwrap();
    plaintext_file.flush().unwrap();
}

pub fn generate_plaintext_with_content(file_path: &str, content: String) {
    let mut plaintext_file = File::create(file_path).unwrap();

    plaintext_file.write_all(content.as_bytes()).unwrap();
    plaintext_file.flush().unwrap();
}

pub fn generate_random_key() -> [u8; 32] {
    rand::thread_rng().gen::<[u8; 32]>()
}

pub fn init_test_path(tests_path: &str) {
    create_dir(tests_path).unwrap();
}

pub fn clean_test_path(tests_path: &str) {
    remove_dir_all(tests_path).unwrap();
}
