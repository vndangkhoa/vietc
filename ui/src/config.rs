use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRestoreConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_restore_keys")]
    pub trigger_keys: Vec<String>,
}

impl Default for AutoRestoreConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            trigger_keys: default_restore_keys(),
        }
    }
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

impl Default for AppStateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            english_apps: vec![],
            vietnamese_apps: vec![],
        }
    }
}

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
    pub macros: std::collections::HashMap<String, String>,

    #[serde(default = "default_grab")]
    pub grab: bool,

    #[serde(default = "default_false")]
    pub debug: bool,
}

fn default_input_method() -> String { "telex".into() }
fn default_toggle_key() -> String { "space".into() }
fn default_start_enabled() -> bool { true }
fn default_grab() -> bool { true }
fn default_true() -> bool { true }
fn default_false() -> bool { false }
fn default_restore_keys() -> Vec<String> { vec!["space".into(), "escape".into()] }

impl Default for Config {
    fn default() -> Self {
        Self {
            input_method: default_input_method(),
            toggle_key: default_toggle_key(),
            start_enabled: default_start_enabled(),
            auto_restore: AutoRestoreConfig::default(),
            app_state: AppStateConfig::default(),
            macros: std::collections::HashMap::new(),
            grab: default_grab(),
            debug: default_false(),
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

pub fn is_autostart_installed() -> bool {
    if let Some(config_dir) = dirs::config_dir() {
        config_dir.join("autostart").join("vietc-tray.desktop").exists()
    } else {
        false
    }
}

pub fn uninstall_autostart() {
    if let Some(config_dir) = dirs::config_dir() {
        let desktop_file = config_dir.join("autostart").join("vietc-tray.desktop");
        if desktop_file.exists() {
            let _ = fs::remove_file(desktop_file);
            eprintln!("[vietc] Removed autostart entry");
        }
    }
}

pub fn install_autostart() {
    if let Some(config_dir) = dirs::config_dir() {
        let autostart_dir = config_dir.join("autostart");
        let desktop_file = autostart_dir.join("vietc-tray.desktop");
        let _ = fs::create_dir_all(&autostart_dir);

        let exec_path = std::env::var("APPIMAGE")
            .ok()
            .unwrap_or_else(|| {
                std::env::current_exe()
                    .unwrap_or_else(|_| PathBuf::from("vietc-tray"))
                    .to_string_lossy()
                    .into_owned()
            });

        let content = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=Viet+\n\
             Comment=Vietnamese Input Method\n\
             Exec={}\n\
             Icon=input-keyboard\n\
             Terminal=false\n\
             Categories=Utility;System;\n\
             X-GNOME-Autostart-enabled=true\n\
             StartupNotify=false\n",
            exec_path
        );

        let _ = fs::write(desktop_file, content);
        eprintln!("[vietc] Installed autostart entry");
    }
}
