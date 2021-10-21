use std::fs::File;
use std::io::Write;
use rand::Rng;

const PLAINTEXT_FILE: &str = "./crypt_small_file/plaintext";

pub fn generate_random_plaintext_file(length: usize) {
    let mut plaintext_file = File::create(PLAINTEXT_FILE).unwrap();
    let random_bytes: Vec<u8> = (0..length).map(|_| rand::random::<u8>()).collect();

    plaintext_file.write_all(&random_bytes).unwrap();
    plaintext_file.flush().unwrap();
}

pub fn generate_random_key() -> [u8; 32] {
    rand::thread_rng().gen::<[u8; 32]>()
}
