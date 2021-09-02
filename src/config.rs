use std::{env, fs::read_to_string, sync::Arc};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub struct Endpoint {
    pub endpoint_url: String,
    pub access_key: String,
    pub region: String,
    pub secret_key: String,
    pub bucket_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Configuration {
    pub source_path: String,
    pub encrypted_path: String,
    pub endpoint: Endpoint,
}

impl Configuration {
    pub fn read_from_file() -> Arc<Self> {
        let config_path = env::var("CONFIG_FILE").expect("Cannot read CONFIG_FILE env");
        let config_raw = read_to_string(config_path).expect("Cannot read config to string");

        let config: Configuration =
            toml::from_str(&config_raw).expect("Cannot parse toml config file");

        Arc::new(config)
    }
}
