// SPDX-License-Identifier: MIT
//! Viet+ integration test harness.

pub mod clipboard;
pub mod distro;
pub mod virtual_keyboard;

use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Check if the current process is running as root.
pub fn is_root() -> bool {
    unsafe { libc::geteuid() == 0 }
}

/// Create a temporary configuration for the daemon.
pub fn create_temp_config(method: &str, grab: bool, start_enabled: bool) -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("failed to create temp dir");
    let config_path = dir.path().join("config.toml");

    let grab_str = if grab { "true" } else { "false" };
    let enabled_str = if start_enabled { "true" } else { "false" };
    let toml = format!(
        r#"input_method = "{}"
toggle_key = "space"
start_enabled = {}
grab = {}

[app_state]
enabled = true
english_apps = ["code", "vim"]
vietnamese_apps = ["telegram", "discord", "firefox"]
"#,
        method, enabled_str, grab_str,
    );
    std::fs::write(&config_path, &toml).expect("failed to write temp config");
    eprintln!("[test] Temp config at {}", config_path.display());
    dir
}

/// Manages a daemon subprocess for integration testing.
pub struct DaemonProcess {
    child: Arc<Mutex<Child>>,
    log: Arc<Mutex<Vec<String>>>,
    _reader: std::thread::JoinHandle<()>,
}

impl DaemonProcess {
    /// Spawn the daemon with a config from the given directory.
    pub fn spawn(config_dir: &Path) -> Self {
        let config_path = config_dir.join("config.toml");
        let bin_path = daemon_binary_path();

        eprintln!("[test] Spawning daemon: {} config={}", bin_path.display(), config_path.display());

        let child = Command::new(&bin_path)
            .env("VIETC_CONFIG", config_path.to_str().unwrap())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to spawn daemon");

        let child = Arc::new(Mutex::new(child));
        let stderr = child.lock().unwrap().stderr.take()
            .expect("daemon stderr not captured");

        let log = Arc::new(Mutex::new(Vec::new()));
        let log_clone = log.clone();

        let reader = std::thread::spawn(move || {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                if let Ok(line) = line {
                    eprintln!("[daemon] {}", line);
                    if let Ok(mut guard) = log_clone.lock() {
                        guard.push(line);
                    }
                }
            }
        });

        Self { child, log, _reader: reader }
    }

    /// Read all log lines captured so far.
    pub fn logs(&self) -> Vec<String> {
        self.log.lock().unwrap().clone()
    }

    /// Wait for a specific pattern to appear in the daemon's log.
    pub fn wait_for_log(&self, pattern: &str, timeout: Duration) -> bool {
        let start = std::time::Instant::now();
        while start.elapsed() < timeout {
            if let Ok(guard) = self.log.lock() {
                if guard.iter().any(|l| l.contains(pattern)) {
                    return true;
                }
            }
            if !self.is_running() {
                return false;
            }
            std::thread::sleep(Duration::from_millis(20));
        }
        let last = self.logs().last().cloned().unwrap_or_default();
        eprintln!("[test] Timeout waiting for '{}'. Last log: '{}'", pattern, last);
        false
    }

    /// Check if the daemon is still running.
    pub fn is_running(&self) -> bool {
        let mut child = self.child.lock().unwrap();
        match child.try_wait() {
            Ok(Some(status)) => {
                eprintln!("[test] Daemon exited: {}", status);
                false
            }
            Ok(None) => true,
            Err(e) => {
                eprintln!("[test] Error checking daemon: {}", e);
                false
            }
        }
    }

    /// Kill the daemon and wait for it to exit.
    pub fn kill(self) {
        eprintln!("[test] Killing daemon...");
        let mut child = self.child.lock().unwrap();
        let _ = child.kill();
        let _ = child.wait();
        eprintln!("[test] Daemon killed");
    }
}

impl Drop for DaemonProcess {
    fn drop(&mut self) {
        let mut child = self.child.lock().unwrap();
        let _ = child.kill();
        let _ = child.wait();
    }
}

fn daemon_binary_path() -> PathBuf {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_vietc") {
        return PathBuf::from(path);
    }
    let mut path = std::env::current_dir().unwrap_or_default();
    if path.ends_with("daemon") {
        path.pop();
    }
    let debug = path.join("target").join("debug").join("vietc");
    if debug.exists() {
        return debug;
    }
    let release = path.join("target").join("release").join("vietc");
    if release.exists() {
        return release;
    }
    panic!("Daemon binary not found. Build first: cargo build");
}
