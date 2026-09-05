use std::fs;

use crate::log::log_info;

pub fn is_valid_keyboard(device: &evdev::Device) -> bool {
    let dev_name = device.name().unwrap_or("unknown").to_string();
    let dev_name_lower = dev_name.to_lowercase();
    if dev_name.eq_ignore_ascii_case("vietc") {
        return false;
    }
    if dev_name_lower.contains("touchpad") || dev_name_lower.contains("trackpoint") {
        return false;
    }
    // Reject pointers/mice based on relative axes (REL_X / REL_Y)
    if let Some(rel) = device.supported_relative_axes() {
        if rel.contains(evdev::RelativeAxisType::REL_X)
            || rel.contains(evdev::RelativeAxisType::REL_Y)
        {
            return false;
        }
    }
    if let Some(keys) = device.supported_keys() {
        keys.contains(evdev::Key::KEY_A)
            && keys.contains(evdev::Key::KEY_Z)
            && keys.contains(evdev::Key::KEY_SPACE)
            && keys.contains(evdev::Key::KEY_ENTER)
    } else {
        false
    }
}

pub fn get_device_phys_id(path: &std::path::Path) -> Option<String> {
    let file_name = path.file_name()?.to_string_lossy();
    if !file_name.starts_with("event") {
        return None;
    }
    let ev_num = file_name.strip_prefix("event")?;
    let dev_dir = format!("/sys/class/input/event{}/device", ev_num);
    let dev_path = std::path::Path::new(&dev_dir);
    if !dev_path.exists() {
        return None;
    }

    // 1. Check uniq (Bluetooth MAC address or device serial)
    if let Ok(uniq) = fs::read_to_string(dev_path.join("uniq")) {
        let u = uniq.trim();
        if !u.is_empty() {
            return Some(format!("uniq:{}", u));
        }
    }

    // 2. Check phys (USB hardware endpoint prefix)
    if let Ok(phys) = fs::read_to_string(dev_path.join("phys")) {
        let ph = phys.trim();
        if !ph.is_empty() {
            // Strip "/input0", "/input1" suffix so multiple interfaces of the same physical USB dongle share one ID
            let base = ph.split('/').next().unwrap_or(ph);
            return Some(format!("phys:{}", base));
        }
    }

    // 3. Fallback to parent sysfs path — use full canonical path (unique per event)
    // Previous code used parent.parent() which collapsed all virtual uinput
    // devices (e.g. /sys/devices/virtual/input/inputXX) to the same
    // "sysfs:/sys/devices/virtual/input", causing only the first virtual
    // keyboard to be grabbed and subsequent ones (e.g. vietc-vk, tests) to be
    // ignored → raw input, no conversion, and apparent double/loss.
    if let Ok(canonical) = fs::canonicalize(dev_path) {
        let canon_str = canonical.display().to_string();
        // Virtual uinput devices all live under /sys/devices/virtual/input/
        // and reuse input numbers (input31, input32, ...) quickly. Using the
        // canonical path as phys_id would still collide when the kernel
        // reuses the same inputXX for a new device before the old stale entry
        // is cleared. For virtual devices, return None so they are deduped
        // only by their /dev/input/eventXX path, not by phys.
        if canon_str.contains("/virtual/") {
            return None;
        }
        return Some(format!("sysfs:{}", canon_str));
    }

    Some(format!("path:{}", path.display()))
}

