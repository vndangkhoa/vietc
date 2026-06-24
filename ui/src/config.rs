use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_input_method")]
    pub input_method: String,

    #[serde(default = "default_toggle_key")]
    pub toggle_key: String,

    #[serde(default = "default_start_enabled")]
    pub start_enabled: bool,

    #[serde(default)]
    pub auto_restore: AutoRestoreConfig,

    #[serde(default)]
    pub app_state: AppStateConfig,

    #[serde(default)]
    pub macros: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRestoreConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStateConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub english_apps: Vec<String>,

    #[serde(default)]
    pub vietnamese_apps: Vec<String>,
}

fn default_input_method() -> String { "telex".into() }
fn default_toggle_key() -> String { "space".into() }
fn default_start_enabled() -> bool { true }
fn default_true() -> bool { true }

impl Default for Config {
    fn default() -> Self {
        let mut macros = HashMap::new();
        macros.insert("ko".into(), "không".into());
        macros.insert("dc".into(), "được".into());
        macros.insert("vs".into(), "với".into());
        macros.insert("lm".into(), "làm".into());

        Self {
            input_method: default_input_method(),
            toggle_key: default_toggle_key(),
            start_enabled: default_start_enabled(),
            auto_restore: AutoRestoreConfig { enabled: true },
            app_state: AppStateConfig {
                enabled: true,
                english_apps: vec![
                    "code".into(), "vim".into(), "nvim".into(),
                    "terminal".into(), "kitty".into(), "alacritty".into(),
                ],
                vietnamese_apps: vec![
                    "telegram".into(), "discord".into(), "firefox".into(),
                ],
            },
            macros,
        }
    }
}

impl Config {
    pub fn load() -> Self {
        for path in config_paths() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(config) = toml::from_str::<Config>(&content) {
                    return config;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        fs::write(&path, content)?;
        Ok(())
    }

    pub fn path() -> PathBuf {
        config_path()
    }
}

fn config_path() -> PathBuf {
    config_paths()
        .into_iter()
        .find(|p| p.exists())
        .unwrap_or_else(|| {
            dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("vietc")
                .join("config.toml")
        })
}

fn config_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(config_dir) = dirs::config_dir() {
        paths.push(config_dir.join("vietc").join("config.toml"));
    }

    paths.push(PathBuf::from("vietc.toml"));

    paths
}
