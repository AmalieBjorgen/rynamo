use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    pub environments: Vec<String>,
    pub current_env: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        if !config_path.exists() {
            return Ok(Config::default());
        }

        let content = fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = toml::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    fn config_path() -> Result<PathBuf> {
        let mut path = dirs::config_dir().context("Could not find config directory")?;
        path.push("rynamo");
        path.push("config.toml");
        Ok(path)
    }

    pub fn add_environment(&mut self, url: String) {
        if !self.environments.contains(&url) {
            self.environments.push(url.clone());
        }
        self.current_env = Some(url);
    }
}
