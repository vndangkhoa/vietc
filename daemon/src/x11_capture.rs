use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use crate::commands::OutputCommand;
use crate::daemon::Daemon;
use crate::display;
use crate::inject::{create_injector, execute_commands};
use crate::log::log_info;
use crate::signal::SIGNAL_EXIT;

#[cfg(feature = "x11")]
pub fn run_with_x11(
    mut capture: vietc_protocol::x11_capture::X11Capture,
    daemon: &mut Daemon,
    shared_active_window: Arc<Mutex<String>>,
    shared_window_class: Arc<Mutex<String>>,
    config_changed: Arc<std::sync::atomic::AtomicBool>,
    status_changed: Arc<std::sync::atomic::AtomicBool>,
    _engine_enabled: Arc<std::sync::atomic::AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let injector: Box<dyn vietc_protocol::KeyInjector> = Box::new(
        vietc_protocol::x11_inject::X11Injector::new()?
    );
    let mut last_active_window = String::new();
    let mut pressed_keys: HashSet<u32> = HashSet::new();

    use std::io::Write;
    let _ = std::io::stderr().write_all(b"[vietc] X11 event loop starting\n");
    std::io::stderr().flush().ok();

    loop {
        let _ = std::io::stderr().write_all(b"[vietc] X11: check status_changed\n");
        std::io::stderr().flush().ok();
        if status_changed.load(Ordering::SeqCst) {
            daemon.sync_status_file();
            status_changed.store(false, Ordering::SeqCst);
        }

        let _ = std::io::stderr().write_all(b"[vietc] X11: check config_changed\n");
        std::io::stderr().flush().ok();
        if config_changed.load(Ordering::SeqCst) {
            daemon.reload_config();
            config_changed.store(false, Ordering::SeqCst);
        }

        let _ = std::io::stderr().write_all(b"[vietc] X11: lock active_window\n");
        std::io::stderr().flush().ok();
        {
            let active_window = shared_active_window.lock().unwrap().clone();
            if active_window != last_active_window {
                last_active_window = active_window.clone();
                daemon.replay_reset();
            }
        }

        let _ = std::io::stderr().write_all(b"[vietc] X11: lock window_class\n");
        std::io::stderr().flush().ok();
        if daemon.config.app_state.enabled {
            let class = shared_window_class.lock().unwrap().clone();
            if !class.is_empty() {
                injector.set_active_window(&class);
                daemon.check_app_change_with(class);
            }
        }

        if capture.focus_lost {
            daemon.replay_reset();
            pressed_keys.clear();
            capture.focus_lost = false;
        }

        let _ = std::io::stderr().write_all(b"[vietc] X11: wait_for_event\n");
        std::io::stderr().flush().ok();
        let _got_data = capture.wait_for_event(100);
        vietc_protocol::x11_capture::SKIP_RECORD_EVENTS.store(false, Ordering::Relaxed);
        let _ = std::io::stderr().write_all(b"[vietc] X11: next_event\n");
        std::io::stderr().flush().ok();
        let evt = capture.next_event();
        if evt.is_none() {
            continue;
        }
        let event = evt.unwrap();

        {
            if event.pressed {
                if !pressed_keys.insert(event.keycode) {
                    continue;
                }

                if let Some(' ') = event.ch {
                    if (event.state & 4) != 0 {
                        pressed_keys.remove(&event.keycode);
                        daemon.replay_reset();
                        daemon.toggle();
                        continue;
                    }
                }

                if capture.is_modifier_pressed(event.state) || event.ch.is_none() {
                    daemon.replay_reset();
                    vietc_protocol::x11_capture::SKIP_RECORD_EVENTS.store(true, Ordering::Relaxed);
                    let _ = injector.send_key_event(event.keycode as u16, 1);
                    continue;
                }

                if let Some(ch) = event.ch {
                    match ch {
                        '\x08' => {
                            let commands = daemon.replay_backspace();
                            pressed_keys.remove(&event.keycode);
                            vietc_protocol::x11_capture::SKIP_RECORD_EVENTS.store(true, Ordering::Relaxed);
                            execute_commands(&*injector, &commands);
                            if daemon.event_store.is_empty() && commands.is_empty() {
                                let _ = injector.send_backspace();
                            }
                        }
                        '\n' => {
                            pressed_keys.remove(&event.keycode);
                            daemon.replay_reset();
                            vietc_protocol::x11_capture::SKIP_RECORD_EVENTS.store(true, Ordering::Relaxed);
                            let _ = injector.send_key_event(event.keycode as u16, 1);
                            let _ = injector.send_key_event(event.keycode as u16, 0);
                        }
                        _ => {
                            let commands = daemon.replay_and_inject(ch);
                            pressed_keys.remove(&event.keycode);
                            vietc_protocol::x11_capture::SKIP_RECORD_EVENTS.store(true, Ordering::Relaxed);
                            execute_commands(&*injector, &commands);
                        }
                    }
                }
            } else {
                if pressed_keys.remove(&event.keycode) {
                    vietc_protocol::x11_capture::SKIP_RECORD_EVENTS.store(true, Ordering::Relaxed);
                    let _ = injector.send_key_event(event.keycode as u16, 0);
                }
            }
        }
    }
}

