/// A random path generator in /tmp/ that automatically creates and destroys
/// the path
use std::{
    fs::{create_dir, remove_dir_all},
    path::PathBuf,
};

use rand::{distributions::Alphanumeric, Rng};

const TMP_FILENAME_LENGTH: usize = 24;

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
            remove_dir_all(&self.path).unwrap();
        } else {
            panic!(
                "Dropping `TempPath`, but folder {:?} does not exist",
                self.path
            )
        }
    }
}
