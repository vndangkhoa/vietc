// SPDX-License-Identifier: MIT
//
// vietcctl — small control tool for the Viet+ aux-controller (Bamboo) setup.
// Cycles / sets the input mode and persists it to ~/.config/vietc/mode.
//
//   vietcctl cycle          -> EN -> VNI -> TELEX -> EN (and apply)
//   vietcctl en|vn|vni|telex -> set a specific mode
//   vietcctl status         -> print the current mode
//
// Mode is applied by switching the active IBus engine (Bamboo / BambooUs) and,
// for VNI/TELEX, rewriting Bamboo's InputMethod and re-activating the engine so
// the change takes effect immediately.

use std::path::PathBuf;
use std::process::Command;

use serde::Deserialize;
use serde::Serialize;

const VN_ENGINE: &str = "Bamboo";
const EN_ENGINE: &str = "BambooUs";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    En,
    Vni,
    Telex,
}

impl Mode {
    fn from_str(s: &str) -> Mode {
        match s.trim().to_ascii_lowercase().as_str() {
            "en" | "english" => Mode::En,
            "telex" => Mode::Telex,
            _ => Mode::Vni,
        }
    }

    fn as_str(&self) -> &'static str {
        match self {
            Mode::En => "en",
            Mode::Vni => "vni",
            Mode::Telex => "telex",
        }
    }

    fn label(&self) -> &'static str {
        match self {
            Mode::En => "EN",
            Mode::Vni => "VNI",
            Mode::Telex => "TELEX",
        }
    }

    /// Advance EN -> VNI -> TELEX -> EN.
    fn next(&self) -> Mode {
        match self {
            Mode::En => Mode::Vni,
            Mode::Vni => Mode::Telex,
            Mode::Telex => Mode::En,
        }
    }
}

fn config_dir() -> PathBuf {
    dirs::config_dir().unwrap_or_else(|| PathBuf::from(".config"))
}

fn mode_file() -> PathBuf {
    config_dir().join("vietc").join("mode")
}

fn read_mode() -> Mode {
    std::fs::read_to_string(mode_file())
        .map(|s| Mode::from_str(&s))
        .unwrap_or(Mode::Vni)
}

fn write_mode(mode: Mode) {
    if let Some(parent) = mode_file().parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(mode_file(), mode.as_str());
}

fn bamboo_config_path() -> PathBuf {
    config_dir().join("ibus-bamboo").join("ibus-bamboo.config.json")
}

#[derive(Serialize, Deserialize)]
struct BambooConfig {
    #[serde(rename = "InputMethod", skip_serializing_if = "Option::is_none")]
    input_method: Option<String>,
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

fn set_bamboo_method(method: &str) {
    let path = bamboo_config_path();
    let mut cfg = if let Ok(content) = std::fs::read_to_string(&path) {
        serde_json::from_str::<BambooConfig>(&content).unwrap_or(BambooConfig {
            input_method: None,
            extra: std::collections::HashMap::new(),
        })
    } else {
        let _ = std::fs::create_dir_all(path.parent().unwrap());
        BambooConfig {
            input_method: None,
            extra: std::collections::HashMap::new(),
        }
    };
    cfg.input_method = Some(method.to_string());
    if let Ok(json) = serde_json::to_string_pretty(&cfg) {
        let _ = std::fs::write(&path, json);
    }
}

fn run_ibus(engine: &str) {
    let _ = Command::new("ibus")
        .args(["engine", engine])
        .status();
}

/// Apply a mode. For VNI/TELEX we rewrite Bamboo's InputMethod and
/// re-activate the engine (BambooUs -> Bamboo) so the new method loads.
fn apply(mode: Mode) {
    match mode {
        Mode::En => {
            run_ibus(EN_ENGINE);
        }
        Mode::Vni => {
            set_bamboo_method("VNI");
            run_ibus(EN_ENGINE); // deactivate Bamboo so the next line re-inits it
            run_ibus(VN_ENGINE);
        }
        Mode::Telex => {
            set_bamboo_method("TELEX");
            run_ibus(EN_ENGINE);
            run_ibus(VN_ENGINE);
        }
    }
    write_mode(mode);
    eprintln!("[vietcctl] mode -> {}", mode.label());
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let cmd = args.get(1).map(|s| s.as_str()).unwrap_or("cycle");

    match cmd {
        "status" => {
            println!("{}", read_mode().label());
        }
        "en" | "english" => apply(Mode::En),
        "vn" | "vni" => apply(Mode::Vni),
        "telex" => apply(Mode::Telex),
        "cycle" | "toggle" => {
            let next = read_mode().next();
            apply(next);
        }
        other => {
            eprintln!("Usage: vietcctl [cycle|en|vni|telex|status]");
            eprintln!("Unknown command: {}", other);
            std::process::exit(1);
        }
    }
}