#[cfg(feature = "x11")]
pub fn run_with_x11_keymap(
    daemon: &mut Daemon,
    shared_active_window: Arc<Mutex<String>>,
    shared_window_class: Arc<Mutex<String>>,
    config_changed: Arc<std::sync::atomic::AtomicBool>,
    status_changed: Arc<std::sync::atomic::AtomicBool>,
    _engine_enabled: Arc<std::sync::atomic::AtomicBool>,
    display: display::DisplayServer,
) -> Result<(), Box<dyn std::error::Error>> {
    use vietc_protocol::x11_inject::X11KeymapCapture;
    use vietc_protocol::x11_inject::set_keymap_suppress;

    let mut capture = X11KeymapCapture::new()?;
    let injector = create_injector(display)?;
    let mut last_active_window = String::new();
    let mut last_window_class = String::new();
    let mut key_state: HashSet<u32> = HashSet::new();

    // In keymap mode the original keystrokes stay on screen, so screen
    // accounting must use the raw on-screen length. This flag also drives the
    // feedback suppression while we inject our own keystrokes.
    daemon.keymap_mode = true;

    log_info("[vietc] X11 keymap capture active");
    loop {
        if SIGNAL_EXIT.load(Ordering::SeqCst) {
            log_info("[vietc] Exiting on signal");
            return Ok(());
        }

        if status_changed.load(Ordering::SeqCst) {
            daemon.sync_status_file();
            status_changed.store(false, Ordering::SeqCst);
        }
        if config_changed.load(Ordering::SeqCst) {
            daemon.reload_config();
            config_changed.store(false, Ordering::SeqCst);
        }

        {
            let active_window = shared_active_window.lock().unwrap().clone();
            if active_window != last_active_window {
                last_active_window = active_window.clone();
                daemon.engine.reset();
                daemon.replay_reset();
            }
        }
        if daemon.config.app_state.enabled {
            let class = shared_window_class.lock().unwrap().clone();
            if !class.is_empty() && class != last_window_class {
                last_window_class = class.clone();
                injector.set_active_window(&class);
                daemon.check_app_change_with(class.clone());
            }
        }

        let events = capture.poll();
        if events.is_empty() {
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        }

        for (keycode, pressed) in &events {
            if *pressed {
                key_state.insert(*keycode);
            } else {
                key_state.remove(keycode);
            }
        }

        for (keycode, pressed) in &events {
            if !*pressed {
                continue;
            }
            let keycode = *keycode;

            let shift_pressed = key_state.contains(&42) || key_state.contains(&54);
            let ctrl_pressed = key_state.contains(&29) || key_state.contains(&97);
            let alt_pressed = key_state.contains(&56) || key_state.contains(&100);
            let caps_state = key_state.contains(&58);

            let mut mod_state = 0i32;
            if shift_pressed { mod_state |= 1; }
            if caps_state { mod_state |= 2; }
            if ctrl_pressed { mod_state |= 4; }
            if alt_pressed { mod_state |= 8; }

            let is_mod = ctrl_pressed || alt_pressed || key_state.contains(&125);

            if is_mod {
                continue;
            }

            if ctrl_pressed && keycode == 57 {
                daemon.toggle();
                continue;
            }

            if ctrl_pressed && shift_pressed {
                daemon.toggle_method();
                continue;
            }

            if daemon.config.app_state.enabled {
                let is_pw = daemon.app_state.check_password_field();
                let currently_enabled = daemon.engine.is_enabled();
                if is_pw && currently_enabled {
                    daemon.engine.set_enabled(false);
                    daemon.engine.reset();
                    daemon.replay_reset();
                    daemon.write_status();
                } else if !is_pw && !currently_enabled && daemon.config.start_enabled {
                    let default_state = daemon.app_state.get_default_state();
                    if default_state {
                        daemon.engine.set_enabled(true);
                        daemon.engine.reset();
                        daemon.replay_reset();
                        daemon.write_status();
                    }
                }
            }

            if !daemon.engine.is_enabled() {
                continue;
            }

            // Backspace (evdev 14) is not returned by lookup_keycode, so handle
            // it explicitly: record it in the engine and correct the screen.
            if *pressed && keycode == 14 {
                let mut commands = daemon.replay_backspace();
                if !commands.is_empty() {
                    set_keymap_suppress(true);
                    execute_commands(&*injector, &commands);
                    // Keep suppressing while our injected keystrokes play out,
                    // resyncing the keymap baseline each poll so they are never
                    // re-captured. 200ms comfortably covers a short correction.
                    for _ in 0..40 {
                        std::thread::sleep(std::time::Duration::from_millis(5));
                        capture.poll();
                    }
                    set_keymap_suppress(false);
                }
                continue;
            }

            if let Some(ch) = capture.lookup_keycode(keycode, mod_state) {
                let mut commands = daemon.replay_and_inject(ch);

                if !commands.is_empty() {
                    // Inject our output, but suppress re-capturing the keystrokes
                    // we generate (backspace + typed result), which would feed
                    // back into the engine and corrupt the text.
                    set_keymap_suppress(true);
                    execute_commands(&*injector, &commands);
                    // Resync the keymap baseline so the injected keystrokes are
                    // discarded rather than re-reported as new events.
                    // Keep suppressing while our injected keystrokes play out,
                    // resyncing the keymap baseline each poll so they are never
                    // re-captured. 200ms comfortably covers a short correction.
                    for _ in 0..40 {
                        std::thread::sleep(std::time::Duration::from_millis(5));
                        capture.poll();
                    }
                    set_keymap_suppress(false);
                }
            }
        }
    }
}
