use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub locked_path: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self { locked_path: None }
    }
}

impl Config {
    pub fn read() -> Self {
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

    pub fn write(config: Self) {
        let config_path = config_file_path();

        let mut f = File::options()
            .create(true)
            .write(true)
            .open(config_path)
            .unwrap();

        let s = toml::to_string_pretty(&config).unwrap();
        f.write_all(s.as_bytes()).unwrap();
    }
}

/// Get config file path
fn config_file_path() -> PathBuf {
    PathBuf::from("./krypta.toml")
}
