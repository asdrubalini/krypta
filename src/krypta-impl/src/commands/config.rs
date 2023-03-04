use std::path::PathBuf;

use crate::utils::config::Config;

pub async fn config(key: String, value: Option<String>) {
    let mut config = Config::get();

    let value_mut = match key.as_str() {
        "locked" => &mut config.locked_path,
        _ => panic!(),
    };

    match value {
        Some(new_value) => {
            // set

            if key == "locked" {
                let path = PathBuf::from(new_value).canonicalize().unwrap();
                let new_value = path.to_string_lossy().to_string();

                *value_mut = Some(new_value);
            } else {
                *value_mut = Some(new_value)
            }
        }
        None => {
            // get
            println!("{key} => {value_mut:?}");
        }
    }

    Config::set(config);
}
