use anyhow::bail;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    // pub database: DatabaseConfig,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[allow(dead_code)]
impl AppConfig {
    pub fn try_load() -> anyhow::Result<Self> {
        let config = match (
            File::open("config.yaml"),
            File::open("/etc/config.yaml"),
            env::var("APP_CONFIG_PATH"),
        ) {
            (Ok(reader), _, _) => serde_yaml::from_reader(reader),
            (_, Ok(reader), _) => serde_yaml::from_reader(reader),
            (_, _, Ok(path)) => serde_yaml::from_reader(File::open(path)?),
            _ => bail!(anyhow::anyhow!("Config file not found")),
        };

        Ok(config?)
    }
}
