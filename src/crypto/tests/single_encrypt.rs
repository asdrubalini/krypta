use std::{fs::remove_file, path::Path};

use crypto::{
    crypt::{FileDecryptUnit, FileEncryptUnit},
    traits::ComputeUnit,
    AEAD_KEY_SIZE, AEAD_NONCE_SIZE,
};
use file_diff::diff;
use tmp::Tmp;

use crate::common::{generate_random_plaintext_file, generate_seeded_key};

mod common;

const PLAINTEXT_FILE: &str = "plaintext";
const ENCRYPTED_FILE: &str = "encrypted";
const RECOVERED_FILE: &str = "recovered";

fn encrypt_decrypt_with_key(
    tmp_path: impl AsRef<Path>,
    key: [u8; AEAD_KEY_SIZE],
    nonce: [u8; AEAD_NONCE_SIZE],
) {
    let file_path = tmp_path.as_ref().to_path_buf();

    let mut plaintext_path = file_path.clone();
    plaintext_path.push(PLAINTEXT_FILE);

    let mut encrypted_path = file_path.clone();
    encrypted_path.push(ENCRYPTED_FILE);

    let mut recovered_path = file_path.clone();
    recovered_path.push(RECOVERED_FILE);

    let encryptor = FileEncryptUnit::try_new(&plaintext_path, &encrypted_path, key, nonce).unwrap();
    encryptor.start().unwrap();

    let decryptor = FileDecryptUnit::try_new(&encrypted_path, &recovered_path, key, nonce).unwrap();
    decryptor.start().unwrap();

    // Make sure that plaintext and recovered files are the same
    assert!(diff(
        &plaintext_path.as_os_str().to_string_lossy().to_string(),
        &recovered_path.as_os_str().to_string_lossy().to_string()
    ));

    remove_file(plaintext_path).unwrap();
    remove_file(encrypted_path).unwrap();
    remove_file(recovered_path).unwrap();
}

#[test]
fn small_file_seeded_key() {
    let tmp = Tmp::new();

    let tests_file_size = [
        1, 2, 3, 8, 9, 200, 256, 512, 893, 1024, 8192, 100_000, 250_000, 1_000_000,
    ];

    for length in tests_file_size {
        println!("Testing with plaintext len = {}", length);

        let mut plaintext_file = tmp.path();
        plaintext_file.push(PLAINTEXT_FILE);

        generate_random_plaintext_file(plaintext_file, length);

        let (key, nonce) = generate_seeded_key();
        encrypt_decrypt_with_key(tmp.path(), key, nonce);
    }
}
