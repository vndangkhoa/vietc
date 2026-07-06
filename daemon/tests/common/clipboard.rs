// SPDX-License-Identifier: MIT
//! Clipboard backend: reads the system clipboard to verify daemon output.
//! Automatically selects xclip (X11) or wl-paste (Wayland).

use std::time::Duration;

/// Read the current clipboard content. Returns None if no clipboard tool
/// is available or the clipboard is empty.
pub fn read_clipboard() -> Option<String> {
    let is_wayland = std::env::var("WAYLAND_DISPLAY").ok()?.contains("wayland");
    let (prog, args): (&str, &[&str]) = if is_wayland {
        ("wl-paste", &["-n"])
    } else {
        ("xclip", &["-selection", "clipboard", "-o"])
    };

    let output = std::process::Command::new(prog)
        .args(args)
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() { None } else { Some(text) }
}

/// Wait for clipboard to contain the expected text, with a timeout.
pub fn wait_for_clipboard(expected: &str, timeout: Duration) -> bool {
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if let Some(content) = read_clipboard() {
            if content == expected || content.contains(expected) {
                return true;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }
    eprintln!(
        "[test] Clipboard timeout: expected '{}', got '{:?}'",
        expected.escape_default(),
        read_clipboard().unwrap_or_default().escape_default()
    );
    false
}

/// Clear the clipboard (set to empty string).
pub fn clear_clipboard() -> bool {
    let is_wayland = std::env::var("WAYLAND_DISPLAY").ok().map_or(false, |v| v.contains("wayland"));
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
