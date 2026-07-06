use std::fs;

use crate::log::log_info;

pub fn open_keyboard_devices() -> Result<Vec<(evdev::Device, String)>, Box<dyn std::error::Error>> {
    let dir = std::path::Path::new("/dev/input");
    if !dir.exists() {
        return Err("No /dev/input directory".into());
    }

    let mut devices: Vec<(evdev::Device, String)> = Vec::new();
    let mut permission_denied_count = 0u32;
    let mut total_event_count = 0u32;

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if name_str.starts_with("event") {
            total_event_count += 1;
            match evdev::Device::open(entry.path()) {
                Ok(device) => {
                    let dev_name = device.name().unwrap_or("unknown").to_string();
                    if dev_name.eq_ignore_ascii_case("vietc") {
                        continue;
                    }
                    if device
                        .supported_keys()
                        .is_some_and(|k| k.contains(evdev::Key::KEY_A))
                    {
                        log_info(&format!(
                            "[vietc] Found keyboard device: {} ({})",
                            entry.path().display(),
                            dev_name
                        ));
                        devices.push((device, format!("{} ({})", entry.path().display(), dev_name)));
                    }
                }
                Err(e) => {
                    if e.raw_os_error() == Some(libc::EACCES) {
                        permission_denied_count += 1;
                    }
                    continue;
                }
            }
        }
    }

    if !devices.is_empty() {
        log_info(&format!("[vietc] Opened {} keyboard device(s)", devices.len()));
        return Ok(devices);
    }

    if permission_denied_count > 0 {
        let username = std::env::var("USER").unwrap_or_else(|_| {
            std::process::Command::new("id")
                .arg("-un")
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
                .unwrap_or_default()
        });
        let in_group_db = if !username.is_empty() {
            std::process::Command::new("id")
                .args(["-nG", &username])
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).contains("input"))
                .unwrap_or(false)
        } else {
            false
        };

        if in_group_db {
            Err(format!(
                "Permission denied on {}/{} devices. Your user IS in the 'input' group, \
                 but your current session hasn't picked it up yet. \
                 Please LOG OUT and LOG BACK IN to activate group permissions.",
                permission_denied_count, total_event_count
            )
            .into())
        } else {
            Err(format!(
                "Permission denied on {}/{} devices. Add your user to the 'input' group: \
                 sudo usermod -aG input $USER, \
                 then log out and log back in.",
                permission_denied_count, total_event_count
            )
            .into())
        }
    } else {
        Err("No keyboard device found".into())
    }
}
