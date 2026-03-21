use config::{Config, Environment, File};
use serde::Deserialize;
use std::{env, path::PathBuf};

const CONFIG_FILE_PREFIX: &'static str = "application";

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum TraceFormat {
    Json,
    Pretty,
    Full,
    Compact,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ConsoleConfig {
    pub filter: String,
    pub format: TraceFormat,
    pub buffered_lines_limit: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TraceConfig {
    pub console: Option<ConsoleConfig>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub trace: TraceConfig,
}

impl AppConfig {
    pub fn new() -> crate::Result<Self> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "dev".into());
        let config_dir = env::var("CONFIG_DIR").unwrap_or_else(|_| "configs".into());

        let mut builder = Config::builder();
        //#[cfg(test)]
        {
            let mut path = crate::util::workspace_dir();
            path.push("configs");
            path.push(CONFIG_FILE_PREFIX);
            let t_path = path
                .to_str()
                .ok_or(crate::Error::IllegalState("invalid config path".into()))?
                .to_string();
            println!("{}", t_path);
            builder = builder
                .add_source(File::with_name(&t_path).required(false))
                .add_source(File::with_name(&format!("{}-{}", t_path, run_mode)).required(false))
                .add_source(File::with_name(&format!("{}-local", t_path)).required(false))
        }

        let mut config_path = PathBuf::from(config_dir);
        config_path.push(CONFIG_FILE_PREFIX);
        let config_path = config_path
            .to_str()
            .ok_or(crate::Error::IllegalState("invalid config path".into()))?
            .to_string();

        builder = builder
            .add_source(File::with_name(&config_path).required(false))
            .add_source(File::with_name(&format!("{}-{}", config_path, run_mode)).required(false))
            .add_source(File::with_name(&format!("{}-local", config_path)).required(false))
            .add_source(Environment::with_prefix("APP"));

        let s = builder
            .build()
            .map_err(|e| anyhow::Error::new(e).context("failed to build configuration"))?;
        s.try_deserialize().map_err(|e| {
            anyhow::Error::new(e)
                .context("failed to deserialize configuration")
                .into()
        })
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_load_config() {
        let config = super::AppConfig::new().expect("load config");
        println!("{:?}", config);
    }
}

// #[derive(Debug, Deserialize, Clone)]
// pub struct CryptoConfig {
//     pub argon2: Argon2Config,
// }

// #[derive(Debug, Deserialize, Clone)]
// pub struct Argon2Config {
//     pub memory_kib: u32,
//     pub iterations: u32,
//     pub parallelism: u32,
// }

// #[derive(Debug, Deserialize, Clone)]
// pub struct TraceConfig {
//     pub console: Option<TraceConsoleConfig>,
//     pub file: Option<TraceFileConfig>,
//     pub otel: Option<TraceOtelConfig>,
// }

// impl Default for TraceConfig {
//     fn default() -> Self {
//         Self {
//             console: Some(TraceConsoleConfig::default()),
//             file: None,
//             otel: None,
//         }
//     }
// }

// #[derive(Debug, Deserialize, Clone)]
// pub struct TraceConsoleConfig {
//     pub filter: String,
//     pub format: TraceFormat,
//     pub buffer_limit: usize,
//     pub lossy: bool,
// }

// impl Default for TraceConsoleConfig {
//     fn default() -> Self {
//         Self {
//             filter: "debug".to_string(),
//             format: TraceFormat::Compact,
//             buffer_limit: 1024,
//             lossy: true,
//         }
//     }
// }

// #[derive(Debug, Deserialize, Clone)]
// pub struct TraceFileConfig {
//     pub filter: String,
//     pub directory: String,
//     pub filename: String,
//     pub format: TraceFormat,
//     pub buffer_limit: usize,
//     pub lossy: bool,
// }

// #[derive(Debug, Deserialize, Clone)]
// pub struct TraceOtelConfig {
//     pub filter: String,
//     pub endpoint: String,
// }

// #[derive(Debug, Deserialize, Clone, Copy)]
// #[serde(rename_all = "lowercase")]
// pub enum TraceFormat {
//     Json,
//     Pretty,
//     Full,
//     Compact,
// }

// #[derive(Debug, Deserialize, Clone)]
// pub struct DatabaseConfig {
//     pub url: String,
//     pub max_connections: u32,
// }

// #[derive(Debug, Deserialize, Clone)]
// pub struct ServerConfig {
//     pub port: u16,
//     pub host: String,
// }

// #[cfg(test)]
// mod tests {
//     use std::{collections::HashMap, path::PathBuf};

//     use crate::util::workspace_dir;

//     use super::*;

//     fn test_config_dir() -> PathBuf {
//         workspace_dir().join("configs")
//     }

//     #[test]
//     fn test_load_config() {
//         let mut overrides = HashMap::new();
//         overrides.insert(
//             "database.url".to_string(),
//             "postgres://injected_override".to_string(),
//         );

//         let config = AppConfig::load_with_options(
//             &test_config_dir(),
//             Some("test".to_string()),
//             Some(overrides),
//         )
//         .expect("Failed to load Config");

//         println!("Config: {:#?}", config);
//         assert_eq!(config.database.url, "postgres://injected_override");
//     }

//     #[test]
//     fn test_layered_override() {
//         // 1. Default load
//         let config = AppConfig::load_with_options(&test_config_dir(), None, None)
//             .expect("Failed to load Config");
//         assert_eq!(config.server.port, 8080);

//         // 2. Test environment override (Argon2 settings)
//         let config =
//             AppConfig::load_with_options(&test_config_dir(), Some("test".to_string()), None)
//                 .expect("Failed to load Config");
//         assert_eq!(config.crypto.argon2.memory_kib, 8);

//         // 3. Explicit override (simulating env var priority)
//         let mut overrides = HashMap::new();
//         overrides.insert("server.port".to_string(), "9000".to_string());

//         let config = AppConfig::load_with_options(&test_config_dir(), None, Some(overrides))
//             .expect("Failed to load Config");
//         assert_eq!(config.server.port, 9000);
//     }

//     /*
//     /// 실제 환경 변수(ENV)를 사용하여 테스트하는 경우의 예시입니다.
//     /// 글로벌 상태를 공유하므로 병렬 테스트 시 주의가 필요합니다.
//     /// (serial_test 크레이트를 사용하거나 cargo test -- --test-threads=1 로 실행)
//     #[test]
//     fn test_actual_env_override() {
//         unsafe {
//             std::env::set_var("APP_ENV", "dev");
//             std::env::set_var("SHINESPARK_SERVER_PORT", "9999");
//         }

//         let config = AppConfig::new(&test_config_dir()).expect("Failed to load Config");

//         assert_eq!(config.server.port, 9999);

//         // 테스트 종료 후 환경 변수 정리 (권장)
//         unsafe {
//             std::env::remove_var("APP_ENV");
//             std::env::remove_var("SHINESPARK_SERVER_PORT");
//         }
//     }
//     */
// }