pub fn open_keyboard_devices() -> Result<Vec<(evdev::Device, String)>, Box<dyn std::error::Error>> {
    let mut devices: Vec<(evdev::Device, String)> = Vec::new();
    let mut permission_denied_count = 0u32;
    let mut total_event_count = 0u32;
    let mut seen_paths = std::collections::HashSet::new();
    let mut seen_phys_ids = std::collections::HashSet::new();

    // Strategy 1: Open primary keyboard devices via /dev/input/by-path/*-event-kbd
    let by_path = std::path::Path::new("/dev/input/by-path");
    if by_path.exists() {
        if let Ok(rd) = fs::read_dir(by_path) {
            for entry in rd.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with("-event-kbd") {
                    if let Ok(real_path) = fs::canonicalize(entry.path()) {
                        let p = real_path.to_string_lossy().to_string();
                        let phys_opt = get_device_phys_id(&real_path);
                        if let Some(phys_id) = &phys_opt {
                            if !seen_phys_ids.insert(phys_id.clone()) {
                                continue;
                            }
                        }
                        if !seen_paths.insert(p.clone()) {
                            // Roll back phys insert if path was duplicate
                            if let Some(pid) = phys_opt {
                                seen_phys_ids.remove(&pid);
                            }
                            continue;
                        }
                        match evdev::Device::open(&real_path) {
                            Ok(device) => {
                                let dev_name = device.name().unwrap_or("unknown").to_string();
                                if is_valid_keyboard(&device) {
                                    log_info(&format!(
                                        "[vietc] Found keyboard (by-path): {} ({})",
                                        real_path.display(),
                                        dev_name
                                    ));
                                    devices.push((device, format!("{} ({})", real_path.display(), dev_name)));
                                }
                            }
                            Err(e) => {
                                if e.raw_os_error() == Some(libc::EACCES) {
                                    permission_denied_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Strategy 2: Open keyboard devices via /dev/input/by-id/*-event-kbd
    let by_id = std::path::Path::new("/dev/input/by-id");
    if by_id.exists() {
        if let Ok(rd) = fs::read_dir(by_id) {
            for entry in rd.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with("-event-kbd") {
                    if let Ok(real_path) = fs::canonicalize(entry.path()) {
                        let p = real_path.to_string_lossy().to_string();
                        let phys_opt = get_device_phys_id(&real_path);
                        if let Some(phys_id) = &phys_opt {
                            if !seen_phys_ids.insert(phys_id.clone()) {
                                continue;
                            }
                        }
                        if !seen_paths.insert(p.clone()) {
                            if let Some(pid) = phys_opt {
                                seen_phys_ids.remove(&pid);
                            }
                            continue;
                        }
                        match evdev::Device::open(&real_path) {
                            Ok(device) => {
                                let dev_name = device.name().unwrap_or("unknown").to_string();
                                if is_valid_keyboard(&device) {
                                    log_info(&format!(
                                        "[vietc] Found keyboard (by-id): {} ({})",
                                        real_path.display(),
                                        dev_name
                                    ));
                                    devices.push((device, format!("{} ({})", real_path.display(), dev_name)));
                                }
                            }
                            Err(e) => {
                                if e.raw_os_error() == Some(libc::EACCES) {
                                    permission_denied_count += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Strategy 3: Scan /dev/input/event* for Bluetooth & direct devices without by-path/by-id symlinks
    let dir = std::path::Path::new("/dev/input");
    if dir.exists() {
        if let Ok(rd) = fs::read_dir(dir) {
            for entry in rd.flatten() {
                let name_str = entry.file_name().to_string_lossy().to_string();
                if name_str.starts_with("event") {
                    total_event_count += 1;
                    if let Ok(real_path) = fs::canonicalize(entry.path()) {
                        let p = real_path.to_string_lossy().to_string();
                        let phys_opt = get_device_phys_id(&real_path);
                        if let Some(phys_id) = &phys_opt {
                            if !seen_phys_ids.insert(phys_id.clone()) {
                                continue;
                            }
                        }
                        if !seen_paths.insert(p.clone()) {
                            if let Some(pid) = phys_opt {
                                seen_phys_ids.remove(&pid);
                            }
                            continue;
                        }
                        match evdev::Device::open(&real_path) {
                            Ok(device) => {
                                let dev_name = device.name().unwrap_or("unknown").to_string();
                                if is_valid_keyboard(&device) {
                                    log_info(&format!(
                                        "[vietc] Found keyboard (bluetooth/direct): {} ({})",
                                        real_path.display(),
                                        dev_name
                                    ));
                                    devices.push((device, format!("{} ({})", real_path.display(), dev_name)));
                                }
                            }
                            Err(e) => {
                                if e.raw_os_error() == Some(libc::EACCES) {
                                    permission_denied_count += 1;
                                }
                            }
                        }
                    }
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
