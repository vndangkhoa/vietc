// SPDX-License-Identifier: MIT
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct Config {
    #[serde(default = "default_input_method")]
    pub input_method: String,

    #[serde(default = "default_toggle_key")]
    pub toggle_key: String,

    #[serde(default = "default_toggle_method_key")]
    pub toggle_method_key: String,

    #[serde(default = "default_start_enabled")]
    pub start_enabled: bool,

    #[serde(default)]
    pub auto_restore: AutoRestoreConfig,

    #[serde(default)]
    pub password_detection: PasswordDetectionConfig,

    #[serde(default)]
    pub app_state: AppStateConfig,

    #[serde(default)]
    pub macros: HashMap<String, String>,

    #[serde(default)]
    pub grab: bool,

    #[serde(default = "default_false")]
    pub debug: bool,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct PasswordDetectionConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_true")]
    pub check_atspi2: bool,

    #[serde(default = "default_true")]
    pub check_window_title: bool,

    #[serde(default = "default_title_keywords")]
    pub title_keywords: Vec<String>,

    #[serde(default = "default_password_apps")]
    pub password_apps: Vec<String>,
}

impl Default for PasswordDetectionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            check_atspi2: true,
            check_window_title: true,
            title_keywords: default_title_keywords(),
            password_apps: default_password_apps(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AutoRestoreConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default = "default_restore_keys")]
    pub trigger_keys: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct AppStateConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,

    #[serde(default)]
    pub english_apps: Vec<String>,

    #[serde(default)]
    pub vietnamese_apps: Vec<String>,

    #[serde(default = "default_bypass_apps")]
    pub bypass_apps: Vec<String>,
}

impl Default for AutoRestoreConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            trigger_keys: default_restore_keys(),
        }
    }
}

impl Default for AppStateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            english_apps: default_english_apps(),
            vietnamese_apps: default_vietnamese_apps(),
            bypass_apps: default_bypass_apps(),
        }
    }
}

fn default_input_method() -> String {
    "vni".into()
}
fn default_toggle_key() -> String {
    "space".into()
}
fn default_toggle_method_key() -> String {
    "shift".into()
}
fn default_start_enabled() -> bool {
    true
}
fn default_true() -> bool {
    true
}
fn default_false() -> bool {
    false
}
fn default_restore_keys() -> Vec<String> {
    vec!["space".into(), "escape".into()]
}
fn default_title_keywords() -> Vec<String> {
    vec![
        "password".into(),
        "passphrase".into(),
        "secret".into(),
        "mật khẩu".into(),
        "mk".into(),
        "sudo".into(),
    ]
}
fn default_password_apps() -> Vec<String> {
    vec![
        "pinentry".into(),
        "pinentry-gtk-2".into(),
        "pinentry-qt".into(),
        "lxqt-sudo".into(),
        "kdesudo".into(),
        "gksudo".into(),
        "polkit-gnome-authentication-agent-1".into(),
        "kwallet".into(),
        "gnome-keyring".into(),
        "ssh-askpass".into(),
    ]
}

fn default_english_apps() -> Vec<String> {
    vec![
        "code".into(),
        "jetbrains".into(),
        "intellij".into(),
        "pycharm".into(),
        "webstorm".into(),
        "vim".into(),
        "nvim".into(),
    ]
}

fn default_bypass_apps() -> Vec<String> {
    vec![
        "terminal".into(),
        "kitty".into(),
        "alacritty".into(),
        "foot".into(),
        "wezterm".into(),
        "konsole".into(),
        "gnome-terminal".into(),
        "st".into(),
        "urxvt".into(),
        "xterm".into(),
        "steam".into(),
        "dota".into(),
        "csgo".into(),
        "minecraft".into(),
        "factorio".into(),
    ]
}

fn default_vietnamese_apps() -> Vec<String> {
    vec![
        "telegram".into(),
        "discord".into(),
        "slack".into(),
        "firefox".into(),
        "chromium".into(),
        "thunderbird".into(),
    ]
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let paths = [
            dirs().map(|d| d.join("vietc").join("config.toml")),
            Some(PathBuf::from("vietc.toml")),
            // AppImage bundled config: <exe dir>/../../etc/vietc/config.toml
            std::env::current_exe().ok().and_then(|exe| {
                exe.parent()
                    .and_then(|p| p.parent())
                    .and_then(|p| p.parent())
                    .map(|p| p.join("etc").join("vietc").join("config.toml"))
            }),
        ];

        for path in paths.into_iter().flatten() {
            if path.exists() {
                let content = fs::read_to_string(&path)?;
                let config: Config = toml::from_str(&content)?;
                eprintln!("[vietc] Loaded config from: {}", path.display());
                return Ok(config);
            }
        }

        eprintln!("[vietc] Using default config");
        Ok(Self::default())
    }

    pub fn load_from(path: &std::path::Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut macros = HashMap::new();
        macros.insert("ko".into(), "không".into());
        macros.insert("kc".into(), "không có".into());
        macros.insert("ko dc".into(), "không được".into());
        macros.insert("dc".into(), "được".into());
        macros.insert("ng".into(), "người".into());
        macros.insert("nk".into(), "như".into());
        macros.insert("vs".into(), "với".into());
        macros.insert("lm".into(), "làm".into());
        macros.insert("rd".into(), "rất".into());
        macros.insert("bt".into(), "biết".into());

        Self {
            input_method: default_input_method(),
            toggle_key: default_toggle_key(),
            toggle_method_key: default_toggle_method_key(),
            start_enabled: default_start_enabled(),
            auto_restore: AutoRestoreConfig::default(),
            password_detection: PasswordDetectionConfig::default(),
            app_state: AppStateConfig::default(),
            macros,
            grab: false, // default false so daemon works without root (needs input group for uinput)
            debug: false,
        }
    }
}

