// SPDX-License-Identifier: MIT
use std::collections::HashMap;
use std::fs;
use std::process::Command;

use crate::password_detector::PasswordDetector;

/// Query _NET_ACTIVE_WINDOW directly via X11 client library (dlopen).
/// Works inside the Flatpak sandbox where xdotool/xprop are unavailable
/// but libX11.so.6 is present in the GNOME runtime.  No external process
/// or subclassing needed — open display, query property, return hex ID.
fn get_active_window_x11_dlopen() -> Option<String> {
    unsafe {
        let lib = libc::dlopen(
            b"libX11.so.6\0".as_ptr() as *const libc::c_char,
            libc::RTLD_LAZY,
        );
        if lib.is_null() {
            return None;
        }

        type FnOpenDisplay =
            unsafe extern "C" fn(*const libc::c_char) -> *mut libc::c_void;
        type FnDefaultRoot =
            unsafe extern "C" fn(*mut libc::c_void) -> u64;
        type FnInternAtom = unsafe extern "C" fn(
            *mut libc::c_void, *const libc::c_char, libc::c_int,
        ) -> u64;
        type FnGetProperty = unsafe extern "C" fn(
            *mut libc::c_void, u64, u64, u64, u64, u64, libc::c_int,
            *mut u64, *mut libc::c_int, *mut u64, *mut u64,
            *mut *mut u8,
        ) -> libc::c_int;
        type FnFree = unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int;
        type FnCloseDisplay =
            unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int;

        macro_rules! dlsym_fn {
            ($lib:expr, $name:literal) => {
                std::mem::transmute::<*mut libc::c_void, _>(libc::dlsym(
                    $lib,
                    concat!($name, "\0").as_ptr() as *const libc::c_char,
                ))
            };
        }

        let xopen: FnOpenDisplay = dlsym_fn!(lib, "XOpenDisplay");
        let xroot: FnDefaultRoot = dlsym_fn!(lib, "XDefaultRootWindow");
        let xatom: FnInternAtom = dlsym_fn!(lib, "XInternAtom");
        let xgetprop: FnGetProperty = dlsym_fn!(lib, "XGetProperty");
        let xfree: FnFree = dlsym_fn!(lib, "XFree");
        let xclosedpy: FnCloseDisplay = dlsym_fn!(lib, "XCloseDisplay");

        let dpy = xopen(std::ptr::null());
        if dpy.is_null() {
            libc::dlclose(lib);
            return None;
        }

        let root = xroot(dpy);
        let net_active = xatom(
            dpy,
            b"_NET_ACTIVE_WINDOW\0".as_ptr() as *const libc::c_char,
            0,
        );

        // XA_WINDOW = 33 (the standard X11 atom for Window type)
        let xa_window: u64 = 33;
        let mut actual_type: u64 = 0;
        let mut actual_format: libc::c_int = 0;
        let mut nitems: u64 = 0;
        let mut bytes_after: u64 = 0;
        let mut data: *mut u8 = std::ptr::null_mut();

        let status = xgetprop(
            dpy,
            root,
            net_active,
            xa_window,
            0,    // offset
            1,    // length
            0,    // delete
            &mut actual_type,
            &mut actual_format,
            &mut nitems,
            &mut bytes_after,
            &mut data,
        );

        let result = if status != 0
            && !data.is_null()
            && nitems > 0
            && actual_format == 32
        {
            // Format=32 elements are returned as unsigned long arrays
            let id = *(data as *const u64);
            if id != 0 {
                Some(format!("0x{:x}", id))
            } else {
                None
            }
        } else {
            None
        };

        if !data.is_null() {
            xfree(data as *mut libc::c_void);
        }
        xclosedpy(dpy);
        libc::dlclose(lib);

        result
    }
}

