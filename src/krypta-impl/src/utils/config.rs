use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Config {
    pub locked_path: Option<String>,
}

impl Config {
    pub(crate) fn get() -> Self {
        let config_path = config_file_path();

        if config_path.exists() {
            let mut f = File::open(config_path).unwrap();
            let mut s = String::new();
            f.read_to_string(&mut s).unwrap();

            toml::from_str(&s).unwrap()
        } else {
            Config::default()
        }
    }

    pub(crate) fn set(config: Self) {
        let config_path = config_file_path();

        let mut f = File::options()
            .create(true)
            .write(true)
            .open(config_path)
            .unwrap();

        let s = toml::to_string_pretty(&config).unwrap();
        f.write_all(s.as_bytes()).unwrap();
    }

    pub fn get_locked_path() -> PathBuf {
        Config::get()
            .locked_path
            .map(PathBuf::from)
            .expect("Please set the `locked_path` by running `krypta config locked <locked_path>`")
    }
}

/// Get config file path
fn config_file_path() -> PathBuf {
    PathBuf::from("./krypta.toml")
}
