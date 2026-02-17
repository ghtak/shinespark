use config::{Config, Environment, File};
use serde::Deserialize;
use std::{env, path::Path};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingConfig {
    pub format: LoggingFormat,
    pub filter: String,
    pub file: Option<LoggingFileConfig>,
    pub buffer_limit: usize,
    pub lossy: bool,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LoggingFileConfig {
    pub format: LoggingFormat,
    pub directory: String,
    pub filename: String,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum LoggingFormat {
    Json,
    Pretty,
    Full,
    Compact,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
}

impl AppConfig {
    pub fn new<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
        let run_mode = env::var("APP_ENV").unwrap_or_else(|_| "".into());

        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name(&format!(
                "{}/default",
                path.as_ref().to_string_lossy()
            )))
            // Add in the current environment file
            // Default to 'development' env
            // Note that this file is _optional_
            .add_source(
                File::with_name(&format!("{}/{}", path.as_ref().to_string_lossy(), run_mode))
                    .required(false),
            )
            // Add in a local configuration file
            // This file should not be committed to git
            .add_source(
                File::with_name(&format!("{}/local", path.as_ref().to_string_lossy()))
                    .required(false),
            )
            // Add in Config from the environment (with a prefix of SHINESPARK)
            // Eg.. `SHINESPARK_DATABASE_URL=postgres://..` would set `database.url`
            .add_source(Environment::with_prefix("SHINESPARK").separator("_"))
            .build()
            .map_err(|e| crate::Error::Config(anyhow::anyhow!(e)))?;

        // You can deserialize the entire configuration as a struct
        s.try_deserialize()
            .map_err(|e| crate::Error::Config(anyhow::anyhow!(e)))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::util::workspace_dir;

    use super::*;

    fn test_config_dir() -> PathBuf {
        workspace_dir().join("configs").join("test")
    }

    #[test]
    fn test_load_config() {
        unsafe {
            env::set_var("APP_ENV", "dev");
        }

        // Try various formats to see what config-rs 0.14 likes
        unsafe {
            env::set_var("SHINESPARK_DATABASE_URL", "postgres://all_caps_double_dash");
        }

        let config = AppConfig::new(&test_config_dir()).expect("Failed to load Config");
        println!("Config: {:#?}", config);
        assert_eq!(config.database.url, "postgres://all_caps_double_dash");
    }

    #[test]
    fn test_layered_override() {
        let config = AppConfig::new(&test_config_dir()).expect("Failed to load Config");
        assert_eq!(config.server.port, 8080);

        unsafe {
            env::set_var("APP_ENV", "dev");
        }

        let config = AppConfig::new(&test_config_dir()).expect("Failed to load Config");
        assert_eq!(config.server.port, 8081);

        unsafe {
            env::set_var("SHINESPARK_SERVER_PORT", "9000");
        }
        let config = AppConfig::new(&test_config_dir()).expect("Failed to load Config");
        assert_eq!(config.server.port, 9000);
    }
}