/// Get the active window's title (lowercase)
pub fn get_active_window_title() -> Option<String> {
    // Try GNOME Shell D-Bus (Wayland GNOME)
    if let Some(title) = get_gnome_window_title() {
        return Some(title.to_lowercase());
    }

    // Try X11 via xdotool
    if let Ok(output) = Command::new("xdotool")
        .args(["getactivewindow", "getwindowname"])
        .output()
    {
        if output.status.success() {
            let title = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !title.is_empty() {
                return Some(title.to_lowercase());
            }
        }
    }

    // Try X11 via xprop/wmctrl (fallback when xdotool not installed)
    if let Some(title) = get_wmctrl_window_title() {
        return Some(title);
    }

    None
}

/// Query GNOME Shell via D-Bus for the focused window's title
fn get_gnome_window_title() -> Option<String> {
    let js = "global.display.focus_window?.get_title() ?? ''";
    let (_, title) = gnome_shell_eval(js)?;
    if title.is_empty() { None } else { Some(title) }
}

/// Get the active window's X11 ID (unique per window — even within the same
/// application).  Returns a unique window-identifier string.
pub fn get_active_window_id() -> Option<String> {
    // Try GNOME Shell D-Bus (Wayland GNOME) — returns hex window ID
    if let Some(id) = get_gnome_active_window_id() {
        return Some(id);
    }

    // Try xdotool first (fast, direct, X11)
    if let Ok(output) = Command::new("xdotool")
        .args(["getactivewindow"])
        .output()
    {
        if output.status.success() {
            let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !id.is_empty() {
                return Some(id);
            }
        }
    }

    // Fallback: xprop -root _NET_ACTIVE_WINDOW (x11-utils, preinstalled on most distros)
    if let Ok(output) = Command::new("xprop")
        .args(["-root", "_NET_ACTIVE_WINDOW"])
        .output()
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Format: "_NET_ACTIVE_WINDOW(WINDOW): window id # 0x3a00004"
            if let Some(hex) = stdout.split("window id # ").nth(1) {
                let hex = hex.trim();
                if !hex.is_empty() {
                    return Some(hex.to_string());
                }
            }
        }
    }

    // Final fallback: direct X11 client library query (works in Flatpak sandbox)
    if let Some(id) = get_active_window_x11_dlopen() {
        return Some(id);
    }

    None
}

/// Query GNOME Shell via D-Bus for the focused window's XID
fn get_gnome_active_window_id() -> Option<String> {
    let js = "global.display.focus_window?.get_id()?.toString(16) ?? ''";
    let (_, id) = gnome_shell_eval(js)?;
    if id.is_empty() { None } else { Some(format!("0x{}", id)) }
}

/// Detect the currently focused window's class name
pub fn get_focused_window_class() -> Option<String> {
    // Try GNOME Shell D-Bus (Wayland GNOME)
    if let Some(class) = get_gnome_focused_wm_class() {
        return Some(class);
    }

    // Try Wayland via wlrctl (wlroots compositors)
    if let Some(class) = get_wayland_window_class() {
        return Some(class);
    }

    // Try X11 via xdotool
    if let Some(class) = get_x11_window_class() {
        return Some(class);
    }

    // Try X11 via xprop (fallback when xdotool is not installed)
    if let Some(class) = get_xprop_window_class() {
        return Some(class);
    }

    // Try X11 via wmctrl (fallback)
    if let Some(class) = get_wmctrl_window_class() {
        return Some(class);
    }

    // Fallback: try reading from /proc
    if let Some(class) = get_proc_window_class() {
        return Some(class);
    }

    None
}

/// Query GNOME Shell via D-Bus for the focused window's WM class (app ID)
fn get_gnome_focused_wm_class() -> Option<String> {
    let js = "global.display.focus_window?.get_wm_class() ?? ''";
    let (_, result) = gnome_shell_eval(js)?;
    if result.is_empty() { None } else { Some(result.to_lowercase()) }
}

