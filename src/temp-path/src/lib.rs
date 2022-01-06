/// A random path generator in /tmp/ that automatically creates and destroys
/// the path
use std::{
    fs::{create_dir, remove_dir_all},
    path::PathBuf,
};

use rand::{distributions::Alphanumeric, Rng};

const TMP_FILENAME_LENGTH: usize = 24;

#[derive(Debug)]
pub struct TempPath {
    path: PathBuf,
}

impl TempPath {
    /// Generate a random path in the form of "/tmp/<random chars>/"
    fn generate_random_tmp(folder_lenght: usize) -> PathBuf {
        let random_name = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(folder_lenght)
            .map(char::from)
            .collect::<String>();

        let mut random_tmp = PathBuf::new();
        random_tmp.push("/tmp/");
        random_tmp.push(random_name);

        random_tmp
    }

    pub fn new() -> Self {
        let random_tmp = Self::generate_random_tmp(TMP_FILENAME_LENGTH);

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

#[cfg(test)]
mod tests {
    use super::TempPath;

    #[test]
    fn test_generate_random_tmp_path_lenght() {
        for lenght in 1..32 {
            let path = TempPath::generate_random_tmp(lenght);
            let generated_folder = path.into_iter().last().unwrap();

            assert_eq!(generated_folder.len(), lenght);
        }
    }

    #[test]
    fn test_temp_path_folder_creation_and_destruction() {
        for _ in 0..256 {
            let path = {
                let tmp = TempPath::new();

                // Make sure that path gets created
                assert!(tmp.path().exists());
                tmp.path()
            };

            // Make sure that path gets destroyed
            assert_eq!(path.exists(), false);
        }
    }
}
