use crate::utils::config::Config;

pub async fn config(key: String, value: Option<String>) -> anyhow::Result<()> {
    let mut config = Config::read();

    let value_mut = match key.as_str() {
        "locked" => &mut config.locked_path,
        _ => panic!(),
    };

    match value {
        Some(new_value) => {
            // set
            *value_mut = Some(new_value)
        }
        None => {
            // get
            println!("{key} => {value_mut:?}");
        }
    }

    Config::write(config);

    Ok(())
}