/// Execute JavaScript in GNOME Shell and return (success, output)
fn gnome_shell_eval(js: &str) -> Option<(bool, String)> {
    use std::time::Duration;
    let conn = dbus::blocking::Connection::new_session().ok()?;
    let proxy = dbus::blocking::Proxy::new(
        "org.gnome.Shell",
        "/org/gnome/Shell",
        Duration::from_secs(1),
        &conn,
    );
    let (success, output): (bool, String) = proxy
        .method_call("org.gnome.Shell", "Eval", (js,))
        .ok()?;
    Some((success, output))
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

/// Get WM_CLASS via xprop (works on X11 without xdotool)
fn get_xprop_window_class() -> Option<String> {
    // First get the active window ID
    let id = get_active_window_id_xprop()?;
    // Then get WM_CLASS for that window
    let output = Command::new("xprop")
        .args(["-id", &id, "WM_CLASS"])
        .output()
        .ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Format: WM_CLASS(STRING) = "gnome-terminal", "Gnome-terminal"
        // We want the first string (application name)
        if let Some(class_part) = stdout.split('"').nth(1) {
            let class = class_part.trim().to_lowercase();
            if !class.is_empty() {
                return Some(class);
            }
        }
    }
    None
}

/// Get active window ID via xprop (X11, no xdotool needed)
fn get_active_window_id_xprop() -> Option<String> {
    let output = Command::new("xprop")
        .args(["-root", "_NET_ACTIVE_WINDOW"])
        .output()
        .ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Format: "_NET_ACTIVE_WINDOW(WINDOW): window id # 0x3a00004"
        if let Some(hex) = stdout.split("window id # ").nth(1) {
            let hex = hex.trim();
            if !hex.is_empty() {
                return Some(hex.to_string());
            }
        }
    }
    None
}

/// Get window class via wmctrl (X11 fallback)
fn get_wmctrl_window_class() -> Option<String> {
    // Only try wmctrl if xdotool is unavailable
    if Command::new("xdotool").output().is_ok() {
        return None; // xdotool exists, prefer it
    }
    // wmctrl -l -x lists windows with their class (WM_CLASS)
    let output = Command::new("wmctrl")
        .args(["-l", "-x"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Each line format: 0x00a00001  desktop_num  class_name  title
    // Class name format: "gnome-terminal.Gnome-terminal"
    // We want the part before the dot
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let class_part = parts[2];
            // Get the app part before the dot if present
            let class = class_part.split('.').next().unwrap_or(class_part).to_lowercase();
            if !class.is_empty() && class != "nvidia-settings" {
                // wmctrl returns ALL windows, not just focused.
                // We check if the first listed window is focused,
                // or if there's an active-state marker.
                return Some(class);
            }
        }
    }
    None
}

