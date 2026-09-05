use std::collections::HashSet;
use std::fs;
use std::os::unix::io::AsRawFd;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::daemon::Daemon;
use crate::display;
#[cfg(feature = "x11")]
use crate::display::DisplayServer;
use crate::event::{
    is_caps_lock_on, is_flush_char,
    is_modifier_pressed, is_modifier_held_shift, is_toggle_combination_state,
    is_toggle_key, is_vn_control_key, key_to_char,
};
use crate::inject::{create_injector, execute_commands};
use crate::log::log_info;
use crate::signal::SIGNAL_EXIT;
use crate::commands::OutputCommand;

pub fn run_with_evdev(
    devices: &mut Vec<(evdev::Device, String)>,
    daemon: &mut Daemon,
    shared_active_window: Arc<Mutex<String>>,
    shared_window_class: Arc<Mutex<String>>,
    config_changed: Arc<std::sync::atomic::AtomicBool>,
    status_changed: Arc<std::sync::atomic::AtomicBool>,
    _engine_enabled: Arc<std::sync::atomic::AtomicBool>,
    display: display::DisplayServer,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut injector = create_injector(display)?;

    let mut any_grabbed = false;
    if daemon.grab_enabled && !devices.is_empty() {
        for (idx, (dev, _name)) in devices.iter_mut().enumerate() {
            match dev.grab() {
                Ok(()) => {
                    any_grabbed = true;
                    log_info(&format!("[vietc] Grabbed keyboard device #{}", idx));
                }
                Err(e) => {
                    log_info(&format!(
                        "[vietc] Could not grab keyboard #{}: {} (run as root for grab)",
                        idx, e
                    ));
                }
            }
        }
        if any_grabbed {
            log_info("[vietc] All available keyboards grabbed — race condition eliminated");
        } else {
            log_info("[vietc] Falling back to non-grabbing mode (may have race)");
        }
    } else if !daemon.grab_enabled {
        log_info("[vietc] Keyboard grab disabled (config grab = false)");
        log_info("[vietc] Set grab = true in vietc.toml to enable (needs root)");
    }
    let grabbed = any_grabbed;

    // Track currently opened device paths and physical IDs so newly connected keyboards can be hotplugged without duplicates
    let mut known_paths: HashSet<String> = HashSet::new();
    let mut known_phys_ids: HashSet<String> = HashSet::new();
    for (_, n) in devices.iter() {
        let p = dev_path_of(n);
        known_paths.insert(p.clone());
        if let Some(phys) = crate::device::get_device_phys_id(std::path::Path::new(&p)) {
            known_phys_ids.insert(phys);
        }
    }

    if !grabbed {
        if unsafe { libc::geteuid() } != 0 {
            log_info("[vietc] WARNING: not running as root — keyboard grab unavailable");
            log_info("[vietc] WARNING: non-grabbed mode has race conditions with fast typing");
            log_info("[vietc] WARNING: run with sudo, or setcap cap_sys_admin+ep on the binary");
        }
        #[cfg(feature = "x11")]
        if display != DisplayServer::Wayland {
            if let Ok(x11_inj) = vietc_protocol::x11_inject::X11Injector::new() {
                injector = Box::new(x11_inj);
                log_info("[vietc] Non-grabbed: using X11 injection (faster than uinput)");
            }
        }
    }

    let mut device_states: Vec<(evdev::AttributeSet<evdev::Key>, bool)> = devices
        .iter()
        .map(|(d, _)| {
            let caps = is_caps_lock_on(d);
            (evdev::AttributeSet::new(), caps)
        })
        .collect();

    let mut consumed_keys: HashSet<u16> = HashSet::new();
    let mut dedup = crate::key_dedup::DedupState::new();
    let mut last_active_window = String::new();
    let mut last_window_class = String::new();
    let mut password_check_counter: u32 = 0;
    let mut last_key_time = std::time::Instant::now();

    log_info("[vietc] Event loop started");

    // Cache debug flag to avoid reloading config every keystroke
    let mut debug_logging = daemon.config.debug;

    loop {
        if SIGNAL_EXIT.load(Ordering::SeqCst) {
            if grabbed && !devices.is_empty() {
                for (dev, _name) in devices.iter_mut() {
                    let _ = dev.ungrab();
                }
                log_info("[vietc] Signal received — keyboard grab released");
            }
            log_info("[vietc] Exiting on signal");
            return Ok(());
        }

        // Poll ALL devices simultaneously
        let mut pfds: Vec<libc::pollfd> = devices
            .iter()
            .map(|(d, _)| libc::pollfd {
                fd: d.as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            })
            .collect();

        let poll_ret = unsafe { libc::poll(pfds.as_mut_ptr(), pfds.len() as libc::nfds_t, 100) };
        if poll_ret < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                continue;
            }
            log_info(&format!(
                "[vietc] poll error on evdev fd: {:?} — exiting",
                err
            ));
            return Err(err.into());
        }
        if poll_ret == 0 {
            if daemon.config.app_state.enabled {
                let class = shared_window_class.lock().unwrap().clone();
                if !class.is_empty() && class != last_window_class {
                    last_window_class = class.clone();
                    daemon.check_app_change_with(last_window_class.clone());
                }
            }
            // Hotplug: dynamically attach keyboards connected after startup
            let new_devs = discover_new_keyboards(&known_paths, &mut known_phys_ids);
            for (mut dev, name) in new_devs {
                known_paths.insert(dev_path_of(&name));
                if daemon.grab_enabled {
                    if dev.grab().is_ok() {
                        log_info(&format!("[vietc] Hotplug grabbed keyboard: {}", name));
                    }
                }
                let caps = is_caps_lock_on(&dev);
                log_info(&format!("[vietc] Hotplug connected keyboard: {}", name));
                devices.push((dev, name.clone()));
                device_states.push((evdev::AttributeSet::new(), caps));
            }
            continue;
        }

        // Check for status changes instantly
        if status_changed.load(Ordering::SeqCst) {
            daemon.sync_status_file();
            status_changed.store(false, Ordering::SeqCst);
        }

        // Check for config reload instantly
        if config_changed.load(Ordering::SeqCst) {
            daemon.reload_config();
            config_changed.store(false, Ordering::SeqCst);
            debug_logging = daemon.config.debug;
        }

        // Process events from whichever device(s) have data ready
        let mut dead: Vec<usize> = Vec::new();
        for (i, pfd) in pfds.iter().enumerate() {
            if (pfd.revents & libc::POLLIN) == 0 {
                continue;
            }

            let (ref mut device, ref _name) = devices[i];
            let caps = device_states[i].1;
            let mut key_state = std::mem::take(&mut device_states[i].0);

            let event_list = match device.fetch_events() {
                Ok(events) => events.collect::<Vec<_>>(),
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::Interrupted {
                        continue;
                    }
                    log_info(&format!(
                        "[vietc] fetch_events error on device {}: {:?} — removing device",
                        i, e
                    ));
                    dead.push(i);
                    continue;
                }
            };

            for event in event_list {
                if event.event_type() != evdev::EventType::KEY {
                    continue;
                }
                let keycode = event.code();
                let value = event.value();
                let key = evdev::Key(keycode);

                // Update key state dynamically
                if value == 1 {
                    key_state.insert(key);
                } else if value == 0 {
                    key_state.remove(key);
                }

                // Completely bypass all IME processing for terminal emulators, IDE terminals, and games
                if daemon.is_current_app_bypassed() {
                    if grabbed {
                        injector.send_key_event(keycode, value);
                    }
                    continue;
                }

                if value == 1
                    && is_toggle_key(key, &daemon.config.toggle_key)
                    && is_toggle_combination_state(&key_state, &daemon.config.toggle_key)
                {
                    consumed_keys.insert(keycode);
                    daemon.toggle_method();
                    continue;
                }

                if value == 1 && daemon.config.deduplicate_keys {
                    let dedupable = keycode < 256 && !is_modifier_pressed(&key_state);
                    if dedup.observe(
                        keycode as u32,
                        dedupable,
                        keycode == 57,
                        daemon.config.deduplicate_two_back,
                        daemon.config.deduplicate_window_ms,
                        std::time::Instant::now(),
                    ) {
                        consumed_keys.insert(keycode);
                        continue;
                    }
                }

                // In non-grabbed mode only the primary device is processed
                // through the engine, because non-primary devices aren't grabbed
                // and their events reach the app directly — processing them would
                // inject a second copy of every keystroke. In grabbed mode every
                // keyboard is grabbed (originals suppressed), so all devices may
                // be processed safely. This also lets a uinput virtual keyboard
                // (e.g. the vietc-vk test tool) drive the engine.
                if !grabbed && i != 0 {
                    continue;
                }

                if !grabbed {
                    if value != 1 {
                        continue;
                    }
                    if is_modifier_pressed(&key_state) {
                        continue;
                    }
                    if !daemon.engine.is_enabled() {
                        continue;
                    }
                    if let Some(ch) = key_to_char(key) {
                        let buf_before = daemon.engine.buffer();
                        let mut commands = daemon.process_key(ch);
                        if commands.is_empty()
                            && daemon.engine.is_enabled()
                            && is_vn_control_key(daemon.app_state.effective_method(), ch)
                        {
                            let buf_after = daemon.engine.buffer();
                            if buf_after != buf_before && !buf_before.is_empty() {
                                let len = buf_before.chars().count();
                                commands.push(OutputCommand::Backspace(len + 1));
                                commands.push(OutputCommand::Type(buf_after));
                            }
                        }
                        // Non-grabbed fix: the VNI/Telex control key character reached
                        // the app directly. Add 1 extra backspace to remove it.
                        if !commands.is_empty()
                            && is_vn_control_key(daemon.app_state.effective_method(), ch)
                        {
                            for cmd in &mut commands {
                                if let OutputCommand::Backspace(ref mut n) = cmd {
                                    *n += 1;
                                }
                            }
                            log_info(&format!(
                                "[vietc] non-grabbed: ch='{}' adjusted backspace+1",
                                ch.escape_default()
                            ));
                        }
                        if !commands.is_empty() {
                            log_info(&format!(
                                "[vietc] non-grabbed inject: ch='{}' cmds={:?}",
                                ch.escape_default(),
                                commands
                            ));
                        }
                        execute_commands(&*injector, &commands);
                    }
                } else {
                    if is_modifier_pressed(&key_state) {
                        injector.send_key_event(keycode, value);
                        continue;
                    }

                    // Engine disabled in grabbed mode: forward keys directly
                    if !daemon.engine.is_enabled() {
                        injector.send_key_event(keycode, value);
                        continue;
                    }

                    if key == evdev::Key::KEY_BACKSPACE {
                        if value == 1 || value == 2 {
                            daemon.engine.process_key('\x08');
                            injector.send_key_event(14, 1);
                            injector.send_key_event(14, 0);
                        }
                        consumed_keys.insert(keycode);
                        continue;
                    }

                    if value == 1 {
                        if debug_logging {
                            log_info(&format!(
                                "[vietc] grabbed key: code={} ch='{}' buf='{}' enabled={}",
                                keycode,
                                key_to_char(key).map(|c| c.escape_default().to_string()).unwrap_or_default(),
                                daemon.engine.buffer().escape_default(),
                                daemon.engine.is_enabled(),
                            ));
                        }
                        if consumed_keys.contains(&keycode) {
                            consumed_keys.remove(&keycode);
                        }
                        if let Some(mut ch) = key_to_char(key) {
                            let gap = last_key_time.elapsed();
                            last_key_time = std::time::Instant::now();

                            let active_window_id = shared_active_window.lock().unwrap().clone();
                            let active_window_class = shared_window_class.lock().unwrap().clone();
                            let mut new_window = None;

                            if !active_window_id.is_empty() && active_window_id != last_active_window {
                                new_window = Some(active_window_id.clone());
                            } else if !active_window_class.is_empty()
                                && active_window_class != last_window_class
                            {
                                new_window = Some(active_window_class.clone());
                            }

                            if let Some(id) = new_window {
                                log_info(&format!(
                                    "[vietc] Window changed: '{}' -> '{}' (gap={:?})",
                                    last_active_window, id, gap
                                ));
                                last_active_window = id.clone();
                                if !active_window_class.is_empty() {
                                    last_window_class = active_window_class.clone();
                                }
                                daemon.engine.reset();
                                daemon.replay_reset();

                                if daemon.config.app_state.enabled {
                                    let class = shared_window_class.lock().unwrap().clone();
                                    let class = if class.is_empty() {
                                        crate::app_state::get_focused_window_class().unwrap_or_default()
                                    } else {
                                        class
                                    };
                                    injector.set_active_window(&class);
                                    daemon.check_app_change_with(class);
                                }

                                if daemon.config.password_detection.enabled {
                                    let is_pw = daemon.app_state.check_password_field();
                                    if is_pw && daemon.engine.is_enabled() {
                                        daemon.engine.set_enabled(false);
                                        daemon.engine.reset();
                                        daemon.replay_reset();
                                        daemon.write_status();
                                    }
                                }
                            } else if daemon.config.app_state.enabled {
                                let class = shared_window_class.lock().unwrap().clone();
                                if !class.is_empty() {
                                    injector.set_active_window(&class);
                                }
                            }

                            if daemon.config.password_detection.enabled {
                                password_check_counter += 1;
                                if password_check_counter >= 30 {
                                    password_check_counter = 0;
                                    let is_pw = daemon.app_state.check_password_field();
                                    let currently_enabled = daemon.engine.is_enabled();
                                    if is_pw && currently_enabled {
                                        daemon.engine.set_enabled(false);
                                        daemon.engine.reset();
                                        daemon.replay_reset();
                                        daemon.write_status();
                                        log_info("[vietc] Password field detected (periodic) — engine disabled");
                                    } else if !is_pw && !currently_enabled {
                                        if daemon.app_state.get_default_state() {
                                            daemon.engine.set_enabled(true);
                                            daemon.engine.reset();
                                            daemon.replay_reset();
                                            daemon.write_status();
                                        }
                                    }
                                }
                            }

                            let shift = is_modifier_held_shift(&key_state);
                            if ch.is_ascii_alphabetic() && (shift ^ caps) {
                                ch = ch.to_ascii_uppercase();
                            }
                            let buf_before = daemon.engine.buffer().chars().count();
                            let commands = daemon.process_key(ch);
                            if !commands.is_empty() {
                                if debug_logging {
                                    log_info(&format!(
                                        "[vietc] grabbed inject: ch='{}' cmds={:?}",
                                        ch.escape_default(),
                                        commands,
                                    ));
                                }
                                consumed_keys.insert(keycode);
                                execute_commands(&*injector, &commands);
                                if is_flush_char(ch) && daemon.engine.is_enabled() {
                                    injector.send_key_event(keycode, 1);
                                    injector.send_key_event(keycode, 0);
                                }
                            } else if daemon.engine.is_enabled()
                                && is_vn_control_key(daemon.app_state.effective_method(), ch)
                                && daemon.engine.buffer().chars().count() <= buf_before
                            {
                                if debug_logging {
                                    log_info(&format!(
                                        "[vietc] grabbed consumed: ch='{}' (control key, no screen change)",
                                        ch.escape_default(),
                                    ));
                                }
                                consumed_keys.insert(keycode);
                            } else {
                                if debug_logging {
                                    log_info(&format!(
                                        "[vietc] grabbed forward: ch='{}' (no commands)",
                                        ch.escape_default(),
                                    ));
                                }
                                injector.send_key_event(keycode, 1);
                            }
                        } else {
                            injector.send_key_event(keycode, 1);
                        }
                    } else if value == 2 {
                        if consumed_keys.contains(&keycode) {
                            continue;
                        }
                        injector.send_key_event(keycode, 2);
                    } else if value == 0 {
                        if consumed_keys.remove(&keycode) {
                            continue;
                        }
                        injector.send_key_event(keycode, 0);
                    }
                }
            }

            // Save updated key state back
            device_states[i].0 = key_state;
        }

        // Remove any devices that errored out (e.g. a uinput virtual keyboard
        // like vietc-vk was closed) so we don't crash the whole daemon.
        if !dead.is_empty() {
            dead.sort_unstable();
            dead.dedup();
            for &d in dead.iter().rev() {
                let name = devices[d].1.clone();
                devices.remove(d);
                device_states.remove(d);
                let path = dev_path_of(&name);
                known_paths.remove(&path);
                // Rebuild phys set from remaining paths — virtual uinput
                // devices disappear from sysfs on close, so recomputing phys_id
                // for the dead path would return None and leave a stale entry
                // that blocks the next virtual keyboard with the same input
                // number (e.g. re-used /dev/input/event31 → same
                // sysfs:/sys/devices/virtual/input/input31).
                known_phys_ids.clear();
                for p in known_paths.iter() {
                    if let Some(pid) = crate::device::get_device_phys_id(std::path::Path::new(p)) {
                        known_phys_ids.insert(pid);
                    }
                }
                log_info(&format!("[vietc] Device disconnected/removed: {}", name));
            }
        }
    }
}

