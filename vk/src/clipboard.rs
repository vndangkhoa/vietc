// SPDX-License-Identifier: MIT
//! Clipboard read/clear, Wayland-aware (wl-paste/wl-copy), X11 fallback (xclip).
//! Mirrors vietc's daemon/tests/common/clipboard.rs.

/// Read the current clipboard content. Returns None if no tool/empty.
pub fn read_clipboard() -> Option<String> {
    let is_wayland = std::env::var("WAYLAND_DISPLAY").ok()?.contains("wayland");
    let (prog, args): (&str, &[&str]) = if is_wayland {
        ("wl-paste", &["-n"])
    } else {
        ("xclip", &["-selection", "clipboard", "-o"])
    };
    let output = std::process::Command::new(prog).args(args).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

/// Clear the clipboard (set to empty string).
pub fn clear_clipboard() -> bool {
    let is_wayland = std::env::var("WAYLAND_DISPLAY")
        .is_ok_and(|v| v.contains("wayland"));
    if is_wayland {
        std::process::Command::new("wl-copy")
            .arg("")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        std::process::Command::new("xclip")
            .args(["-selection", "clipboard", "-i"])
            .stdin(std::process::Stdio::null())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}