/// Get active window title using wmctrl
fn get_wmctrl_window_title() -> Option<String> {
    let id = get_active_window_id_xprop()?;
    let output = Command::new("xprop")
        .args(["-id", &id, "WM_NAME"])
        .output()
        .ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Format: "WM_NAME(STRING) = "title""
        if let Some(title) = stdout.split('"').nth(1) {
            let title = title.trim();
            if !title.is_empty() {
                return Some(title.to_lowercase());
            }
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
    /// Password detection config
    password_enabled: bool,
    check_atspi2: bool,
    check_window_title: bool,
    title_keywords: Vec<String>,
    password_apps: Vec<String>,
    /// Password detector (AT-SPI2)
    password_detector: PasswordDetector,
    /// Cached password field state
    is_password_field: bool,
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
            password_enabled: false,
            check_atspi2: true,
            check_window_title: true,
            title_keywords: Vec::new(),
            password_apps: Vec::new(),
            password_detector: PasswordDetector::new(),
            is_password_field: false,
        }
    }

    /// Update password detection config
    pub fn set_password_config(
        &mut self,
        enabled: bool,
        check_atspi2: bool,
        check_window_title: bool,
        title_keywords: Vec<String>,
        password_apps: Vec<String>,
    ) {
        self.password_enabled = enabled;
        self.check_atspi2 = check_atspi2;
        self.check_window_title = check_window_title;
        self.title_keywords = title_keywords.iter().map(|s| s.to_lowercase()).collect();
        self.password_apps = password_apps.iter().map(|s| s.to_lowercase()).collect();
    }

    /// Check if the current focused widget is a password field
    /// Returns true if password detected, forcing English mode
    pub fn check_password_field(&mut self) -> bool {
        if !self.password_enabled {
            self.is_password_field = false;
            return false;
        }

        // Layer 1: AT-SPI2 (most accurate, works in terminals and dialogs)
        if self.check_atspi2 {
            if let Some(is_password) = self.password_detector.check() {
                self.is_password_field = is_password;
                if is_password {
                    log_password_detection("AT-SPI2", &self.current_app);
                }
                return is_password;
            }
        }

        // Layer 2: Window class match (for known password dialogs)
        for pattern in &self.password_apps {
            if self.current_app.contains(pattern.as_str()) {
                self.is_password_field = true;
                log_password_detection("window-class", &self.current_app);
                return true;
            }
        }

        // Layer 3: Window title heuristic (for sudo prompts, browser dialogs)
        if self.check_window_title {
            if let Some(title) = get_active_window_title() {
                for keyword in &self.title_keywords {
                    if title.contains(keyword.as_str()) {
                        self.is_password_field = true;
                        log_password_detection("window-title", &title);
                        return true;
                    }
                }
            }
        }

        // Layer 4: Process-based detection (for terminal sudo/passwd prompts)
        if let Some(pid) = get_active_window_pid() {
            if is_sudo_process(pid) {
                self.is_password_field = true;
                log_password_detection("process-sudo", &format!("PID {}", pid));
                return true;
            }
        }

        self.is_password_field = false;
        false
    }

    /// Is the current widget a password field? (cached)
    pub fn is_password_field(&self) -> bool {
        self.is_password_field
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
    pub fn get_default_state(&self) -> bool {
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

fn log_password_detection(method: &str, context: &str) {
    eprintln!("[vietc] Password field detected via {}: {}", method, context);
}

/// Get the PID of the active window via xprop
fn get_active_window_pid() -> Option<u32> {
    let id = get_active_window_id_xprop()?;
    // Some terminals (gnome-terminal) don't have _NET_WM_PID directly
    // Try xprop first
    let output = Command::new("xprop")
        .args(["-id", &id, "_NET_WM_PID"])
        .output()
        .ok()?;
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // Format: _NET_WM_PID(CARDINAL) = 12345
        if let Some(pid_str) = stdout.split("= ").nth(1) {
            if let Ok(pid) = pid_str.trim().parse::<u32>() {
                if pid > 0 {
                    return Some(pid);
                }
            }
        }
    }
    None
}

/// Check if the given PID or any of its children is running sudo/passwd
fn is_sudo_process(pid: u32) -> bool {
    // Check the process itself
    if let Ok(output) = Command::new("ps")
        .args(["-o", "comm=", "-p", &pid.to_string()])
        .output()
    {
        let comm = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if comm == "sudo" || comm == "passwd" || comm == "pkexec" {
            return true;
        }
    }

    // Check child processes recursively (depth = 2)
    if let Ok(output) = Command::new("ps")
        .args(["--ppid", &pid.to_string(), "-o", "comm="])
        .output()
    {
        let output = String::from_utf8_lossy(&output.stdout);
        for line in output.lines() {
            let comm = line.trim();
            if comm == "sudo" || comm == "passwd" || comm == "pkexec" {
                return true;
            }
        }
    }

    // Check grandchild processes (depth = 3)
    if let Ok(output) = Command::new("ps")
        .args(["--ppid", &pid.to_string(), "-o", "pid="])
        .output()
    {
        let output = String::from_utf8_lossy(&output.stdout);
        for line in output.lines() {
            let child_pid = line.trim();
            if child_pid.is_empty() { continue; }
            if let Ok(output) = Command::new("ps")
                .args(["--ppid", child_pid, "-o", "comm="])
                .output()
            {
                let output = String::from_utf8_lossy(&output.stdout);
                for line in output.lines() {
                    let comm = line.trim();
                    if comm == "sudo" || comm == "passwd" || comm == "pkexec" {
                        return true;
                    }
                }
            }
        }
    }

    false
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
