// SPDX-License-Identifier: MIT
//
// Thin wrapper that switches the active IBus engine by shelling out to
// `ibus engine <name>`. The target engine is cached so we never issue
// redundant switches (e.g. on every focus poll).

use std::process::Command;
use std::sync::Mutex;

/// Engine used for Vietnamese composition (Bamboo, configured for VNI).
pub const VN_ENGINE: &str = "Bamboo";
/// Engine used when Vietnamese should be off (Bamboo's English layout).
pub const EN_ENGINE: &str = "BambooUs";

static CURRENT: Mutex<Option<String>> = Mutex::new(None);

/// Switch the active IBus engine. No-op if `name` is already active.
///
/// `ibus engine <name>` sometimes returns a non-zero exit status even when the
/// switch succeeds, so we verify by reading the active engine back.
pub fn set_ibus_engine(name: &str) {
    let mut cur = match CURRENT.lock() {
        Ok(g) => g,
        Err(_) => return,
    };
    if cur.as_deref() == Some(name) {
        return;
    }

    let _ = Command::new("ibus").arg("engine").arg(name).status();

    let active = Command::new("ibus")
        .arg("engine")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    if active == name {
        *cur = Some(name.to_string());
        eprintln!("[vietc] IBus engine -> {}", name);
    } else {
        eprintln!(
            "[vietc] failed to switch IBus engine to '{}' (active: '{}')",
            name, active
        );
    }
}

/// Forget the cached engine (call after an external engine change so the next
/// `set_ibus_engine` always applies).
pub fn reset_cache() {
    if let Ok(mut cur) = CURRENT.lock() {
        *cur = None;
    }
}
