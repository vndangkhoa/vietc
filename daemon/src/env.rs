use std::fs;
use std::path::Path;

use crate::log::log_info;

/// Recover the user's display environment (DISPLAY / XAUTHORITY /
/// WAYLAND_DISPLAY / XDG_RUNTIME_DIR) into the daemon's own environment.
///
/// vietc is normally launched by the user's session (systemd --user + tray,
/// possibly via setcap) but can also be launched with `sudo`. In either case
/// the daemon's environment may be missing WAYLAND_DISPLAY, which makes the
/// clipboard paste (`wl-copy`) fall back to `xclip` — and on a pure Wayland
/// session with no X server that silently drops all Vietnamese (Unicode)
/// output. We recover the variables from the owning user's processes in /proc.
pub fn recover_display_env() {
    // X11: if DISPLAY is already present we're fine, just make sure DBUS works.
    if let Ok(d) = std::env::var("DISPLAY") {
        if !d.is_empty() {
            recover_dbus_env();
            return;
        }
    }

    let target_uid: u32 = std::env::var("SUDO_UID")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or_else(|| unsafe { libc::getuid() });

    if let Ok(entries) = fs::read_dir("/proc") {
        'outer: for entry in entries.flatten() {
            let name_s = entry.file_name().to_string_lossy().into_owned();
            if !name_s.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            #[cfg(target_os = "linux")]
            {
                use std::os::linux::fs::MetadataExt;
                if let Ok(meta) = entry.metadata() {
                    if meta.st_uid() != target_uid {
                        continue;
                    }
                }
            }
            let environ_path = entry.path().join("environ");
            if let Ok(content) = fs::read(&environ_path) {
                let mut display = None;
                let mut xauth = None;
                let mut wayland_display = None;
                let mut xdg_runtime_dir = None;
                for chunk in content.split(|&b| b == 0) {
                    if let Ok(s) = std::str::from_utf8(chunk) {
                        if let Some(v) = s.strip_prefix("DISPLAY=") {
                            if !v.is_empty() {
                                display = Some(v.to_string());
                            }
                        }
                        if let Some(v) = s.strip_prefix("XAUTHORITY=") {
                            xauth = Some(v.to_string());
                        }
                        if let Some(v) = s.strip_prefix("WAYLAND_DISPLAY=") {
                            if !v.is_empty() {
                                wayland_display = Some(v.to_string());
                            }
                        }
                        if let Some(v) = s.strip_prefix("XDG_RUNTIME_DIR=") {
                            if !v.is_empty() {
                                xdg_runtime_dir = Some(v.to_string());
                            }
                        }
                    }
                }
                if let Some(ref d) = display {
                    std::env::set_var("DISPLAY", d);
                    if let Some(ref x) = xauth {
                        std::env::set_var("XAUTHORITY", x);
                    }
                    log_info(&format!("[vietc] Recovered DISPLAY={} from /proc", d));
                }
                if let Some(ref w) = wayland_display {
                    std::env::set_var("WAYLAND_DISPLAY", w);
                    log_info(&format!("[vietc] Recovered WAYLAND_DISPLAY={} from /proc", w));
                }
                if let Some(ref r) = xdg_runtime_dir {
                    std::env::set_var("XDG_RUNTIME_DIR", r);
                    log_info(&format!("[vietc] Recovered XDG_RUNTIME_DIR={} from /proc", r));
                }
                if display.is_some() || wayland_display.is_some() {
                    break 'outer;
                }
            }
        }
    }

    // Fall back to the standard Wayland socket name inside the user's runtime
    // dir when WAYLAND_DISPLAY wasn't explicit but the socket exists.
    if std::env::var("WAYLAND_DISPLAY").map(|v| v.is_empty()).unwrap_or(true) {
        if let Ok(x) = std::env::var("XDG_RUNTIME_DIR") {
            let sock = Path::new(&x).join("wayland-0");
            if sock.exists() {
                std::env::set_var("WAYLAND_DISPLAY", "wayland-0");
                log_info("[vietc] Recovered WAYLAND_DISPLAY=wayland-0 from XDG_RUNTIME_DIR");
            }
        }
    }

    recover_dbus_env();
}

pub fn recover_dbus_env() {
    if let Ok(d) = std::env::var("DBUS_SESSION_BUS_ADDRESS") {
        if !d.is_empty() {
            return;
        }
    }
    let target_uid: u32 = std::env::var("SUDO_UID")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or_else(|| unsafe { libc::getuid() });

    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let name_s = entry.file_name().to_string_lossy().into_owned();
            if !name_s.chars().all(|c| c.is_ascii_digit()) {
                continue;
            }
            #[cfg(target_os = "linux")]
            {
                use std::os::linux::fs::MetadataExt;
                if let Ok(meta) = entry.metadata() {
                    if meta.st_uid() != target_uid {
                        continue;
                    }
                }
            }
            let environ_path = entry.path().join("environ");
            if let Ok(content) = fs::read(&environ_path) {
                if let Ok(dbus_env) = std::str::from_utf8(&content) {
                    for var in dbus_env.split('\0') {
                        if let Some(val) = var.strip_prefix("DBUS_SESSION_BUS_ADDRESS=") {
                            if !val.is_empty() {
                                std::env::set_var("DBUS_SESSION_BUS_ADDRESS", val);
                                log_info("[vietc] Recovered DBUS_SESSION_BUS_ADDRESS");
                                return;
                            }
                        }
                        if let Some(val) = var.strip_prefix("XDG_RUNTIME_DIR=") {
                            if !val.is_empty() {
                                std::env::set_var("XDG_RUNTIME_DIR", val);
                            }
                        }
                    }
                }
            }
        }
    }
    if let Ok(xdg_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let bus_path = Path::new(&xdg_dir).join("bus");
        if bus_path.exists() {
            let addr = format!("unix:path={}", bus_path.display());
            std::env::set_var("DBUS_SESSION_BUS_ADDRESS", &addr);
            log_info("[vietc] Set DBUS_SESSION_BUS_ADDRESS from XDG_RUNTIME_DIR/bus");
        }
    }
}
