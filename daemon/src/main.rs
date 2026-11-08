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
mod im_plan;
mod inject;
mod log;
mod password_detector;
mod signal;
mod stdin;
mod wayland_im;

#[cfg(feature = "x11")]
mod x11_capture;

mod ibus_engine;
mod key_dedup;
mod controller;
mod ibus_control;

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

/// Set when vietc stops a running IBus or Fcitx5 daemon/unit, so it can be restarted on exit.
static IBUS_STOPPED: AtomicBool = AtomicBool::new(false);
static FCITX_OMARCHY_STOPPED: AtomicBool = AtomicBool::new(false);
static FCITX_UNIT_STOPPED: AtomicBool = AtomicBool::new(false);
static FCITX_PROC_STOPPED: AtomicBool = AtomicBool::new(false);

fn is_systemd_unit_active_or_starting(unit: &str) -> bool {
    std::process::Command::new("systemctl")
        .args(["--user", "show", "-P", "ActiveState", unit])
        .output()
        .ok()
        .map(|o| {
            let state = String::from_utf8_lossy(&o.stdout);
            matches!(state.trim(), "active" | "activating" | "reloading")
        })
        .unwrap_or(false)
}

/// Stop any running IBus / Fcitx5 daemon so it does not compete with vietc as a
/// second X11/Wayland input method. On Hyprland/Wayland (including Omarchy's
/// `omarchy-fcitx5.service`), an active `fcitx5` holds `zwp_input_method_v2`
/// (`hl-virtual-keyboard-fcitx5`), which intercepts `uinput` BackSpaces and races
/// with `wtype` (`zwp_virtual_keyboard_v1`), causing glued syllables and orphaned
/// key-release runaway character repeats.
fn stop_competing_imes() {
    if std::process::Command::new("pgrep")
        .args(["-x", "ibus-daemon"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        if let Ok(status) = std::process::Command::new("ibus").arg("exit").status() {
            log_info(&format!("[vietc] stopped IBus (status {})", status));
            IBUS_STOPPED.store(true, Ordering::SeqCst);
        }
    }

    // Check systemd user services for fcitx5 first (including `activating` state at
    // login, since `omarchy-fcitx5.service` is `Type=dbus` with `Restart=always`).
    if is_systemd_unit_active_or_starting("omarchy-fcitx5.service") {
        if std::process::Command::new("systemctl")
            .args(["--user", "stop", "omarchy-fcitx5.service"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            log_info("[vietc] paused omarchy-fcitx5.service to prevent Wayland virtual-keyboard race");
            FCITX_OMARCHY_STOPPED.store(true, Ordering::SeqCst);
            return;
        }
    }

    if is_systemd_unit_active_or_starting("fcitx5.service") {
        if std::process::Command::new("systemctl")
            .args(["--user", "stop", "fcitx5.service"])
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            log_info("[vietc] paused fcitx5.service to prevent Wayland virtual-keyboard race");
            FCITX_UNIT_STOPPED.store(true, Ordering::SeqCst);
            return;
        }
    }

    if std::process::Command::new("pgrep")
        .args(["-x", "fcitx5"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
    {
        let exited = std::process::Command::new("fcitx5-remote")
            .arg("-e")
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
            || std::process::Command::new("pkill")
                .args(["-x", "fcitx5"])
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
        if exited {
            log_info("[vietc] paused standalone fcitx5 to prevent Wayland virtual-keyboard race");
            FCITX_PROC_STOPPED.store(true, Ordering::SeqCst);
        }
    }
}

/// Restart IBus / Fcitx5 if vietc stopped them, restoring the user's previous
/// input method when vietc quits. Best-effort; failures are ignored.
fn restart_competing_imes() {
    if IBUS_STOPPED.load(Ordering::SeqCst) {
        match std::process::Command::new("ibus-daemon").arg("-d").status() {
            Ok(_) => log_info("[vietc] restarted IBus"),
            Err(e) => log_info(&format!("[vietc] could not restart IBus: {}", e)),
        }
    }
    if FCITX_OMARCHY_STOPPED.load(Ordering::SeqCst) {
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "start", "omarchy-fcitx5.service"])
            .status();
        log_info("[vietc] restarted omarchy-fcitx5.service");
    } else if FCITX_UNIT_STOPPED.load(Ordering::SeqCst) {
        let _ = std::process::Command::new("systemctl")
            .args(["--user", "start", "fcitx5.service"])
            .status();
        log_info("[vietc] restarted fcitx5.service");
    } else if FCITX_PROC_STOPPED.load(Ordering::SeqCst) {
        let _ = std::process::Command::new("fcitx5")
            .args(["-d", "--disable", "notificationitem"])
            .status();
        log_info("[vietc] restarted fcitx5");
    }
}

/// Restarts competing IMEs on drop (i.e. when vietc exits cleanly via signal/return).
struct ImeRestartGuard;
impl Drop for ImeRestartGuard {
    fn drop(&mut self) {
        restart_competing_imes();
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    install_signal_handlers();
    ensure_single_instance("vietc-daemon");
    // Restart IBus/Fcitx5 on clean exit if we paused them.
    let _ime_guard = ImeRestartGuard;

    recover_display_env();
    let config_path = config::find_config_path();
    let config = config::Config::load()?;
    let engine_enabled = Arc::new(AtomicBool::new(config.start_enabled));
    let mut daemon = Daemon::new(config, config_path.clone(), engine_enabled.clone());

    daemon.write_status();

    // Aux controller mode: Bamboo (or another external IBus engine) does the
    // Vietnamese composition; vietc only switches the active IBus engine by
    // focused app / password field. vietc registers no IBus component.
    if daemon.config.controller_mode {
        log_info("[vietc] Aux controller mode enabled");
        return controller::run_controller(daemon);
    }

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

    // Native IBus engine mode: covers every app (incl. native-Wayland GNOME
    // clients that vietc's capture paths cannot reach). Takes precedence over
    // the capture paths and must NOT stop the running ibus-daemon.
    //
    // On Ubuntu GNOME Wayland (Mutter), zwp_input_method_v2 is unavailable
    // (docs/wayland-rootless.md:55), so the compositor-approved path is IBus.
    // We auto-enable it when `auto_ibus` is true — mirroring Funput's Linux
    // strategy: be an IBus engine instead of fighting it with evdev/X11.
    let should_use_ibus = crate::config::should_use_ibus_engine(&daemon.config);
    if should_use_ibus {
        if daemon.config.ibus_engine {
            log_info("[vietc] IBus engine mode enabled (explicit)");
        } else {
            log_info("[vietc] IBus engine mode auto-enabled for GNOME Wayland (Ubuntu fix)");
            log_info("[vietc] Set auto_ibus=false in config.toml to force legacy path");
        }
        let method = match daemon.config.input_method.as_str() {
            "telex" => vietc_engine::InputMethod::Telex,
            _ => vietc_engine::InputMethod::Vni,
        };
        match ibus_engine::run_ibus_engine(
            method,
            engine_enabled.clone(),
            false,
            daemon.config.deduplicate_keys,
            daemon.config.deduplicate_two_back,
            daemon.config.deduplicate_window_ms,
        ) {
            Ok(()) => {
                log_info("[vietc] IBus engine session ended");
                return Ok(());
            }
            Err(e) => log_info(&format!(
                "[vietc] IBus engine failed ({}); falling back to capture paths",
                e
            )),
        }
    }

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
            let status_path = config_path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .map(|p| p.join("status"))
                .or_else(|| dirs::config_dir().map(|d| d.join("vietc").join("status")))
                .unwrap_or_else(|| std::path::PathBuf::from("/tmp/vietc-status"));
            let method_path = config_path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .map(|p| p.join("method"))
                .or_else(|| dirs::config_dir().map(|d| d.join("vietc").join("method")))
                .unwrap_or_else(|| std::path::PathBuf::from("/tmp/vietc-method"));
            let mut last_method = String::new();
            loop {
                if let Some(id) = app_state::get_active_window_id() {
                    let mut lock = shared_active_window.lock().unwrap();
                    if *lock != id {
                        log_info(&format!("[vietc] bg: window ID '{}' -> '{}'", *lock, id));
                        *lock = id;
                    }
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

                if let Ok(content) = std::fs::read_to_string(&method_path) {
                    let m = content.trim();
                    if !m.is_empty() && m != last_method {
                        if !last_method.is_empty() {
                            status_changed.store(true, Ordering::SeqCst);
                        }
                        last_method = m.to_string();
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

    // Primary engine: hardware keyboard interception via evdev + uinput.
    // Handles all Wayland compositors (Hyprland, Sway, GNOME, KDE, River, etc.)
    // and X11 seamlessly. Rootless when in `input` group.
    match open_keyboard_devices() {
        Ok(mut devices) => {
            if !devices.is_empty() {
                stop_competing_imes();
                // Catch boot-time race where graphical-session starts fcitx5/ibus a few seconds after vietc
                thread::spawn(|| {
                    for _ in 0..2 {
                        for _ in 0..10 {
                            if crate::signal::SIGNAL_EXIT.load(Ordering::SeqCst) {
                                return;
                            }
                            thread::sleep(std::time::Duration::from_millis(500));
                        }
                        if !crate::signal::SIGNAL_EXIT.load(Ordering::SeqCst) {
                            stop_competing_imes();
                        }
                    }
                });
            }
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
                    log_info("[vietc] evdev session ended");
                    return Ok(());
                }
                Err(e) => {
                    log_info(&format!(
                        "[vietc] evdev exited with error: {} — trying fallbacks",
                        e
                    ));
                }
            }
        }
        Err(e) => {
            log_info(&format!("[vietc] evdev not available: {}", e));
        }
    }

    // Fallback on Wayland: compositor input-method protocol
    #[cfg(feature = "wayland")]
    if display == display::DisplayServer::Wayland {
        match wayland_im::run_wayland_im(&daemon.config, engine_enabled.clone(), display) {
            Ok(()) => {
                log_info("[vietc] Wayland input method session ended");
                return Ok(());
            }
            Err(e) => {
                log_info(&format!(
                    "[vietc] Wayland IM unavailable ({}), falling back",
                    e
                ));
            }
        }
    }

    // Fallback on X11: X11 keymap-based capture
    #[cfg(feature = "x11")]
    if std::env::var("DISPLAY").is_ok() {
        log_info("[vietc] Trying X11 keymap-based capture (rootless via XWayland)");
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
