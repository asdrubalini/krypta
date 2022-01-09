/// A random path generator in /tmp/ that automatically creates and deletes the path
use std::{
    fs::{create_dir, remove_dir_all, File},
    io::Write,
    path::PathBuf,
};

use rand::{distributions::Alphanumeric, Rng};

const TMP_PATH_LENGTH: usize = 16;

pub fn random_string(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect::<String>()
}

#[derive(Debug)]
pub struct Tmp {
    path: PathBuf,
}

impl Default for Tmp {
    fn default() -> Self {
        Self::new()
    }
}

impl Tmp {
    /// Generate a random path in the form of "/tmp/<random chars>/"
    fn generate_random_tmp(folder_length: usize) -> PathBuf {
        let random_name = random_string(folder_length);

        let mut random_tmp = PathBuf::new();
        random_tmp.push("/tmp/");
        random_tmp.push(random_name);

        random_tmp
    }

    pub fn new() -> Self {
        let random_tmp = Self::generate_random_tmp(TMP_PATH_LENGTH);

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

impl Drop for Tmp {
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

pub trait RandomFill {
    /// Random fill with files
    fn random_fill(&self, count: usize, fill_length: usize) -> Vec<PathBuf>;
}

impl RandomFill for Tmp {
    fn random_fill(&self, count: usize, fill_length: usize) -> Vec<PathBuf> {
        let mut current_base = self.path();
        let mut paths = vec![];
        let mut rng = rand::thread_rng();

        for _ in 0..count {
            let mut path = PathBuf::from(&current_base);
            path.push(random_string(TMP_PATH_LENGTH));

            let mut file =
                File::create(&path).unwrap_or_else(|_| panic!("Cannot create {:?}", path));
            let random_bytes: Vec<u8> = (0..fill_length).map(|_| rand::random::<u8>()).collect();

            file.write_all(&random_bytes).unwrap();
            file.flush().unwrap();

            let rand = rng.gen::<f32>();

            if rand > 0.99 {
                current_base = self.path();
            } else if rand > 0.98 {
                let mut base = current_base.clone();
                base.push(random_string(TMP_PATH_LENGTH));
                create_dir(&base).unwrap();
                current_base = base;
            }

            paths.push(path);
        }

        paths
    }
}

#[cfg(test)]
mod tests {
    use super::Tmp;

    #[test]
    fn test_generate_random_tmp_path_length() {
        for length in 1..32 {
            let path = Tmp::generate_random_tmp(length);
            let generated_folder = path.into_iter().last().unwrap();

            assert_eq!(generated_folder.len(), length);
        }
    }

    #[test]
    fn test_temp_path_folder_creation_and_destruction() {
        for _ in 0..256 {
            let path = {
                let tmp = Tmp::new();

                // Make sure that path gets created
                assert!(tmp.path().exists());
                tmp.path()
            };

            // Make sure that path gets destroyed
            assert_eq!(path.exists(), false);
        }
    }
}
