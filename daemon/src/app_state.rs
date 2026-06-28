// SPDX-License-Identifier: MIT
use std::collections::HashMap;
use std::fs;
use std::process::Command;

/// Detect the currently focused window's class name
pub fn get_focused_window_class() -> Option<String> {
    // Try Wayland first (wlr-foreign-toplevel)
    if let Some(class) = get_wayland_window_class() {
        return Some(class);
    }

    // Try X11 via xdotool
    if let Some(class) = get_x11_window_class() {
        return Some(class);
    }

    // Fallback: try reading from /proc
    if let Some(class) = get_proc_window_class() {
        return Some(class);
    }

    None
}

fn get_x11_window_class() -> Option<String> {
    let output = Command::new("xdotool")
        .args(["getactivewindow", "getwindowclassname"])
        .output()
        .ok()?;

    if output.status.success() {
        let class = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !class.is_empty() {
            return Some(class.to_lowercase());
        }
    }

    None
}

fn get_wayland_window_class() -> Option<String> {
    // Try wlr-foreign-toplevel-management protocol via wlrctl
    let output = Command::new("wlrctl")
        .args(["toplevel", "list", "--format", "%app-id"])
        .output()
        .ok()?;

    if output.status.success() {
        let lines = String::from_utf8_lossy(&output.stdout);
        // First line is typically the focused window
        if let Some(class) = lines.lines().next() {
            let class = class.trim().to_string();
            if !class.is_empty() {
                return Some(class.to_lowercase());
            }
        }
    }

    None
}

fn get_proc_window_class() -> Option<String> {
    // Read /proc/active-windows if available (some compositors expose this)
    let content = fs::read_to_string("/proc/active-windows").ok()?;
    // Format: pid window_class window_title
    content
        .lines()
        .next()?
        .split_whitespace()
        .nth(1)
        .map(|s| s.to_lowercase())
}

/// Manages per-app IME state
pub struct AppStateManager {
    /// Current app class (lowercase)
    current_app: String,
    /// Per-app overrides (user toggled manually)
    overrides: HashMap<String, bool>,
    /// Default English apps from config
    english_apps: Vec<String>,
    /// Default Vietnamese apps from config
    vietnamese_apps: Vec<String>,
    /// Bypass apps from config
    bypass_apps: Vec<String>,
    /// Global enabled state
    global_enabled: bool,
}

impl AppStateManager {
    pub fn new(
        english_apps: Vec<String>,
        vietnamese_apps: Vec<String>,
        bypass_apps: Vec<String>,
        global_enabled: bool,
    ) -> Self {
        Self {
            current_app: String::new(),
            overrides: HashMap::new(),
            english_apps: english_apps.iter().map(|s| s.to_lowercase()).collect(),
            vietnamese_apps: vietnamese_apps.iter().map(|s| s.to_lowercase()).collect(),
            bypass_apps: bypass_apps.iter().map(|s| s.to_lowercase()).collect(),
            global_enabled,
        }
    }

    /// Check if focused app changed with a pre-detected class and return whether engine should be enabled
    pub fn update_with_app(&mut self, new_class: String) -> Option<bool> {
        if new_class == self.current_app {
            return None; // No change
        }

        let old_app = self.current_app.clone();
        self.current_app = new_class;

        eprintln!("[vietc] App: {} → {}", old_app, self.current_app);

        let should_enable = self.get_default_state();
        Some(should_enable)
    }

    /// Get the default Vietnamese state for the current app
    fn get_default_state(&self) -> bool {
        if !self.global_enabled {
            return false;
        }

        // Check user override first
        if let Some(&override_state) = self.overrides.get(&self.current_app) {
            return override_state;
        }

        // Check config defaults
        for pattern in &self.english_apps {
            if self.current_app.contains(pattern.as_str()) {
                return false;
            }
        }

        for pattern in &self.vietnamese_apps {
            if self.current_app.contains(pattern.as_str()) {
                return true;
            }
        }

        // Default: enabled
        true
    }

    /// Toggle the IME state for the current app (manual override)
    pub fn toggle_current_app(&mut self) -> bool {
        let current_state = self.get_default_state();
        let new_state = !current_state;
        self.overrides.insert(self.current_app.clone(), new_state);
        eprintln!(
            "[vietc] {} → {} (manual override)",
            self.current_app,
            if new_state { "Vietnamese" } else { "English" }
        );
        if let Err(e) = self.save_overrides() {
            eprintln!("[vietc] Failed to save app overrides: {}", e);
        }
        new_state
    }

    /// Clear all overrides
    #[allow(dead_code)]
    pub fn clear_overrides(&mut self) {
        self.overrides.clear();
        eprintln!("[vietc] All app overrides cleared");
    }

    /// Update app lists from reloaded config
    pub fn update_lists(
        &mut self,
        english_apps: Vec<String>,
        vietnamese_apps: Vec<String>,
        bypass_apps: Vec<String>,
    ) -> &Self {
        self.english_apps = english_apps.iter().map(|s| s.to_lowercase()).collect();
        self.vietnamese_apps = vietnamese_apps.iter().map(|s| s.to_lowercase()).collect();
        self.bypass_apps = bypass_apps.iter().map(|s| s.to_lowercase()).collect();
        eprintln!(
            "[vietc] App lists updated: {} English, {} Vietnamese, {} Bypass",
            self.english_apps.len(),
            self.vietnamese_apps.len(),
            self.bypass_apps.len()
        );
        self
    }

    /// Check if the currently active application should bypass the IME completely
    pub fn is_current_app_bypassed(&self) -> bool {
        for pattern in &self.bypass_apps {
            if self.current_app.contains(pattern.as_str()) {
                return true;
            }
        }
        false
    }

    /// Save overrides to config file
    #[allow(dead_code)]
    pub fn save_overrides(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = override_path();
        let content = toml::to_string(&self.overrides)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, content)?;
        Ok(())
    }

    /// Load overrides from config file
    pub fn load_overrides(&mut self) {
        let path = override_path();
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(overrides) = toml::from_str::<HashMap<String, bool>>(&content) {
                self.overrides = overrides;
                eprintln!("[vietc] Loaded {} app overrides", self.overrides.len());
            }
        }
    }

    #[allow(dead_code)]
    pub fn current_app(&self) -> &str {
        &self.current_app
    }
}

fn override_path() -> std::path::PathBuf {
    std::env::var("XDG_CONFIG_HOME")
        .ok()
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var("HOME")
                .ok()
                .map(|h| std::path::PathBuf::from(h).join(".config"))
        })
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("vietc")
        .join("overrides.toml")
}
