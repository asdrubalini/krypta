/// A random path generator in /tmp/ that automatically creates and deletes the path
use std::{
    fs::{create_dir, remove_dir_all, File},
    io::Write,
    path::{Path, PathBuf},
};

use rand::{distributions::Alphanumeric, thread_rng, Rng};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

const TMP_PATH_LENGTH: usize = 16;

pub fn random_string(length: usize, rng: &mut impl Rng) -> String {
    rng.sample_iter(&Alphanumeric)
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
        Self::empty()
    }
}

impl Tmp {
    /// Generate a random path in the form of "/tmp/<random chars>/"
    #[cfg(target_os = "linux")]
    fn generate_random_tmp(folder_length: usize, rng: &mut impl Rng) -> PathBuf {
        let mut random_name = "krypta_".to_string();
        random_name.push_str(&random_string(folder_length, rng));

        let mut random_tmp = PathBuf::new();
        random_tmp.push("/tmp/");
        random_tmp.push(random_name);

        random_tmp
    }

    pub fn empty() -> Self {
        Self::empty_with_rng(&mut thread_rng())
    }

    pub fn empty_with_rng(rng: &mut impl Rng) -> Self {
        let random_tmp = Self::generate_random_tmp(TMP_PATH_LENGTH, rng);

        if PathBuf::from(&random_tmp).exists() {
            panic!("Random tmp path already exists: {:?}", random_tmp);
        }

        create_dir(&random_tmp).unwrap();

        Self { path: random_tmp }
    }

    pub fn with_path(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref().to_path_buf();

        if PathBuf::from(&path).exists() {
            panic!("Random tmp path already exists: {:?}", path);
        }

        create_dir(&path).unwrap();

        Self { path }
    }

    pub fn path(&self) -> PathBuf {
        self.path.clone()
    }

    pub fn to_relative(&self, absolute_path: impl AsRef<Path>) -> PathBuf {
        let absolute_path = absolute_path.as_ref().to_owned();
        absolute_path
            .iter()
            .skip(self.path().iter().count())
            .collect()
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
    fn random_fill(&self, count: usize, rng: &mut impl Rng) -> Vec<PathBuf>;
}

impl RandomFill for Tmp {
    fn random_fill(&self, count: usize, rng: &mut impl Rng) -> Vec<PathBuf> {
        let mut current_base = self.path();

        let paths: Vec<(PathBuf, usize)> = (0..count)
            .into_iter()
            .map(|_| {
                let mut path = PathBuf::from(&current_base);
                path.push(random_string(TMP_PATH_LENGTH, rng));

                let rand = rng.gen::<f32>();

                if rand > 0.99 {
                    current_base = self.path();
                } else if rand > 0.98 {
                    let mut base = current_base.clone();
                    base.push(random_string(TMP_PATH_LENGTH, rng));
                    create_dir(&base).unwrap();
                    current_base = base;
                }

                let fill_len: usize = if rng.gen_bool(0.9) {
                    rng.gen_range(10..8192)
                } else {
                    rng.gen_range(50_000..100_000)
                };

                (path, fill_len)
            })
            .collect();

        paths
            .par_iter()
            .map(|(path, fill_len)| {
                let mut file =
                    File::create(path).unwrap_or_else(|_| panic!("Cannot create {:?}", path));
                let random_bytes: Vec<u8> = (0..*fill_len).map(|_| rand::random::<u8>()).collect();

                file.write_all(&random_bytes).unwrap();
                file.flush().unwrap();
            })
            .count();

        paths.into_iter().map(|p| p.0).collect()
    }
}

#[cfg(test)]
mod tests {
    use rand::{prelude::SmallRng, SeedableRng};

    use super::Tmp;

    #[test]
    fn test_generate_random_tmp_path_length() {
        let mut rng = SmallRng::seed_from_u64(0);

        for length in 1..32 {
            let path = Tmp::generate_random_tmp(length, &mut rng);
            let generated_folder = path.iter().last().unwrap();

            assert_eq!(generated_folder.len() - "krypta_".len(), length);
        }
    }

    #[test]
    fn test_temp_path_folder_creation_and_destruction() {
        for _ in 0..256 {
            let path = {
                let tmp = Tmp::empty();

                // Make sure that path gets created
                assert!(tmp.path().exists());
                tmp.path()
            };

            // Make sure that path gets destroyed
            assert!(!path.exists());
        }
    }
}
