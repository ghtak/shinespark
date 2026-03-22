use config::{Config, Environment, File};
use serde::Deserialize;
use std::{env, path::PathBuf};

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

#[derive(Debug, Deserialize, Clone, Default)]
pub struct HttpConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct AppConfig {
    pub trace: TraceConfig,
    pub database: DatabaseConfig,
    pub http: HttpConfig,
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        if let Ok(p) = env::var("CONFIG_PATH") {
            return PathBuf::from(p);
        }

        #[cfg(debug_assertions)]
        return crate::util::workspace_root().join("configs");

        #[cfg(not(debug_assertions))]
        return crate::util::base_path();
    }

    pub fn load_dotenv() {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "local".into());
        let env_path = Self::config_path();
        let exe_name = crate::util::base_executable_name();
        for file in [
            format!("{}.env", exe_name),
            format!("{}-{}.env", exe_name, run_mode),
            format!("{}-local.env", exe_name),
        ] {
            let env_file = env_path.join(file);
            if env_file.exists() {
                dotenvy::from_path_override(env_file).ok();
            }
        }
    }

    pub fn new() -> crate::Result<Self> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "local".into());
        return Self::load(Self::config_path(), &run_mode);
    }

    pub fn load(config_path: PathBuf, run_mode: &str) -> crate::Result<Self> {
        let config_file = config_path
            .join(crate::util::base_executable_name())
            .to_string_lossy()
            .into_owned();

        Config::builder()
            .add_source(File::with_name(&config_file).required(false))
            .add_source(File::with_name(&format!("{}-{}", config_file, run_mode)).required(false))
            .add_source(File::with_name(&format!("{}-local", config_file)).required(false))
            .add_source(Environment::with_prefix("APP").separator("__"))
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
    use serial_test::serial;

    use crate::config::AppConfig;

    #[test]
    #[serial]
    fn test_load_config() {
        let config = super::AppConfig::new().expect("load config");
        println!("{:?}", config);
    }

    #[test]
    #[serial]
    fn test_env_override() {
        // 병렬 테스트 시 환경 변수 오염에 주의해야 하므로,
        // serial_test를 쓰거나 독립적인 환경에서 실행하는 것이 좋습니다.
        unsafe {
            std::env::set_var(
                "APP__DATABASE__URL",
                "postgres://test_user:test_pass@localhost:5432/test_db",
            );
            std::env::set_var("APP__DATABASE__MAX_CONNECTIONS", "999");
        }

        let config_result = AppConfig::new();

        // 테스트 성공/실패와 무관하게 환경 변수 원복(cleanup)
        unsafe {
            std::env::remove_var("APP__DATABASE__URL");
        }

        let config = config_result.expect("Failed to load config");
        assert_eq!(
            config.database.url,
            "postgres://test_user:test_pass@localhost:5432/test_db"
        );
        assert_eq!(config.database.max_connections, 999);
    }

    #[test]
    #[serial]
    fn test_trace_env_override() {
        // 계층이 깊은(Trace -> Console / File) 설정들의 환경 변수 오버라이딩 테스트
        unsafe {
            std::env::set_var("APP__TRACE__CONSOLE__FILTER", "warn");
            std::env::set_var("APP__TRACE__FILE__DIRECTORY", "/var/log/shinespark");
            std::env::set_var("APP__TRACE__FILE__FORMAT", "json");
        }

        let config_result = AppConfig::new();

        unsafe {
            std::env::remove_var("APP__TRACE__CONSOLE__FILTER");
            std::env::remove_var("APP__TRACE__FILE__DIRECTORY");
            std::env::remove_var("APP__TRACE__FILE__FORMAT");
        }

        let config = config_result.expect("Failed to load config");

        // 1. Console config 검증
        let console_cfg = config.trace.console.expect("Console config should exist");
        assert_eq!(console_cfg.filter, "warn");

        // 2. File config 검증 (경로 및 Json enum 매핑 확인)
        let file_cfg = config.trace.file.expect("File config should exist");
        assert_eq!(file_cfg.directory, "/var/log/shinespark");
        assert!(matches!(file_cfg.format, crate::config::TraceFormat::Json));
    }

    #[test]
    #[serial]
    fn test_load_dotenv() {
        use std::fs;
        use std::io::Write;

        let ws = AppConfig::config_path();

        // 1. 파일에 키가 없으면 추가하거나 주석을 해제하는 헬퍼 함수
        let setup_env_file = |file_name: &str, line_to_add: &str| {
            let path = ws.join(file_name);
            let content = fs::read_to_string(&path).unwrap_or_default();
            let uncommented = content.replace(&format!("# {}", line_to_add), line_to_add);
            if !uncommented.contains(line_to_add) {
                let mut file = fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&path)
                    .unwrap();
                writeln!(file, "\n{}", line_to_add).unwrap();
            } else {
                fs::write(&path, uncommented).unwrap();
            }
        };

        // 2. 파일에서 해당 라인을 찾아 주석 처리(#)하는 헬퍼 함수
        let teardown_env_file = |file_name: &str, line_to_comment: &str| {
            let path = ws.join(file_name);
            if let Ok(content) = fs::read_to_string(&path) {
                let new_content =
                    content.replace(line_to_comment, &format!("# {}", line_to_comment));
                fs::write(&path, new_content).ok();
            }
        };

        let base_line = "APP_OVERRIDE_TEST_VAL=base";
        let dev_line = "APP_OVERRIDE_TEST_VAL=dev";
        let local_line = "APP_OVERRIDE_TEST_VAL=local";

        let exe_name = crate::util::base_executable_name();

        let app_env = format!("{}.env", exe_name);
        let app_dev_env = format!("{}-dev.env", exe_name);
        let app_local_env = format!("{}-local.env", exe_name);

        // 워크스페이스의 환경 변수 셋업
        setup_env_file(&app_env, base_line);
        setup_env_file(&app_dev_env, dev_line);
        setup_env_file(&app_local_env, local_line);

        unsafe {
            std::env::set_var("RUN_MODE", "dev");
        }

        // Step 1: 3개 모두 존재할 경우 (.local.env 최우선)
        AppConfig::load_dotenv();
        assert_eq!(std::env::var("APP_OVERRIDE_TEST_VAL").unwrap(), "local");

        // Step 2: .env.local 주석 처리 후 재로딩 (.dev.env 우선)
        teardown_env_file(&app_local_env, local_line);
        AppConfig::load_dotenv();
        assert_eq!(std::env::var("APP_OVERRIDE_TEST_VAL").unwrap(), "dev");

        // Step 3: .env.dev 주석 처리 후 재로딩 (.env 우선)
        teardown_env_file(&app_dev_env, dev_line);
        AppConfig::load_dotenv();
        assert_eq!(std::env::var("APP_OVERRIDE_TEST_VAL").unwrap(), "base");

        // 마무리: .env에 남은 설정도 주석 처리 (원상 복구)
        teardown_env_file(&app_env, base_line);

        // 다른 테스트에 영향이 없도록 환경 변수 클렌징
        unsafe {
            std::env::remove_var("APP_OVERRIDE_TEST_VAL");
        }
    }
}
