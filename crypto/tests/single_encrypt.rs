use std::{fs::remove_file, path::Path};

use crypto::{
    crypt::{FileDecryptor, FileEncryptor},
    traits::Compute,
};
use file_diff::diff;

use crate::common::{generate_random_plaintext_file, generate_seeded_key, TempPath};

mod common;

const PLAINTEXT_FILE: &str = "plaintext";
const ENCRYPTED_FILE: &str = "encrypted";
const RECOVERED_FILE: &str = "recovered";

async fn encrypt_decrypt_with_key(tmp_path: impl AsRef<Path>, key: &[u8; 32]) {
    let file_path = tmp_path.as_ref().to_path_buf();

    let mut plaintext_path = file_path.clone();
    plaintext_path.push(PLAINTEXT_FILE);

    let mut encrypted_path = file_path.clone();
    encrypted_path.push(ENCRYPTED_FILE);

    let mut recovered_path = file_path.clone();
    recovered_path.push(RECOVERED_FILE);

    let encryptor = FileEncryptor::try_new(&plaintext_path, &encrypted_path, key).unwrap();

    encryptor.start().await.unwrap();

    let decryptor = FileDecryptor::try_new(&encrypted_path, &recovered_path, key).unwrap();

    decryptor.start().await.unwrap();

    // Make sure that plaintext and recovered files are the same
    assert!(diff(
        &plaintext_path.as_os_str().to_string_lossy().to_string(),
        &recovered_path.as_os_str().to_string_lossy().to_string()
    ));

    remove_file(plaintext_path).unwrap();
    remove_file(encrypted_path).unwrap();
    remove_file(recovered_path).unwrap();
}

#[tokio::test]
async fn small_file_seeded_key() {
    let tmp = TempPath::new();

    let tests_file_size = [
        0, 1, 2, 3, 8, 9, 200, 256, 512, 893, 1024, 8192, 100_000, 250_000, 1_000_000,
    ];

    for length in tests_file_size {
        println!("Testing with plaintext len = {}", length);

        let mut plaintext_file = tmp.path();
        plaintext_file.push(PLAINTEXT_FILE);

        generate_random_plaintext_file(plaintext_file, length);
        encrypt_decrypt_with_key(tmp.path(), &generate_seeded_key()).await;
    }
}
