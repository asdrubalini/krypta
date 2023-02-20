/// A random path generator in /tmp/ that automatically creates and deletes the path
use std::{
    fs::{create_dir, remove_dir_all, File},
    io::Write,
    path::{Path, PathBuf}, env,
};

use rand::{thread_rng, Rng};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use utils::RandomString;

const TMP_PATH_LENGTH: usize = 16;

#[derive(Debug)]
pub struct Tmp {
    base_path: PathBuf,
}

impl Tmp {
    /// Create new Tmp with random path based on thread rng
    pub fn random() -> Self {
        Self::random_with_rng(&mut thread_rng())
    }

    /// Create new Tmp with random path based on provided rng
    pub fn random_with_rng(rng: &mut impl Rng) -> Self {
        let random_path = Self::generate_random_tmp(TMP_PATH_LENGTH, rng);

        if PathBuf::from(&random_path).exists() {
            panic!("Random tmp path already exists: {:?}", random_path);
        }

        create_dir(&random_path).unwrap();

        Self {
            base_path: random_path,
        }
    }

    /// Get the Tmp's base path
    pub fn base_path(&self) -> PathBuf {
        self.base_path.clone()
    }

    /// Convert an absolute path to a relative one stripping off the /tmp/{name}/ bits
    pub fn to_relative(&self, absolute_path: impl AsRef<Path>) -> PathBuf {
        let absolute_path = absolute_path.as_ref().to_owned();
        absolute_path
            .iter()
            .skip(self.base_path().iter().count())
            .collect()
    }

    pub fn prefix_len() -> usize {
        Self::prefix().iter().count()
    }

    #[cfg(target_family = "unix")]
    fn prefix() -> PathBuf {
        if let Some(prefix) = env::var_os("KRYPTA_TMP_PREFIX") {
            PathBuf::from(prefix)
        } else {
            PathBuf::from("/tmp/")
        }
    }

    /// Generate a random tmp path in the form of /tmp/{name}/
    fn generate_random_tmp(folder_length: usize, rng: &mut impl Rng) -> PathBuf {
        let mut random_name = "krypta_".to_string();
        random_name.push_str(&RandomString::alphanum_with_rng(rng, folder_length));

        let mut random_tmp = PathBuf::new();
        random_tmp.push(Self::prefix());
        random_tmp.push(random_name);

        random_tmp
    }
}

impl Drop for Tmp {
    /// Cleanup the Tmp patg
    fn drop(&mut self) {
        if env::var_os("KRYPTA_TMP_PERSIST").is_some() {
            return;
        }

        if self.base_path.exists() {
            remove_dir_all(&self.base_path).unwrap();
        } else {
            panic!(
                "Dropping `TempPath`, but folder {:?} does not exist",
                self.base_path
            )
        }
    }
}

pub trait RandomFill {
    /// Random fill with files
    fn random_fill(&self, count: usize, rng: &mut impl Rng)
        -> Result<Vec<PathBuf>, std::io::Error>;
}

impl RandomFill for Tmp {
    fn random_fill(
        &self,
        count: usize,
        rng: &mut impl Rng,
    ) -> Result<Vec<PathBuf>, std::io::Error> {
        let mut current_base = self.base_path();

        let paths: Result<Vec<(PathBuf, usize)>, std::io::Error> = (0..count)
            .into_iter()
            .map(|_| {
                let mut path = PathBuf::from(&current_base);
                path.push(RandomString::alphanum_with_rng(rng, TMP_PATH_LENGTH));

                let rand = rng.gen::<f32>();

                if rand > 0.99 {
                    current_base = self.base_path();
                } else if rand > 0.98 {
                    let mut base = current_base.clone();
                    base.push(RandomString::alphanum_with_rng(rng, TMP_PATH_LENGTH));
                    create_dir(&base)?;
                    current_base = base;
                }

                let fill_len: usize = if rng.gen_bool(0.9) {
                    rng.gen_range(10..8192)
                } else {
                    rng.gen_range(50_000..100_000)
                };

                Ok((path, fill_len))
            })
            .collect();

        let paths = paths?;

        paths
            .par_iter()
            .map(|(path, fill_len)| {
                let mut file =
                    File::create(path).unwrap_or_else(|_| panic!("Cannot create {:?}", path));
                let random_bytes: Vec<u8> = (0..*fill_len).map(|_| rand::random::<u8>()).collect();

                file.write_all(&random_bytes)?;
                file.flush().unwrap();
                Ok(())
            })
            .collect::<Result<(), std::io::Error>>()?;

        Ok(paths.into_iter().map(|p| p.0).collect())
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
}
