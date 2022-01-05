use crypto::{hash::Sha256ConcurrentFileHasher, traits::ConcurrentCompute};

use crate::common::{generate_plaintext_with_content, TempPath};

mod common;

#[tokio::test]
async fn empty_equal_files() {
    let tmp = TempPath::new();

    let mut paths = vec![];

    for i in 0..8192 {
        let mut plaintext_path = tmp.path();
        plaintext_path.push(format!("{}.txt", i));

        generate_plaintext_with_content(plaintext_path.to_str().unwrap(), "");

        paths.push(plaintext_path);
    }

    let mut concurrent = Sha256ConcurrentFileHasher::try_new(&paths).unwrap();
    let results = concurrent.start_all().await;

    for i in 0..8192 {
        let mut plaintext_path = tmp.path();
        plaintext_path.push(format!("{}.txt", i));

        let hash = results.get(&plaintext_path).unwrap();
        assert_eq!(
            hash.as_hex(),
            // Empty string hash
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
