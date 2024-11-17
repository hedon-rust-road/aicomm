use std::{env, fs::File};

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub pk: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub db_url: String,
    pub db_user: Option<String>,
    pub db_password: Option<String>,
    pub db_name: String,
    pub base_dir: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let ret = match (
            File::open("analytics.yml"),
            File::open("/etc/config/analytics.yml"),
            env::var("ANALYTICS_CONFIG"),
        ) {
            (Ok(file), _, _) => serde_yaml::from_reader(file),
            (Err(_), Ok(file), _) => serde_yaml::from_reader(file),
            (Err(_), Err(_), Ok(path)) => serde_yaml::from_reader(File::open(path)?),
            _ => bail!("Failed to open config file"),
        };

        Ok(ret?)
    }
}
