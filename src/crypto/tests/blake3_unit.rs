use std::{fs::remove_file, path::Path};

use crypto::{hash::Blake3File, traits::ComputeUnit};
use rand::{prelude::SmallRng, SeedableRng};
use tmp::Tmp;

use common::{
    generate_plaintext_with_content, generate_random_plaintext_file_with_rng, BLAKE3_EMPTY_HASH,
    BLAKE3_EXPECTED_HASHES,
};

mod common;

/// Create file with specified filename and content, write data, compute hash and then
/// remove the file
fn generate_file_with_content_and_blake3_hash(
    content: &str,
    plaintext_file: impl AsRef<Path>,
) -> String {
    generate_plaintext_with_content(plaintext_file.as_ref(), content.as_bytes());

    let hasher = Blake3File::try_new(plaintext_file.as_ref()).unwrap();
    let hash = hasher.start().unwrap();

    remove_file(plaintext_file.as_ref()).unwrap();

    hash.to_string()
}

#[test]
fn test_blake3_small_ascii_file() {
    let tmp = Tmp::random();

    let mut plaintext_file = tmp.base_path();
    plaintext_file.push("small_file.txt");

    // Empty string hash
    assert_eq!(
        generate_file_with_content_and_blake3_hash("", &plaintext_file),
        BLAKE3_EMPTY_HASH
    );

    // Short ascii hash
    assert_eq!(
        generate_file_with_content_and_blake3_hash("abc", &plaintext_file),
        "6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85"
    );
}

fn generate_file_with_rng_and_blake3_hash(
    rng: &mut SmallRng,
    length: usize,
    plaintext_file: impl AsRef<Path>,
) -> String {
    let plaintext_file = plaintext_file.as_ref();
    generate_random_plaintext_file_with_rng(rng, plaintext_file, length);

    let hasher = Blake3File::try_new(plaintext_file).unwrap();
    let hash = hasher.start().unwrap();

    hash.to_string()
}

#[test]
fn test_blake3_random_file() {
    let tmp = Tmp::random();

    let mut generated_random_file = tmp.base_path();
    generated_random_file.push("big_file.txt");
    let generated_random_file = generated_random_file.as_path();

    let mut rng = SmallRng::seed_from_u64(0);

    for expected_hash in BLAKE3_EXPECTED_HASHES {
        let hash =
            generate_file_with_rng_and_blake3_hash(&mut rng, 2usize.pow(20), generated_random_file);
        remove_file(generated_random_file).unwrap();
        assert_eq!(hash, expected_hash);
    }
}
