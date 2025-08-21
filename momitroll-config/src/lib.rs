use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::{env, path::PathBuf};

pub const CONFIG_FILE_NAME: &str = "momitroll-config";

#[derive(Deserialize)]
pub struct Config {
    pub migration: MigrationConfig,
    pub db: DbConfig,
    #[serde(rename = "creds-env-vars")]
    pub creds_env_vars: CredEnvVars,
}

impl Config {
    pub fn load() -> Result<Self> {
        match find_config_file()? {
            ConfigFile::TOML(path) => {
                let config: Config = toml::from_str(&std::fs::read_to_string(path)?)?;
                Ok(config)
            }
            ConfigFile::JSON(path) => {
                let config: Config = serde_json::from_str(&std::fs::read_to_string(path)?)?;
                Ok(config)
            }
        }
    }
}

#[derive(Deserialize)]
pub struct MigrationConfig {
    pub dir: String,
    #[serde(rename = "changelog-coll-name")]
    pub changelog_coll_name: String,
}

impl MigrationConfig {
    pub fn coll_name(&self) -> String {
        format!("_{}", self.changelog_coll_name)
    }
}

#[derive(Deserialize)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
}

#[derive(Deserialize)]
pub struct CredEnvVars {
    pub username: String,
    pub password: String,
}

impl CredEnvVars {
    pub fn get_creds(&self) -> Result<(String, String)> {
        Ok((env::var(&self.username)?, env::var(&self.password)?))
    }
}

#[allow(non_snake_case)]
pub enum ConfigFile {
    TOML(PathBuf),
    JSON(PathBuf),
}

impl From<ConfigFile> for PathBuf {
    fn from(file: ConfigFile) -> Self {
        match file {
            ConfigFile::TOML(path) => path,
            ConfigFile::JSON(path) => path,
        }
    }
}

impl ConfigFile {
    pub fn is_valid_extension(ext: &str) -> bool {
        ext == "toml" || ext == "json"
    }
}

pub fn find_config_file() -> Result<ConfigFile> {
    use std::env;
    use walkdir::WalkDir;

    let current_dir = env::current_dir()?;

    for entry in WalkDir::new(&current_dir) {
        let entry = entry?;
        let path = entry.path();
        let extension = path.extension().and_then(|s| s.to_str());

        match extension {
            Some("toml") => return Ok(ConfigFile::TOML(path.to_path_buf())),
            Some("json") => return Ok(ConfigFile::JSON(path.to_path_buf())),
            _ => continue,
        }
    }

    Err(anyhow!("didn't found config file"))
}
