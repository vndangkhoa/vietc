// SPDX-License-Identifier: MIT
//! Distro detection and backend selection.
//! Auto-detects the current distro and selects appropriate
//! clipboard, device, and display backends for integration tests.

#[derive(Debug, Clone, PartialEq)]
pub enum DisplayServer {
    X11,
    Wayland,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct DistroInfo {
    pub id: String,
    pub version: String,
    pub display_server: DisplayServer,
    pub desktop: String,
}

/// Detect the current distro from /etc/os-release.
pub fn detect_distro() -> DistroInfo {
    let (id, version) = if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        let mut id = String::from("unknown");
        let mut version = String::from("unknown");
        for line in content.lines() {
            if let Some(val) = line.strip_prefix("ID=") {
                id = val.trim_matches('"').to_string();
            }
            if let Some(val) = line.strip_prefix("VERSION_ID=") {
                version = val.trim_matches('"').to_string();
            }
        }
        (id, version)
    } else {
        ("unknown".to_string(), "unknown".to_string())
    };

    let display_server = match std::env::var("XDG_SESSION_TYPE").ok().as_deref() {
        Some("x11") => DisplayServer::X11,
        Some("wayland") => DisplayServer::Wayland,
        _ => {
            if std::env::var("WAYLAND_DISPLAY").is_ok() {
                DisplayServer::Wayland
            } else if std::env::var("DISPLAY").is_ok() {
                DisplayServer::X11
            } else {
                DisplayServer::Unknown
            }
        }
    };

    let desktop = std::env::var("XDG_CURRENT_DESKTOP")
        .unwrap_or_else(|_| String::from("unknown"));

    DistroInfo {
        id,
        version,
        display_server,
        desktop,
    }
}

/// Check if the current distro is Debian-based (Ubuntu, Mint, Pop, etc.).
pub fn is_debian_based(info: &DistroInfo) -> bool {
    matches!(info.id.as_str(), "ubuntu" | "linuxmint" | "debian" | "pop" | "neon" | "zorin" | "elementary")
}

/// Check if the current distro is Arch-based.
pub fn is_arch_based(info: &DistroInfo) -> bool {
    matches!(info.id.as_str(), "arch" | "manjaro" | "cachyos" | "endeavouros" | "garuda" | "artix")
}

/// Check if the current distro is Fedora/RHEL-based.
pub fn is_fedora_based(info: &DistroInfo) -> bool {
    matches!(info.id.as_str(), "fedora" | "rhel" | "centos")
}
