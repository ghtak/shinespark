use config::{Config, Environment, File};
use serde::Deserialize;
use std::{env, path::Path};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub logging: LoggingConfig,
    pub crypto: CryptoConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct CryptoConfig {
    pub argon2: Argon2Config,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Argon2Config {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
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
        Self::load_with_options(path, Some(run_mode), None)
    }

    pub fn load_with_options<P: AsRef<Path>>(
        path: P,
        run_mode: Option<String>,
        overrides: Option<std::collections::HashMap<String, String>>,
    ) -> crate::Result<Self> {
        let run_mode = run_mode.unwrap_or_else(|| "".into());
        let mut builder = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name(&format!(
                "{}/default",
                path.as_ref().to_string_lossy()
            )))
            // Add in the current environment file
            // Note that this file is _optional_
            .add_source(
                File::with_name(&format!(
                    "{}/{}",
                    path.as_ref().to_string_lossy(),
                    run_mode
                ))
                .required(false),
            )
            // Add in a local configuration file
            // This file should not be committed to git
            .add_source(
                File::with_name(&format!(
                    "{}/local",
                    path.as_ref().to_string_lossy()
                ))
                .required(false),
            )
            // Add in Config from the environment (with a prefix of SHINESPARK)
            .add_source(Environment::with_prefix("SHINESPARK").separator("_"));

        // Add explicit overrides if provided
        if let Some(overrides) = overrides {
            for (key, value) in overrides {
                builder = builder
                    .set_override(key, value)
                    .map_err(|e| crate::Error::Config(anyhow::anyhow!(e)))?;
            }
        }

        let s = builder
            .build()
            .map_err(|e| crate::Error::Config(anyhow::anyhow!(e)))?;

        // You can deserialize the entire configuration as a struct
        s.try_deserialize()
            .map_err(|e| crate::Error::Config(anyhow::anyhow!(e)))
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::PathBuf};

    use crate::util::workspace_dir;

    use super::*;

    fn test_config_dir() -> PathBuf {
        workspace_dir().join("configs")
    }

    #[test]
    fn test_load_config() {
        let mut overrides = HashMap::new();
        overrides.insert(
            "database.url".to_string(),
            "postgres://injected_override".to_string(),
        );

        let config = AppConfig::load_with_options(
            &test_config_dir(),
            Some("test".to_string()),
            Some(overrides),
        )
        .expect("Failed to load Config");

        println!("Config: {:#?}", config);
        assert_eq!(config.database.url, "postgres://injected_override");
    }

    #[test]
    fn test_layered_override() {
        // 1. Default load
        let config =
            AppConfig::load_with_options(&test_config_dir(), None, None)
                .expect("Failed to load Config");
        assert_eq!(config.server.port, 8080);

        // 2. Test environment override (Argon2 settings)
        let config = AppConfig::load_with_options(
            &test_config_dir(),
            Some("test".to_string()),
            None,
        )
        .expect("Failed to load Config");
        assert_eq!(config.crypto.argon2.memory_kib, 8);

        // 3. Explicit override (simulating env var priority)
        let mut overrides = HashMap::new();
        overrides.insert("server.port".to_string(), "9000".to_string());

        let config = AppConfig::load_with_options(
            &test_config_dir(),
            None,
            Some(overrides),
        )
        .expect("Failed to load Config");
        assert_eq!(config.server.port, 9000);
    }

    /*
    /// 실제 환경 변수(ENV)를 사용하여 테스트하는 경우의 예시입니다.
    /// 글로벌 상태를 공유하므로 병렬 테스트 시 주의가 필요합니다.
    /// (serial_test 크레이트를 사용하거나 cargo test -- --test-threads=1 로 실행)
    #[test]
    fn test_actual_env_override() {
        unsafe {
            std::env::set_var("APP_ENV", "dev");
            std::env::set_var("SHINESPARK_SERVER_PORT", "9999");
        }

        let config = AppConfig::new(&test_config_dir()).expect("Failed to load Config");

        assert_eq!(config.server.port, 9999);

        // 테스트 종료 후 환경 변수 정리 (권장)
        unsafe {
            std::env::remove_var("APP_ENV");
            std::env::remove_var("SHINESPARK_SERVER_PORT");
        }
    }
    */
}
