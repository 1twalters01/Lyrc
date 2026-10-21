use std::{fs, io::ErrorKind, path::Path};

use crate::config::Config;

pub fn load_config(
    toml_path: Option<&Path>,
    json_path: Option<&Path>,
) -> Result<Config, Box<dyn std::error::Error>> {
    if let Some(path) = toml_path {
        match fs::read_to_string(path) {
            Ok(contents) => return Ok(toml::from_str(&contents)?),
            Err(error) if error.kind() == ErrorKind::NotFound => {},
            Err(error) => return Err(error.into()),
        }
    }

    if let Some(path) = json_path {
        match fs::read_to_string(path) {
            Ok(contents) => return Ok(serde_json::from_str(&contents)?),
            Err(error) if error.kind() == ErrorKind::NotFound => {},
            Err(error) => return Err(error.into()),
        }
    }

    Ok(Config::default())
}
