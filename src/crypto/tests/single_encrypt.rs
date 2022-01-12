use std::{
    fs::{remove_file, File},
    io::Read,
    path::Path,
};

use common::generate_plaintext_with_content;
use crypto::{
    crypt::{
        generate_random_secure_key_nonce_pair, FileDecryptUnit, FileEncryptUnit, AEAD_KEY_SIZE,
        AEAD_NONCE_SIZE,
    },
    traits::ComputeUnit,
};
use file_diff::diff;
use rand::{prelude::SmallRng, SeedableRng};
use tmp::Tmp;

use crate::common::{generate_random_plaintext_file_with_rng, generate_seeded_key};

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

    let mut unlocked_path = file_path.clone();
    unlocked_path.push(PLAINTEXT_FILE);

    let mut locked_path = file_path.clone();
    locked_path.push(ENCRYPTED_FILE);

    let mut recovered_path = file_path.clone();
    recovered_path.push(RECOVERED_FILE);

    let encryptor = FileEncryptUnit::try_new(&unlocked_path, &locked_path, key, nonce).unwrap();
    encryptor.start().unwrap();

    let decryptor = FileDecryptUnit::try_new(&locked_path, &recovered_path, key, nonce).unwrap();
    decryptor.start().unwrap();

    // Make sure that plaintext and recovered files are the same
    assert!(diff(
        &unlocked_path.as_os_str().to_string_lossy().to_string(),
        &recovered_path.as_os_str().to_string_lossy().to_string()
    ));

    remove_file(unlocked_path).unwrap();
    remove_file(locked_path).unwrap();
    remove_file(recovered_path).unwrap();
}

#[test]
fn small_file_seeded_key() {
    let tmp = Tmp::new();
    let mut rng = SmallRng::seed_from_u64(0);

    let tests_file_size = [
        1, 2, 3, 8, 9, 200, 256, 512, 893, 1024, 8192, 100_000, 250_000, 1_000_000,
    ];

    for length in tests_file_size {
        println!("Testing with plaintext len = {}", length);

        let mut plaintext_file = tmp.path();
        plaintext_file.push(PLAINTEXT_FILE);

        generate_random_plaintext_file_with_rng(&mut rng, plaintext_file, length);

        let (key, nonce) = generate_seeded_key();
        encrypt_decrypt_with_key(tmp.path(), key, nonce);
    }
}

#[test]
fn test_locked_file_is_different_than_unlocked() {
    let tmp = Tmp::new();

    let mut blank_path = tmp.path();
    blank_path.push("blank.txt");

    let mut locked_path = tmp.path();
    locked_path.push("out.txt");

    let plaintext_content = (0..256).into_iter().map(|_| 0x0).collect::<Vec<u8>>();

    generate_plaintext_with_content(&blank_path, plaintext_content.as_slice());

    let (key, nonce) = generate_random_secure_key_nonce_pair();
    let crypto = FileEncryptUnit::try_new(&blank_path, &locked_path, key, nonce).unwrap();
    crypto.start().unwrap();

    let mut encrypted_file = File::open(locked_path).unwrap();
    let mut encrypted_contents = vec![];
    encrypted_file.read_to_end(&mut encrypted_contents).unwrap();

    assert_ne!(plaintext_content, encrypted_contents);
}
