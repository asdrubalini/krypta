use crypto::{hash::Blake3Concurrent, traits::ComputeBulk};
use rand::{prelude::SmallRng, SeedableRng};
use tmp::Tmp;

use common::{
    generate_plaintext_with_content, generate_random_plaintext_file_with_rng, BLAKE3_EMPTY_HASH,
    BLAKE3_EXPECTED_HASHES,
};

mod common;

#[test]
fn test_blake3_same_file() {
    let tmp = Tmp::empty();

    let mut paths = vec![];

    // Populate with 8192 empty files
    for i in 0..8192 {
        let mut unlocked_path = tmp.path();
        unlocked_path.push(format!("{}.txt", i));

        generate_plaintext_with_content(&unlocked_path, "".as_bytes());

        paths.push(unlocked_path);
    }

    let concurrent = Blake3Concurrent::try_new(&paths).unwrap();
    let results = concurrent.start_all();

    // Compute hashes
    for i in 0..8192 {
        let mut unlocked_path = tmp.path();
        unlocked_path.push(format!("{}.txt", i));

        let hash = results.get(&unlocked_path).unwrap();
        assert_eq!(hash.to_string(), BLAKE3_EMPTY_HASH);
    }
}

#[test]
fn test_blake3_random_files() {
    let tmp = Tmp::empty();

    let mut rng = SmallRng::seed_from_u64(0);
    let mut generated_paths = vec![];

    // Populate files
    for i in 0..BLAKE3_EXPECTED_HASHES.len() {
        let mut current_path = tmp.path();
        current_path.push(format!("random_{}.txt", i));

        generate_random_plaintext_file_with_rng(&mut rng, &current_path, 2usize.pow(20));
        generated_paths.push(current_path);
    }

    let concurrent = Blake3Concurrent::try_new(&generated_paths).unwrap();
    let results = concurrent.start_all();

    for (i, generated_path) in generated_paths.into_iter().enumerate() {
        let computed_hash = results.get(&generated_path).unwrap().to_string();
        let expected_hash = BLAKE3_EXPECTED_HASHES[i];

        assert_eq!(computed_hash, expected_hash);
    }
}
