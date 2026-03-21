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
pub struct ConsoleConfig {
    pub filter: String,
    pub format: TraceFormat,
    pub buffered_lines_limit: usize,
}

impl Default for ConsoleConfig {
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
pub struct TraceConfig {
    pub console: Option<ConsoleConfig>,
}

#[derive(Debug, Deserialize, Clone, Default)]
#[serde(default)]
pub struct AppConfig {
    pub trace: TraceConfig,
}

impl AppConfig {
    pub fn new() -> crate::Result<Self> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "dev".into());
        let config_path = env::var("CONFIG_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                let mut cur_path = env::current_dir().unwrap_or_default();
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
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()
            .map_err(|e| anyhow::anyhow!(e).context("failed to build configuration"))?
            .try_deserialize()
            .map_err(|e| {
                anyhow::anyhow!(e)
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
