use std::fs;

use crate::log::log_info;

pub fn recover_display_env() {
    if unsafe { libc::getuid() } != 0 {
        return;
    }
    if let Ok(d) = std::env::var("DISPLAY") {
        if !d.is_empty() {
            recover_dbus_env();
            return;
        }
    }
    let target_uid: u32 = match std::env::var("SUDO_UID") {
        Ok(s) => match s.parse() {
            Ok(v) => v,
            Err(_) => return,
        },
        Err(_) => return,
    };
    if let Ok(entries) = fs::read_dir("/proc") {
        'outer: for entry in entries.flatten() {
            let name = entry.file_name();
            let name_s = name.to_string_lossy();
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
                    }
                }
                if let Some(d) = display {
                    std::env::set_var("DISPLAY", &d);
                    if let Some(x) = xauth {
                        std::env::set_var("XAUTHORITY", x);
                    }
                    log_info(&format!("[vietc] Recovered DISPLAY={} from /proc", d));
                    break 'outer;
                }
            }
        }
    }
    recover_dbus_env();
}

pub fn recover_dbus_env() {
    if unsafe { libc::getuid() } != 0 {
        return;
    }
    let target_uid: u32 = match std::env::var("SUDO_UID") {
        Ok(s) => match s.parse() {
            Ok(v) => v,
            Err(_) => return,
        },
        Err(_) => return,
    };

    if let Ok(entries) = fs::read_dir("/proc") {
        for entry in entries.flatten() {
            let name = entry.file_name();
            let name_s = name.to_string_lossy();
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
        let bus_path = std::path::Path::new(&xdg_dir).join("bus");
        if bus_path.exists() {
            let addr = format!("unix:path={}", bus_path.display());
            std::env::set_var("DBUS_SESSION_BUS_ADDRESS", &addr);
            log_info("[vietc] Set DBUS_SESSION_BUS_ADDRESS from XDG_RUNTIME_DIR/bus");
        }
    }
}