fn dirs() -> Option<PathBuf> {
    std::env::var("XDG_CONFIG_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| PathBuf::from(h).join(".config"))
        })
}

pub fn find_config_path() -> PathBuf {
    let paths = [
        dirs().map(|d| d.join("vietc").join("config.toml")),
        Some(PathBuf::from("vietc.toml")),
        std::env::current_exe().ok().and_then(|exe| {
            exe.parent()
                .and_then(|p| p.parent())
                .and_then(|p| p.parent())
                .map(|p| p.join("etc").join("vietc").join("config.toml"))
        }),
    ];

    for path in paths.into_iter().flatten() {
        if path.exists() {
            return path;
        }
    }

    // Default to current directory
    PathBuf::from("vietc.toml")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_full_config() {
        let toml = r#"
input_method = "vni"
toggle_key = "shift"
start_enabled = false

[auto_restore]
enabled = false

[app_state]
enabled = true
english_apps = ["code", "vim"]
vietnamese_apps = ["telegram", "discord"]

[macros]
ko = "không"
dc = "được"
vs = "với"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.input_method, "vni");
        assert_eq!(config.toggle_key, "shift");
        assert!(!config.start_enabled); // explicitly set to false in test toml
        assert!(!config.auto_restore.enabled);
        assert!(config.app_state.enabled);
        assert_eq!(config.app_state.english_apps, vec!["code", "vim"]);
        assert_eq!(
            config.app_state.vietnamese_apps,
            vec!["telegram", "discord"]
        );
        assert_eq!(config.macros.get("ko").unwrap(), "không");
        assert_eq!(config.macros.get("dc").unwrap(), "được");
        assert_eq!(config.macros.get("vs").unwrap(), "với");
    }

    #[test]
    fn parse_empty_config_uses_defaults() {
        let toml = "";
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.input_method, "vni");
        assert_eq!(config.toggle_key, "space");
        assert!(config.start_enabled); // default changed to true
        assert!(!config.auto_restore.enabled);
        assert!(config.app_state.enabled);
        assert!(!config.app_state.english_apps.is_empty());
        assert!(!config.app_state.vietnamese_apps.is_empty());
    }

    #[test]
    fn parse_partial_config() {
        let toml = r#"
input_method = "vni"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.input_method, "vni");
        assert_eq!(config.toggle_key, "space"); // default
        assert!(config.start_enabled); // default changed to true
    }

    #[test]
    fn parse_macros_only() {
        let toml = r#"
[macros]
hello = "world"
foo = "bar"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.macros.len(), 2);
        assert_eq!(config.macros.get("hello").unwrap(), "world");
        assert_eq!(config.macros.get("foo").unwrap(), "bar");
    }

    #[test]
    fn parse_empty_macros() {
        let toml = r#"
[macros]
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.macros.is_empty());
    }

    #[test]
    fn parse_app_lists() {
        let toml = r#"
[app_state]
english_apps = ["vim", "neovim"]
vietnamese_apps = ["zalo", "messenger"]
bypass_apps = ["kitty"]
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.app_state.english_apps, vec!["vim", "neovim"]);
        assert_eq!(config.app_state.vietnamese_apps, vec!["zalo", "messenger"]);
        assert_eq!(config.app_state.bypass_apps, vec!["kitty"]);
    }

    #[test]
    fn default_config_has_macros() {
        let config = Config::default();
        assert!(config.macros.contains_key("ko"));
        assert!(config.macros.contains_key("dc"));
        assert!(config.macros.contains_key("vs"));
        assert!(config.macros.contains_key("lm"));
    }

    #[test]
    fn default_config_english_apps() {
        let config = Config::default();
        assert!(config.app_state.english_apps.contains(&"code".to_string()));
        assert!(config.app_state.english_apps.contains(&"vim".to_string()));
    }

    #[test]
    fn default_config_bypass_apps() {
        let config = Config::default();
        assert!(config.app_state.bypass_apps.contains(&"kitty".to_string()));
        assert!(config
            .app_state
            .bypass_apps
            .contains(&"alacritty".to_string()));
    }

    #[test]
    fn default_config_vietnamese_apps() {
        let config = Config::default();
        assert!(config
            .app_state
            .vietnamese_apps
            .contains(&"telegram".to_string()));
        assert!(config
            .app_state
            .vietnamese_apps
            .contains(&"firefox".to_string()));
    }

    #[test]
    fn parse_auto_restore_config() {
        let toml = r#"
[auto_restore]
enabled = false
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(!config.auto_restore.enabled);
    }

    #[test]
    fn parse_invalid_toml_fails() {
        let toml = "this is not valid toml {{{";
        let result = toml::from_str::<Config>(toml);
        assert!(result.is_err());
    }

    #[test]
    fn parse_unknown_fields_ignored() {
        let toml = r#"
input_method = "telex"
unknown_field = "value"
"#;
        // serde's default deny_unknown_fields is not set, so this should work
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.input_method, "telex");
    }

    #[test]
    fn parse_password_detection() {
        let toml = r#"
[password_detection]
enabled = true
check_atspi2 = true
check_window_title = true
title_keywords = ["password", "passphrase"]
password_apps = ["pinentry", "kwallet"]
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert!(config.password_detection.enabled);
        assert!(config.password_detection.check_atspi2);
        assert_eq!(config.password_detection.title_keywords, vec!["password", "passphrase"]);
        assert_eq!(config.password_detection.password_apps, vec!["pinentry", "kwallet"]);
    }

    #[test]
    fn parse_toggle_method_key() {
        let toml = r#"
toggle_method_key = "shift"
"#;
        let config: Config = toml::from_str(toml).unwrap();
        assert_eq!(config.toggle_method_key, "shift");
    }
}
