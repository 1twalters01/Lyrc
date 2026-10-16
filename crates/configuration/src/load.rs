use std::{fs, path::Path};

use crate::config::Config;

pub fn load_config(path: &Path) -> Result<Config, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(path)?;
    let config = toml::from_str(&contents)?;

    Ok(config)
}
