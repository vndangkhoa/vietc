// SPDX-License-Identifier: MIT
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

mod app_state;
mod commands;
mod config;
mod daemon;
mod device;
mod display;
mod env;
mod event;
mod evdev_loop;
mod inject;
mod log;
mod password_detector;
mod signal;
mod stdin;

#[cfg(feature = "x11")]
mod x11_capture;

use daemon::Daemon;
use device::open_keyboard_devices;
use env::recover_display_env;
use evdev_loop::run_with_evdev;
use log::log_info;
use signal::{ensure_single_instance, install_signal_handlers};
use stdin::run_stdin_mode;

/// Pin current thread to performance cores (0-3) and boost priority.
fn boost_thread_priority() {
    unsafe {
        libc::setpriority(libc::PRIO_PROCESS, 0, -10);

        #[cfg(target_os = "linux")]
        {
            let mut cpuset: libc::cpu_set_t = std::mem::zeroed();
            for i in 0..4 {
                libc::CPU_SET(i, &mut cpuset);
            }
            let ret = libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &cpuset);
            if ret == 0 {
                eprintln!("[vietc] Pinned to P-cores 0-3, nice=-10");
            } else {
                eprintln!("[vietc] CPU pinning failed ({}), nice=-10 still set", ret);
            }
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    install_signal_handlers();
    ensure_single_instance("vietc-daemon");

    recover_display_env();
    let config_path = config::find_config_path();
    let config = config::Config::load()?;
    let engine_enabled = Arc::new(AtomicBool::new(config.start_enabled));
    let mut daemon = Daemon::new(config, config_path.clone(), engine_enabled.clone());

    daemon.write_status();

    let display = display::detect_display_server();
    let compositor = display::detect_compositor();

    log_info(&format!("Viet+ Daemon v{}", env!("CARGO_PKG_VERSION")));
    log_info(&format!(
        "Display: {:?} ({})",
        display,
        compositor.unwrap_or_else(|| "unknown".into())
    ));
    log_info(&format!("Input method: {:?}", daemon.config.input_method));
    log_info(&format!(
        "Toggle key: Ctrl+{}",
        daemon.config.toggle_key.to_uppercase()
    ));
    log_info(&format!(
        "App memory: {}",
        if daemon.config.app_state.enabled { "ON" } else { "OFF" }
    ));

    let display_var = std::env::var("DISPLAY").unwrap_or_default();
    let xauth_var = std::env::var("XAUTHORITY").unwrap_or_default();
    log_info(&format!("[vietc] DISPLAY='{}'  XAUTHORITY='{}'", display_var, xauth_var));
    if display_var.is_empty() && unsafe { libc::getuid() } == 0 {
        log_info("[vietc] WARNING: DISPLAY not set — clipboard paste won't work");
        log_info("[vietc] WARNING: start via vietc-tray (passes DISPLAY) or use sudo -E");
    }
    match std::process::Command::new("xdotool")
        .args(["getactivewindow"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let id = String::from_utf8_lossy(&output.stdout).trim().to_string();
                log_info(&format!("[vietc] xdotool OK: active window = {}", id));
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                log_info(&format!("[vietc] xdotool FAILED: {}", stderr.trim()));
            }
        }
        Err(e) => {
            log_info(&format!("[vietc] xdotool NOT AVAILABLE: {}", e));
        }
    }

    boost_thread_priority();

    let shared_active_window = Arc::new(Mutex::new(String::new()));
    let shared_window_class = Arc::new(Mutex::new(String::new()));
    let config_changed = Arc::new(AtomicBool::new(false));
    let status_changed = Arc::new(AtomicBool::new(false));

    {
        let shared_active_window = shared_active_window.clone();
        let shared_window_class = shared_window_class.clone();
        let config_changed = config_changed.clone();
        let config_path = config_path.clone();
        let status_changed = status_changed.clone();
        let engine_enabled = engine_enabled.clone();
        let mut last_modified = std::fs::metadata(&config_path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::now());

        thread::spawn(move || {
            let mut window_check_counter = 0;
            let status_path = config_path.parent().unwrap().join("status");
            loop {
                if let Some(id) = app_state::get_active_window_id() {
                    let mut lock = shared_active_window.lock().unwrap();
                    if *lock != id {
                        log_info(&format!("[vietc] bg: window ID '{}' -> '{}'", *lock, id));
                        *lock = id;
                    }
                } else {
                    log_info("[vietc] bg: window ID poll failed");
                }
                if let Some(class) = app_state::get_focused_window_class() {
                    let mut lock = shared_window_class.lock().unwrap();
                    if *lock != class {
                        *lock = class;
                    }
                }

                if let Ok(content) = std::fs::read_to_string(&status_path) {
                    let is_vn = content.trim() == "vn";
                    let current_enabled = engine_enabled.load(Ordering::SeqCst);
                    if is_vn != current_enabled {
                        status_changed.store(true, Ordering::SeqCst);
                    }
                }

                window_check_counter += 1;
                if window_check_counter >= 6 {
                    window_check_counter = 0;
                    if let Ok(metadata) = std::fs::metadata(&config_path) {
                        if let Ok(modified) = metadata.modified() {
                            if modified > last_modified {
                                last_modified = modified;
                                config_changed.store(true, Ordering::SeqCst);
                            }
                        }
                    }
                }

                thread::sleep(std::time::Duration::from_millis(250));
            }
        });
    }

    match open_keyboard_devices() {
        Ok(mut devices) => {
            match run_with_evdev(
                &mut devices,
                &mut daemon,
                shared_active_window.clone(),
                shared_window_class.clone(),
                config_changed.clone(),
                status_changed.clone(),
                engine_enabled.clone(),
                display,
            ) {
                Ok(()) => {
                    log_info("[vietc] evdev returned, trying X11 capture as fallback");
                }
                Err(e) => {
                    log_info(&format!(
                        "[vietc] evdev exited with error: {} — trying X11 capture",
                        e
                    ));
                }
            }
        }
        Err(e) => {
            log_info(&format!("[vietc] evdev not available: {}", e));
        }
    }

    #[cfg(feature = "x11")]
    if display != display::DisplayServer::Wayland {
        log_info("[vietc] Trying X11 keymap-based capture");
        match x11_capture::run_with_x11_keymap(
            &mut daemon,
            shared_active_window.clone(),
            shared_window_class.clone(),
            config_changed.clone(),
            status_changed.clone(),
            engine_enabled.clone(),
            display,
        ) {
            Ok(()) => {
                log_info("[vietc] X11 keymap returned, falling through to stdin mode");
            }
            Err(e) => {
                log_info(&format!(
                    "[vietc] X11 keymap exited with error: {} — falling back",
                    e
                ));
            }
        }
    }

    log_info("[vietc] Running in stdin test mode");
    run_stdin_mode(
        &mut daemon,
        shared_active_window,
        shared_window_class,
        config_changed,
        status_changed,
        engine_enabled,
        display,
    )?;

    Ok(())
}
