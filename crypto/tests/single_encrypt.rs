use std::{
    fs::{create_dir, remove_dir, remove_file, File},
    io::Write,
    path::Path,
};

use crypto::single::{decryption::SingleFileDecryptor, encryption::SingleFileEncryptor};
use file_diff::diff;
use rand::Rng;

const TESTS_PATH: &str = "./crypt_small_file/";
const PLAINTEXT_FILE: &str = "./crypt_small_file/plaintext";
const ENCRYPTED_FILE: &str = "./crypt_small_file/encrypted";
const PLAINTEXT_RECOVERED_FILE: &str = "./crypt_small_file/plaintext-recovered";

async fn encrypt_decrypt_with_key(key: &[u8; 32]) {
    let plaintext_path = Path::new(PLAINTEXT_FILE);
    let encrypted_path = Path::new(ENCRYPTED_FILE);
    let plaintext_recovered_path = Path::new(PLAINTEXT_RECOVERED_FILE);

    let encryptor = SingleFileEncryptor::new(plaintext_path, encrypted_path, key).unwrap();

    encryptor.try_encrypt().await.unwrap();

    let decryptor =
        SingleFileDecryptor::new(encrypted_path, plaintext_recovered_path, key).unwrap();

    decryptor.try_decrypt().await.unwrap();

    assert!(diff(PLAINTEXT_FILE, PLAINTEXT_RECOVERED_FILE));

    remove_file(PLAINTEXT_FILE).unwrap();
    remove_file(ENCRYPTED_FILE).unwrap();
    remove_file(PLAINTEXT_RECOVERED_FILE).unwrap();
}

fn generate_random_plaintext_file(length: usize) {
    let mut plaintext_file = File::create(PLAINTEXT_FILE).unwrap();
    let random_bytes: Vec<u8> = (0..length).map(|_| rand::random::<u8>()).collect();

    plaintext_file.write_all(&random_bytes).unwrap();
    plaintext_file.flush().unwrap();
}

fn generate_random_key() -> [u8; 32] {
    rand::thread_rng().gen::<[u8; 32]>()
}

#[tokio::test]
async fn small_file_zero_key() {
    create_dir(TESTS_PATH).unwrap();

    let tests_file_size = vec![
        0,
        1,
        2,
        3,
        8,
        9,
        200,
        256,
        512,
        893,
        1024,
        8192,
        100_000,
        250_000,
        1_000_000
    ];

    for length in tests_file_size {
        println!("Testing with plaintext len = {}", length);
        generate_random_plaintext_file(length);
        encrypt_decrypt_with_key(&generate_random_key()).await;
    }

    remove_dir(TESTS_PATH).unwrap();
}
