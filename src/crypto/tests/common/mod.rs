#![allow(dead_code)] // So that rust-analyzer stops complaining that common functions are not used

use std::fs::File;
use std::io::Write;
use std::path::Path;

use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};

pub fn generate_random_plaintext_file(file_path: impl AsRef<Path>, length: usize) {
    let mut plaintext_file = File::create(file_path).unwrap();
    let random_bytes: Vec<u8> = (0..length).map(|_| rand::random::<u8>()).collect();

    plaintext_file.write_all(&random_bytes).unwrap();
    plaintext_file.flush().unwrap();
}

pub fn generate_random_plaintext_file_with_rng(
    rng: &mut SmallRng,
    file_path: impl AsRef<Path>,
    length: usize,
) {
    let mut plaintext_file = File::create(file_path).unwrap();
    let mut random_bytes = vec![0; length];

    rng.fill_bytes(&mut random_bytes);

    plaintext_file.write_all(&random_bytes).unwrap();
    plaintext_file.flush().unwrap();
}

pub fn generate_plaintext_with_content(file_path: impl AsRef<Path>, content: impl AsRef<str>) {
    let mut plaintext_file = File::create(file_path).unwrap();

    plaintext_file
        .write_all(content.as_ref().as_bytes())
        .unwrap();
    plaintext_file.flush().unwrap();
}

pub fn generate_seeded_key() -> [u8; 32] {
    let mut rng = SmallRng::seed_from_u64(0);
    let mut bytes = [0u8; 32];

    rng.fill_bytes(&mut bytes);

    bytes
}
