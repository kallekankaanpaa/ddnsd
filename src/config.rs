use once_cell::sync::OnceCell;
use std::{fs, io::Write, path::Path};

use serde::{Deserialize, Serialize};

pub static CONFIG: OnceCell<Config> = OnceCell::new();

#[derive(Debug, Deserialize, Serialize)]
pub struct Ddns {
    pub hostname: String,
    pub username: String,
    pub password: String,
    pub max_interval: u64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct IpChecker {
    pub interval: u64,
    pub url: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Config {
    pub ddns: Ddns,
    pub ip_checker: IpChecker,
}

pub fn init_config() -> anyhow::Result<Config> {
    let config_path = Path::new("config.toml");

    if !config_path.exists() {
        write_default_config(config_path)
    } else {
        read_config(config_path)
    }
}

fn write_default_config(path: &Path) -> anyhow::Result<Config> {
    let default_config = Config::default();
    let mut file = fs::File::create(path).unwrap();
    file.write_all(toml::to_string(&default_config)?.as_bytes())?;
    Ok(default_config)
}

fn read_config(path: &Path) -> anyhow::Result<Config> {
    let config_string = fs::read_to_string(path)?;
    Ok(toml::from_str(&config_string)?)
}

impl Default for Ddns {
    fn default() -> Self {
        Self {
            hostname: "<placeholder>".to_string(),
            username: "<placeholder>".to_string(),
            password: "hostname".to_string(),
            max_interval: 6 * 24 * 60 * 60,
        }
    }
}

impl Default for IpChecker {
    fn default() -> Self {
        Self {
            interval: 60,
            url: "https://ident.me".to_string(),
        }
    }
}
