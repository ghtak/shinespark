use config::{Config, Environment, File};
use serde::Deserialize;
use std::{env, path::PathBuf};

const CONFIG_FILE_PREFIX: &str = "application";
const CONFIG_BASE_DIR: &str = "configs";

#[derive(Debug, Deserialize, Clone, Copy, Default)]
#[serde(rename_all = "lowercase")]
pub enum TraceFormat {
    Json,
    Pretty,
    Full,
    #[default]
    Compact,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(default)]
pub struct TraceConsoleConfig {
    pub filter: String,
    pub format: TraceFormat,
    pub buffered_lines_limit: usize,
}

impl Default for TraceConsoleConfig {
    fn default() -> Self {
        Self {
            filter: "debug".to_string(),
            format: TraceFormat::Compact,
            buffered_lines_limit: 1024,
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct TraceFileConfig {
    pub filter: String,
    pub format: TraceFormat,
    pub buffered_lines_limit: usize,
    pub directory: String,
    pub prefix: String,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct TraceConfig {
    pub console: Option<TraceConsoleConfig>,
    pub file: Option<TraceFileConfig>,
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct AppConfig {
    pub trace: TraceConfig,
    pub database: DatabaseConfig,
}

impl AppConfig {
    pub fn load_dotenv() {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "local".into());
        let mut env_path = std::env::current_exe() // 실행 파일 위치
            .map(|mut p| {
                p.pop();
                p
            })
            .unwrap_or_else(|_| PathBuf::from("."));

        if !env_path.join(".env").exists() {
            env_path = PathBuf::from(".");
            if !env_path.join(".env").exists() {
                env_path = crate::util::workspace_dir();
            }
        }

        dotenvy::from_path(env_path.join(".env")).ok();
        dotenvy::from_path(env_path.join(format!(".env.{}", run_mode))).ok();
        dotenvy::from_path(env_path.join(".env.local")).ok();
    }

    pub fn new() -> crate::Result<Self> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "local".into());
        let config_path = env::var("CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let mut cur_path = std::env::current_exe()
                    .map(|mut p| {
                        p.pop();
                        p
                    })
                    .unwrap_or_default();
                cur_path.push(CONFIG_BASE_DIR);
                if cur_path.is_dir() {
                    cur_path
                } else {
                    let mut ws_path = crate::util::workspace_dir();
                    ws_path.push(CONFIG_BASE_DIR);
                    ws_path
                }
            });
        Self::load(config_path, &run_mode)
    }

    pub fn load(config_path: PathBuf, run_mode: &str) -> crate::Result<Self> {
        let mut config_path = config_path;
        config_path.push(CONFIG_FILE_PREFIX);
        let base_path = config_path.to_string_lossy();
        Config::builder()
            .add_source(File::with_name(&base_path).required(false))
            .add_source(File::with_name(&format!("{}-{}", base_path, run_mode)).required(false))
            .add_source(File::with_name(&format!("{}-local", base_path)).required(false))
            .add_source(Environment::with_prefix("APP").separator("_"))
            .build()
            .map_err(|e| anyhow::anyhow!(e).context("failed to build configuration"))?
            .try_deserialize()
            .map_err(|e| {
                crate::Error::Internal(
                    anyhow::anyhow!(e).context("failed to deserialize configuration"),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::config::AppConfig;

    #[test]
    fn test_load_config() {
        let config = super::AppConfig::new().expect("load config");
        println!("{:?}", config);
    }

    #[test]
    fn test_env_override() {
        // 병렬 테스트 시 환경 변수 오염에 주의해야 하므로,
        // serial_test를 쓰거나 독립적인 환경에서 실행하는 것이 좋습니다.
        unsafe {
            std::env::set_var(
                "APP_DATABASE_URL",
                "postgres://test_user:test_pass@localhost:5432/test_db",
            );
        }

        let config_result = AppConfig::new();

        // 테스트 성공/실패와 무관하게 환경 변수 원복(cleanup)
        unsafe {
            std::env::remove_var("APP_DATABASE_URL");
        }

        let config = config_result.expect("Failed to load config");
        assert_eq!(
            config.database.url,
            "postgres://test_user:test_pass@localhost:5432/test_db"
        );
    }

    #[test]
    fn test_dotenv() {
        AppConfig::load_dotenv();
        let config_result = AppConfig::new();
        let config = config_result.expect("Failed to load config");
        assert_eq!(
            config.database.url,
            "postgres://test_user:test_pass@localhost:5432/test_db"
        );
    }
}
