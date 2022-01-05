use std::fs::{create_dir, remove_dir_all, File};
use std::io::Write;
use std::path::{Path, PathBuf};

use rand::distributions::Alphanumeric;
use rand::rngs::SmallRng;
use rand::{Rng, RngCore, SeedableRng};

const TMP_FILENAME_LENGTH: usize = 24;

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

/// A random path generator in /tmp/ that automatically creates and destroys
/// the path
pub struct TempPath {
    path: PathBuf,
}

impl TempPath {
    fn generate_random_tmp() -> PathBuf {
        let random_name = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(TMP_FILENAME_LENGTH)
            .map(char::from)
            .collect::<String>();

        let mut random_tmp = PathBuf::new();
        random_tmp.push("/tmp/");
        random_tmp.push(random_name);

        random_tmp
    }

    pub fn new() -> Self {
        let random_tmp = Self::generate_random_tmp();

        if PathBuf::from(&random_tmp).exists() {
            panic!("Random tmp path already exists: {:?}", random_tmp);
        }

        create_dir(&random_tmp).unwrap();

        Self { path: random_tmp }
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }
}

impl Drop for TempPath {
    fn drop(&mut self) {
        if self.path.exists() {
            remove_dir_all(&self.path).unwrap()
        }
    }
}
