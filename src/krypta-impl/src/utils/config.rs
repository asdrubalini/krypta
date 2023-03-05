use std::{
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    sync::RwLock,
};

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

static CONFIG: Lazy<RwLock<Config>> = Lazy::new(|| RwLock::from(Config::read_from_disk()));

#[derive(Default, Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub locked_path: Option<String>,
}

impl Config {
    fn read_from_disk() -> Self {
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

    pub(crate) fn get() -> Self {
        // read from global config
        CONFIG.read().unwrap().to_owned()
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

        // Update global config
        let mut c = CONFIG.write().unwrap();
        *c = config;
    }

    pub fn get_locked_path() -> PathBuf {
        let p = Config::get()
            .locked_path
            .map(PathBuf::from)
            .expect("Please set the `locked_path` by running `krypta config locked <locked_path>`");

        File::open(&p).unwrap_or_else(|_| {
            panic!(
                "cannot open `locked_path` {}: No such file or directory",
                p.to_string_lossy().to_string()
            )
        });

        p
    }
}

/// Get config file path
fn config_file_path() -> PathBuf {
    PathBuf::from("./krypta.toml")
}
