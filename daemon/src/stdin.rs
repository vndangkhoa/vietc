use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::daemon::Daemon;
use crate::device::open_keyboard_devices;
use crate::display;
use crate::inject::{create_injector, execute_commands};
use crate::log::log_info;

pub fn run_stdin_mode(
    daemon: &mut Daemon,
    shared_active_window: Arc<Mutex<String>>,
    shared_window_class: Arc<Mutex<String>>,
    config_changed: Arc<std::sync::atomic::AtomicBool>,
    status_changed: Arc<std::sync::atomic::AtomicBool>,
    _engine_enabled: Arc<std::sync::atomic::AtomicBool>,
    display: display::DisplayServer,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, IsTerminal, Read};

    if !io::stdin().is_terminal() {
        log_info("[vietc] Warning: No keyboard device and no terminal.");
        log_info("[vietc] Retrying keyboard access every 5 seconds...");
        log_info("[vietc] Ensure you are in the 'input' group:");
        log_info("      sudo usermod -aG input $USER");
        log_info("  Then log out and back in.");

        loop {
            thread::sleep(std::time::Duration::from_secs(5));

            if status_changed.load(Ordering::SeqCst) {
                daemon.sync_status_file();
                status_changed.store(false, Ordering::SeqCst);
            }
            if config_changed.load(Ordering::SeqCst) {
                daemon.reload_config();
                config_changed.store(false, Ordering::SeqCst);
            }

            if let Ok(mut devices) = open_keyboard_devices() {
                log_info(&format!("[vietc] Keyboard device(s) found: {}", devices.len()));
                return crate::evdev_loop::run_with_evdev(
                    &mut devices,
                    daemon,
                    shared_active_window,
                    shared_window_class,
                    config_changed,
                    status_changed,
                    _engine_enabled,
                    display,
                );
            }
        }
    }

    let injector = create_injector(display)?;
    let mut buffer = [0u8; 1];
    let mut last_active_window = String::new();

    log_info("[vietc] Type to test, Ctrl+C to exit");

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    loop {
        if status_changed.load(Ordering::SeqCst) {
            daemon.sync_status_file();
            status_changed.store(false, Ordering::SeqCst);
        }

        {
            let active_window = shared_active_window.lock().unwrap().clone();
            if active_window != last_active_window {
                log_info(&format!(
                    "[vietc] Window changed: '{}' -> '{}'",
                    last_active_window, active_window
                ));
                last_active_window = active_window.clone();
                daemon.engine.reset();
                log_info("[vietc] Reset engine buffer due to window change");
            }
        }

        if daemon.config.app_state.enabled {
            let active_window = shared_active_window.lock().unwrap().clone();
            injector.set_active_window(&active_window);
            daemon.check_app_change_with(active_window);
        }

        if config_changed.load(Ordering::SeqCst) {
            daemon.reload_config();
            config_changed.store(false, Ordering::SeqCst);
        }

        match handle.read(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {
                let ch = buffer[0] as char;
                let commands = daemon.process_key(ch);
                execute_commands(&*injector, &commands, false);
            }
            Err(e) => {
                log_info(&format!("[vietc] Read error: {}", e));
                break;
            }
        }
    }

    Ok(())
}