/// Extract the device path (the part before " (") from a device label of the
/// form `"<path> (<name>)"` produced by `open_keyboard_devices`.
fn dev_path_of(label: &str) -> String {
    label.split(" (").next().unwrap_or(label).to_string()
}

/// Look up the label for a device index that still exists (used only when
/// removing devices, where `d` is always < devices.len()).
fn devices_name_at(devices: &[(evdev::Device, String)], d: usize) -> &str {
    &devices[d].1
}

/// Discover keyboard devices present in /dev/input that are not already known.
/// Used for hotplug so newly connected keyboards (USB/wireless/virtual) are attached dynamically.
fn discover_new_keyboards(
    existing_paths: &HashSet<String>,
    existing_phys: &mut HashSet<String>,
) -> Vec<(evdev::Device, String)> {
    let mut out = Vec::new();
    let mut added_paths = HashSet::new();

    // 1. Scan /dev/input/by-path/*-event-kbd
    let by_path = std::path::Path::new("/dev/input/by-path");
    if by_path.exists() {
        if let Ok(rd) = fs::read_dir(by_path) {
            for entry in rd.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with("-event-kbd") {
                    if let Ok(real_path) = fs::canonicalize(entry.path()) {
                        let p = real_path.to_string_lossy().to_string();
                        let phys_opt = crate::device::get_device_phys_id(&real_path);
                        if let Some(phys_id) = &phys_opt {
                            if existing_phys.contains(phys_id) {
                                continue;
                            }
                        }
                        if existing_paths.contains(&p) {
                            continue;
                        }
                        if !added_paths.insert(p.clone()) {
                            continue;
                        }
                        if let Ok(device) = evdev::Device::open(&real_path) {
                            let dev_name = device.name().unwrap_or("unknown").to_string();
                            if crate::device::is_valid_keyboard(&device) {
                                if let Some(pid) = phys_opt {
                                    existing_phys.insert(pid);
                                }
                                out.push((device, format!("{} ({})", real_path.display(), dev_name)));
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Scan /dev/input/by-id/*-event-kbd
    let by_id = std::path::Path::new("/dev/input/by-id");
    if by_id.exists() {
        if let Ok(rd) = fs::read_dir(by_id) {
            for entry in rd.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.ends_with("-event-kbd") {
                    if let Ok(real_path) = fs::canonicalize(entry.path()) {
                        let p = real_path.to_string_lossy().to_string();
                        let phys_opt = crate::device::get_device_phys_id(&real_path);
                        if let Some(phys_id) = &phys_opt {
                            if existing_phys.contains(phys_id) {
                                continue;
                            }
                        }
                        if existing_paths.contains(&p) {
                            continue;
                        }
                        if !added_paths.insert(p.clone()) {
                            continue;
                        }
                        if let Ok(device) = evdev::Device::open(&real_path) {
                            let dev_name = device.name().unwrap_or("unknown").to_string();
                            if crate::device::is_valid_keyboard(&device) {
                                if let Some(pid) = phys_opt {
                                    existing_phys.insert(pid);
                                }
                                out.push((device, format!("{} ({})", real_path.display(), dev_name)));
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Scan /dev/input/event* for Bluetooth & direct devices without by-path/by-id symlinks
    let dir = std::path::Path::new("/dev/input");
    if dir.exists() {
        if let Ok(rd) = fs::read_dir(dir) {
            for entry in rd.flatten() {
                let name_str = entry.file_name().to_string_lossy().to_string();
                if name_str.starts_with("event") {
                    if let Ok(real_path) = fs::canonicalize(entry.path()) {
                        let p = real_path.to_string_lossy().to_string();
                        let phys_opt = crate::device::get_device_phys_id(&real_path);
                        if let Some(phys_id) = &phys_opt {
                            if existing_phys.contains(phys_id) {
                                continue;
                            }
                        }
                        if existing_paths.contains(&p) {
                            continue;
                        }
                        if !added_paths.insert(p.clone()) {
                            continue;
                        }
                        if let Ok(device) = evdev::Device::open(&real_path) {
                            let dev_name = device.name().unwrap_or("unknown").to_string();
                            if crate::device::is_valid_keyboard(&device) {
                                if let Some(pid) = phys_opt {
                                    existing_phys.insert(pid);
                                }
                                out.push((device, format!("{} ({})", real_path.display(), dev_name)));
                            }
                        }
                    }
                }
            }
        }
    }

    out
}
