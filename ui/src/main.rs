// SPDX-License-Identifier: MIT
use std::path::PathBuf;

mod config;
mod tray;

fn exe_dir() -> PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("/usr/bin"))
}

fn find_sibling_binary(name: &str) -> String {
    let dir = exe_dir();
    // Try exact name (e.g. "vietc" outside Flatpak)
    let sibling = dir.join(name);
    if sibling.exists() {
        return sibling.to_string_lossy().into_owned();
    }
    // Try name-daemon (e.g. "vietc-daemon" inside Flatpak)
    let daemon = dir.join(format!("{}-daemon", name));
    if daemon.exists() {
        return daemon.to_string_lossy().into_owned();
    }
    name.to_string()
}

fn is_daemon_running() -> bool {
    // Check both "vietc" (outside Flatpak) and "vietc-daemon" (inside Flatpak)
    let check = |name: &str| -> bool {
        std::process::Command::new("pgrep")
            .arg("-x")
            .arg(name)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };
    check("vietc") || check("vietc-daemon")
}

fn needs_root() -> bool {
    let cfg = config::Config::load();
    cfg.grab
}

/// Show a password prompt using available desktop tools.
/// Returns the password, or empty string if cancelled.
fn prompt_password() -> String {
    let title = "Viet+";
    let msg = "Viet+ needs root privileges to capture keyboard input.\nPlease enter your password:";

    // Try zenity (GNOME)
    if let Ok(output) = std::process::Command::new("zenity")
        .args(["--password", "--title", title, "--text", msg])
        .stderr(std::process::Stdio::null())
        .output()
    {
        if output.status.success() {
            let pw = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !pw.is_empty() {
                return pw;
            }
        }
    }

    // Try kdialog (KDE)
    if let Ok(output) = std::process::Command::new("kdialog")
        .args(["--password", msg])
        .stderr(std::process::Stdio::null())
        .output()
    {
        if output.status.success() {
            let pw = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !pw.is_empty() {
                return pw;
            }
        }
    }

    // Try ssh-askpass (X11 fallback)
    if let Ok(output) = std::process::Command::new("ssh-askpass")
        .arg(msg)
        .stderr(std::process::Stdio::null())
        .output()
    {
        if output.status.success() {
            let pw = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !pw.is_empty() {
                return pw;
            }
        }
    }

    // Last resort: terminal prompt
    eprintln!("{}", msg);
    if let Ok(child) = std::process::Command::new("sh")
        .arg("-c")
        .arg("read -s -p 'Password: ' pw && echo \"$pw\"")
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::piped())
        .spawn()
    {
        if let Ok(output) = child.wait_with_output() {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }
    }

    String::new()
}

fn start_daemon() {
    let daemon_bin = find_sibling_binary("vietc");

    if needs_root() && !is_daemon_running() {
        // Mark that we've attempted first launch
        let flag_path = config_path().join(".first-launch-done");

        if !flag_path.exists() {
            let password = prompt_password();
            if password.is_empty() {
                eprintln!("[vietc-tray] No password provided, starting daemon without root");
                let _ = std::process::Command::new(&daemon_bin).spawn();
                return;
            }

            // Start daemon with sudo
            let mut child = match std::process::Command::new("sudo")
                .args(["-S", &daemon_bin])
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
            {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("[vietc-tray] Failed to start daemon with sudo: {}", e);
                    let _ = std::process::Command::new(&daemon_bin).spawn();
                    return;
                }
            };

            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                let _ = stdin.write_all(format!("{}\n", password).as_bytes());
            }
            let _ = child.wait();

            // Mark first launch as done
            let _ = std::fs::write(&flag_path, "1");
            return;
        }
    }

    if !is_daemon_running() {
        eprintln!("[vietc-tray] Starting daemon: {}", daemon_bin);
        let _ = std::process::Command::new(&daemon_bin).spawn();
    }
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("vietc")
}

fn main() {
    eprintln!("[vietc-tray] Starting");

    // Start daemon (with password prompt if first launch)
    start_daemon();

    // Run the tray
    tray::run();
}
