// SPDX-License-Identifier: MIT
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayServer {
    Wayland,
    X11,
    Unknown,
}

/// Detect whether we're running on Wayland or X11
pub fn detect_display_server() -> DisplayServer {
    // Check WAYLAND_DISPLAY first
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        return DisplayServer::Wayland;
    }

    // Check XDG_SESSION_TYPE
    if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
        if session_type.contains("wayland") {
            return DisplayServer::Wayland;
        }
    }

    // Check if XDG_RUNTIME_DIR has wayland sockets
    if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
        let wayland_sock = std::path::Path::new(&xdg_runtime).join("wayland-0");
        if wayland_sock.exists() {
            return DisplayServer::Wayland;
        }
    }

    // Check DISPLAY variable
    if std::env::var("DISPLAY").is_ok() {
        return DisplayServer::X11;
    }

    // Try to detect via loginctl
    if let Ok(output) = Command::new("loginctl")
        .args(["show-session", &get_session_id(), "-p", "Type"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains("wayland") {
            return DisplayServer::Wayland;
        }
        if stdout.contains("x11") {
            return DisplayServer::X11;
        }
    }

    DisplayServer::Unknown
}

fn get_session_id() -> String {
    std::env::var("XDG_SESSION_ID").unwrap_or_else(|_| "self".into())
}

/// Check if a specific compositor is running
pub fn detect_compositor() -> Option<String> {
    // Check common Wayland compositor env vars
    let compositor_vars = [
        ("HYPRLAND_INSTANCE_SIGNATURE", "Hyprland"),
        ("SWAYSOCK", "Sway"),
        ("I3SOCK", "i3"),
        ("KWIN_SESSION", "KWin"),
    ];

    for (var, name) in &compositor_vars {
        if std::env::var(var).is_ok() {
            return Some(name.to_string());
        }
    }

    // Check via process name
    if let Ok(output) = Command::new("pgrep").arg("-x").arg("hyprland").output() {
        if output.status.success() {
            return Some("Hyprland".into());
        }
    }

    if let Ok(output) = Command::new("pgrep").arg("-x").arg("sway").output() {
        if output.status.success() {
            return Some("Sway".into());
        }
    }

    None
}
