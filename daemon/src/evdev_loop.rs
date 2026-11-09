use std::collections::{HashMap, HashSet};
use std::fs;
use std::os::unix::io::AsRawFd;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::daemon::Daemon;
use crate::display;
#[cfg(feature = "x11")]
use crate::display::DisplayServer;
use crate::event::{
    is_caps_lock_on, is_flush_char, is_modifier_held_shift, is_modifier_key,
    is_modifier_pressed, is_navigation_key, is_repeatable_navigation_key,
    is_toggle_combination_state, is_toggle_key, is_vn_control_key, key_to_char,
};
use crate::inject::{create_injector, execute_commands};
use crate::log::log_info;
use crate::signal::SIGNAL_EXIT;
use crate::commands::OutputCommand;

const CHORD_BOUNCE_WINDOW: Duration = Duration::from_millis(220);
const CHORD_RELEASE_BOUNCE_WINDOW: Duration = Duration::from_millis(80);
const CHORD_HOLD_REPEAT_DELAY: Duration = Duration::from_millis(650);
const CHORD_HOLD_REPEAT_INTERVAL: Duration = Duration::from_millis(120);
const NAV_PRESS_BOUNCE_WINDOW: Duration = Duration::from_millis(85);
const NAV_RELEASE_BOUNCE_WINDOW: Duration = Duration::from_millis(40);
const NAV_HOLD_REPEAT_DELAY: Duration = Duration::from_millis(400);
const NAV_HOLD_REPEAT_INTERVAL: Duration = Duration::from_millis(65);
const CROSS_INTERFACE_DEDUP_WINDOW: Duration = Duration::from_millis(50);
const TOGGLE_DEBOUNCE_WINDOW: Duration = Duration::from_millis(180);

struct DeviceState {
    key_state: evdev::AttributeSet<evdev::Key>,
    caps: bool,
    consumed_keys: HashSet<u16>,
    path: String,
    phys_id: Option<String>,
}

/// Compute aggregate held keys across sibling HID interfaces belonging to the
/// same physical keyboard (e.g. `input0` boot + modifiers and `input1` NKRO
/// bitmap without modifier bits on USB gaming keyboards).
fn effective_key_state(states: &[DeviceState], idx: usize) -> evdev::AttributeSet<evdev::Key> {
    let mut out = states[idx].key_state.clone();
    if let Some(ref pid) = states[idx].phys_id {
        for (j, st) in states.iter().enumerate() {
            if j != idx && st.phys_id.as_ref() == Some(pid) {
                for k in st.key_state.iter() {
                    out.insert(k);
                }
            }
        }
    }
    out
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ChordAction {
    ForwardRaw(u16, i32),
    AtomicTap(u16),
    Drop,
}

#[derive(Debug, Default)]
struct ChordGuard {
    last_chord: Option<(u16, Instant)>,
    last_chord_release: Option<(u16, Instant)>,
    last_chord_repeat: Option<Instant>,
}

impl ChordGuard {
    fn handle_modifier_chord(
        &mut self,
        key: evdev::Key,
        value: i32,
        consumed_keys: &mut HashSet<u16>,
        now: Instant,
    ) -> (ChordAction, bool) {
        let keycode = key.code();
        if is_modifier_key(key) {
            return if value == 1 || value == 0 {
                (ChordAction::ForwardRaw(keycode, value), false)
            } else {
                (ChordAction::Drop, false)
            };
        }

        if value == 1 {
            if let Some((prev_code, prev_t)) = self.last_chord {
                if prev_code == keycode
                    && now.saturating_duration_since(prev_t) < CHORD_BOUNCE_WINDOW
                {
                    consumed_keys.insert(keycode);
                    return (ChordAction::Drop, false);
                }
            }
            if let Some((rel_code, rel_t)) = self.last_chord_release {
                if rel_code == keycode
                    && now.saturating_duration_since(rel_t) < CHORD_RELEASE_BOUNCE_WINDOW
                {
                    consumed_keys.insert(keycode);
                    return (ChordAction::Drop, false);
                }
            }
            self.last_chord = Some((keycode, now));
            self.last_chord_repeat = None;
            if key_to_char(key).is_some() {
                consumed_keys.insert(keycode);
                return (ChordAction::AtomicTap(keycode), true);
            }
            return (ChordAction::ForwardRaw(keycode, 1), true);
        }

        if value == 2 {
            if key_to_char(key).is_some() {
                // Never autorepeat one-shot clipboard/selection/save/window chords (1 press = 1 action)
                if matches!(
                    key,
                    evdev::Key::KEY_C
                        | evdev::Key::KEY_V
                        | evdev::Key::KEY_X
                        | evdev::Key::KEY_A
                        | evdev::Key::KEY_S
                        | evdev::Key::KEY_T
                        | evdev::Key::KEY_N
                        | evdev::Key::KEY_W
                        | evdev::Key::KEY_Q
                        | evdev::Key::KEY_P
                        | evdev::Key::KEY_F
                        | evdev::Key::KEY_R
                ) {
                    return (ChordAction::Drop, false);
                }
                if let Some((prev_code, prev_t)) = self.last_chord {
                    if prev_code == keycode
                        && now.saturating_duration_since(prev_t) >= CHORD_HOLD_REPEAT_DELAY
                    {
                        let can_repeat = self
                            .last_chord_repeat
                            .map(|rt| now.saturating_duration_since(rt) >= CHORD_HOLD_REPEAT_INTERVAL)
                            .unwrap_or(true);
                        if can_repeat {
                            self.last_chord_repeat = Some(now);
                            return (ChordAction::AtomicTap(keycode), false);
                        }
                    }
                }
                return (ChordAction::Drop, false);
            }
            if consumed_keys.contains(&keycode) {
                return (ChordAction::Drop, false);
            }
            return (ChordAction::ForwardRaw(keycode, 2), false);
        }

        let was_consumed = consumed_keys.remove(&keycode);
        if key_to_char(key).is_some() {
            self.last_chord_release = Some((keycode, now));
        }
        if was_consumed {
            (ChordAction::Drop, false)
        } else {
            (ChordAction::ForwardRaw(keycode, 0), false)
        }
    }

    fn is_post_chord_bounce(&self, keycode: u16, now: Instant) -> bool {
        let within_press = self
            .last_chord
            .map(|(c, t)| c == keycode && now.saturating_duration_since(t) < CHORD_BOUNCE_WINDOW)
            .unwrap_or(false);
        let within_release = self
            .last_chord_release
            .map(|(c, t)| {
                c == keycode && now.saturating_duration_since(t) < CHORD_RELEASE_BOUNCE_WINDOW
            })
            .unwrap_or(false);
        within_press || within_release
    }
}

/// Ensures arrow and navigation keys move strictly one character/step per press
/// without mechanical switch chatter or Wayland/compositor `250ms`/`40Hz` double-repeat.
#[derive(Debug, Default)]
struct NavGuard {
    active_hold: Option<(usize, u16, Instant)>,
    last_accepted_press: Option<(u16, Instant)>,
    last_release: Option<(u16, Instant)>,
    last_repeat: Option<Instant>,
}

impl NavGuard {
    fn handle_nav_key(
        &mut self,
        dev_idx: usize,
        key: evdev::Key,
        value: i32,
        consumed_keys: &mut HashSet<u16>,
        now: Instant,
    ) -> (ChordAction, bool) {
        let keycode = key.code();
        if value == 1 {
            if let Some((prev_code, prev_t)) = self.last_accepted_press {
                if prev_code == keycode
                    && now.saturating_duration_since(prev_t) < NAV_PRESS_BOUNCE_WINDOW
                {
                    consumed_keys.insert(keycode);
                    return (ChordAction::Drop, false);
                }
            }
            if let Some((rel_code, rel_t)) = self.last_release {
                if rel_code == keycode
                    && now.saturating_duration_since(rel_t) < NAV_RELEASE_BOUNCE_WINDOW
                {
                    consumed_keys.insert(keycode);
                    return (ChordAction::Drop, false);
                }
            }
            self.active_hold = Some((dev_idx, keycode, now));
            self.last_accepted_press = Some((keycode, now));
            self.last_repeat = None;
            consumed_keys.insert(keycode);
            return (ChordAction::AtomicTap(keycode), true);
        }

        if value == 2 {
            if !is_repeatable_navigation_key(key) {
                return (ChordAction::Drop, false);
            }
            if let Some((hold_dev, hold_code, press_t)) = self.active_hold {
                if hold_dev == dev_idx
                    && hold_code == keycode
                    && now.saturating_duration_since(press_t) >= NAV_HOLD_REPEAT_DELAY
                {
                    let can_repeat = self
                        .last_repeat
                        .map(|rt| now.saturating_duration_since(rt) >= NAV_HOLD_REPEAT_INTERVAL)
                        .unwrap_or(true);
                    if can_repeat {
                        self.last_repeat = Some(now);
                        return (ChordAction::AtomicTap(keycode), false);
                    }
                }
            }
            return (ChordAction::Drop, false);
        }

        let was_consumed = consumed_keys.remove(&keycode);
        if self
            .active_hold
            .map(|(d, c, _)| d == dev_idx && c == keycode)
            .unwrap_or(false)
        {
            self.active_hold = None;
            self.last_repeat = None;
            self.last_release = Some((keycode, now));
        } else if self
            .last_release
            .map(|(c, t)| c == keycode && now.saturating_duration_since(t) < Duration::from_millis(120))
            .unwrap_or(false)
        {
            self.last_release = Some((keycode, now));
        }

        if was_consumed {
            (ChordAction::Drop, false)
        } else {
            (ChordAction::ForwardRaw(keycode, 0), false)
        }
    }
}

#[derive(Debug, Default)]
struct CrossInterfaceDedup {
    recent_press: HashMap<(String, u16), (String, Instant)>,
}

impl CrossInterfaceDedup {
    fn should_drop(
        &mut self,
        phys_id: Option<&str>,
        dev_path: &str,
        keycode: u16,
        now: Instant,
    ) -> bool {
        let Some(pid) = phys_id else {
            return false;
        };
        let key = (pid.to_string(), keycode);
        if let Some((prev_path, prev_t)) = self.recent_press.get(&key) {
            if prev_path != dev_path
                && now.saturating_duration_since(*prev_t) < CROSS_INTERFACE_DEDUP_WINDOW
            {
                return true;
            }
        }
        self.recent_press.insert(key, (dev_path.to_string(), now));
        false
    }
}

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

    // Track currently opened device paths so newly connected keyboards can be hotplugged without duplicates
    let mut known_paths: HashSet<String> = HashSet::new();
    for (_, n) in devices.iter() {
        known_paths.insert(dev_path_of(n));
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

    let mut device_states: Vec<DeviceState> = devices
        .iter()
        .map(|(d, n)| {
            let path = dev_path_of(n);
            let phys_id = crate::device::get_device_phys_id(std::path::Path::new(&path));
            DeviceState {
                key_state: evdev::AttributeSet::new(),
                caps: is_caps_lock_on(d),
                consumed_keys: HashSet::new(),
                path,
                phys_id,
            }
        })
        .collect();

    let mut dedup = crate::key_dedup::DedupState::new();
    let mut chord_guard = ChordGuard::default();
    let mut nav_guard = NavGuard::default();
    let mut cross_dedup = CrossInterfaceDedup::default();
    let mut last_toggle_time: Option<Instant> = None;
    let mut last_active_window = String::new();
    let mut last_window_class = String::new();
    let mut password_check_counter: u32 = 0;
    let mut last_key_time = Instant::now();

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
            let new_devs = discover_new_keyboards(&known_paths);
            for (mut dev, name) in new_devs {
                let path = dev_path_of(&name);
                known_paths.insert(path.clone());
                if daemon.grab_enabled {
                    if dev.grab().is_ok() {
                        log_info(&format!("[vietc] Hotplug grabbed keyboard: {}", name));
                    }
                }
                let caps = is_caps_lock_on(&dev);
                let phys_id = crate::device::get_device_phys_id(std::path::Path::new(&path));
                log_info(&format!("[vietc] Hotplug connected keyboard: {}", name));
                devices.push((dev, name.clone()));
                device_states.push(DeviceState {
                    key_state: evdev::AttributeSet::new(),
                    caps,
                    consumed_keys: HashSet::new(),
                    path,
                    phys_id,
                });
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
            let caps = device_states[i].caps;

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
                let now = Instant::now();

                // Update key state dynamically on this HID endpoint
                if value == 1 {
                    device_states[i].key_state.insert(key);
                } else if value == 0 {
                    device_states[i].key_state.remove(key);
                }

                // Drop duplicate key press echoed across a sibling HID interface of the same physical keyboard
                if value == 1
                    && cross_dedup.should_drop(
                        device_states[i].phys_id.as_deref(),
                        &device_states[i].path,
                        keycode,
                        now,
                    )
                {
                    if debug_logging {
                        log_info(&format!(
                            "[vietc] cross-interface dedup dropped keycode {} on {}",
                            keycode, device_states[i].path
                        ));
                    }
                    device_states[i].consumed_keys.insert(keycode);
                    continue;
                }

                let eff_key_state = effective_key_state(&device_states, i);

                // Completely bypass all IME processing for terminal emulators, IDE terminals, and games
                if daemon.is_current_app_bypassed() {
                    if grabbed {
                        if value == 0 && device_states[i].consumed_keys.remove(&keycode) {
                            continue;
                        }
                        injector.send_key_event(keycode, value);
                    }
                    continue;
                }

                if value == 1
                    && is_toggle_key(key, &daemon.config.toggle_key)
                    && is_toggle_combination_state(&eff_key_state, &daemon.config.toggle_key)
                {
                    device_states[i].consumed_keys.insert(keycode);
                    let debounced = last_toggle_time
                        .map(|t| now.saturating_duration_since(t) < TOGGLE_DEBOUNCE_WINDOW)
                        .unwrap_or(false);
                    if !debounced {
                        last_toggle_time = Some(now);
                        daemon.toggle_method();
                    }
                    continue;
                }

                if value == 1 && daemon.config.deduplicate_keys {
                    let dedupable = keycode < 256 && !is_modifier_key(key);
                    if dedup.observe(
                        keycode as u32,
                        dedupable,
                        keycode == 57,
                        daemon.config.deduplicate_two_back,
                        daemon.config.deduplicate_window_ms,
                        now,
                    ) {
                        if debug_logging {
                            log_info(&format!(
                                "[vietc] dedup dropped keycode {} (window {}ms)",
                                keycode, daemon.config.deduplicate_window_ms
                            ));
                        }
                        device_states[i].consumed_keys.insert(keycode);
                        continue;
                    }
                }

                if !grabbed {
                    if value != 1 {
                        continue;
                    }
                    if is_modifier_pressed(&eff_key_state) || is_navigation_key(key) {
                        daemon.engine.reset();
                        daemon.replay_reset();
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
                    if is_navigation_key(key) {
                        let (action, reset_engine) = nav_guard.handle_nav_key(
                            i,
                            key,
                            value,
                            &mut device_states[i].consumed_keys,
                            now,
                        );
                        if reset_engine {
                            daemon.engine.reset();
                            daemon.replay_reset();
                            dedup.reset();
                        }
                        match action {
                            ChordAction::ForwardRaw(code, val) => {
                                injector.send_key_event(code, val);
                            }
                            ChordAction::AtomicTap(code) => {
                                injector.send_key_event(code, 1);
                                injector.send_key_event(code, 0);
                            }
                            ChordAction::Drop => {}
                        }
                        continue;
                    }

                    if is_modifier_pressed(&eff_key_state) {
                        let (action, reset_engine) = chord_guard.handle_modifier_chord(
                            key,
                            value,
                            &mut device_states[i].consumed_keys,
                            now,
                        );
                        if reset_engine {
                            daemon.engine.reset();
                            daemon.replay_reset();
                            dedup.reset();
                        }
                        match action {
                            ChordAction::ForwardRaw(code, val) => {
                                injector.send_key_event(code, val);
                            }
                            ChordAction::AtomicTap(code) => {
                                injector.send_key_event(code, 1);
                                injector.send_key_event(code, 0);
                            }
                            ChordAction::Drop => {}
                        }
                        continue;
                    }

                    // Guard against switch bounce on key-up right after a modifier chord
                    // (e.g. releasing Ctrl slightly before C/V in Ctrl+C / Ctrl+V)
                    if value == 1 && chord_guard.is_post_chord_bounce(keycode, now) {
                        if debug_logging {
                            log_info(&format!(
                                "[vietc] dropped post-chord bounce keycode {}",
                                keycode
                            ));
                        }
                        device_states[i].consumed_keys.insert(keycode);
                        continue;
                    }

                    // Engine disabled in grabbed mode: forward printable keys as
                    // atomic taps so no key can ever stick down in the compositor
                    if !daemon.engine.is_enabled() {
                        if key_to_char(key).is_some() {
                            if value == 1 || value == 2 {
                                device_states[i].consumed_keys.insert(keycode);
                                injector.send_key_event(keycode, 1);
                                injector.send_key_event(keycode, 0);
                            } else if value == 0 {
                                device_states[i].consumed_keys.remove(&keycode);
                            }
                        } else if (value == 0 || value == 2)
                            && device_states[i].consumed_keys.contains(&keycode)
                        {
                            if value == 0 {
                                device_states[i].consumed_keys.remove(&keycode);
                            }
                        } else {
                            injector.send_key_event(keycode, value);
                        }
                        continue;
                    }

                    if key == evdev::Key::KEY_BACKSPACE {
                        if value == 1 || value == 2 {
                            daemon.engine.process_key('\x08');
                            injector.send_key_event(14, 1);
                            injector.send_key_event(14, 0);
                        }
                        device_states[i].consumed_keys.insert(keycode);
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
                        device_states[i].consumed_keys.remove(&keycode);
                        if let Some(mut ch) = key_to_char(key) {
                            let gap = last_key_time.elapsed();
                            last_key_time = now;

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

                            let shift = is_modifier_held_shift(&eff_key_state);
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
                                device_states[i].consumed_keys.insert(keycode);
                                // Temporarily release Shift on uinput during Backspace + Unicode replacement
                                // so Shift+BackSpace or shifted wtype keymap never corrupts output.
                                let lshift = eff_key_state.contains(evdev::Key::KEY_LEFTSHIFT);
                                let rshift = eff_key_state.contains(evdev::Key::KEY_RIGHTSHIFT);
                                if lshift {
                                    injector.send_key_event(42, 0);
                                }
                                if rshift {
                                    injector.send_key_event(54, 0);
                                }
                                execute_commands(&*injector, &commands);
                                if lshift {
                                    injector.send_key_event(42, 1);
                                }
                                if rshift {
                                    injector.send_key_event(54, 1);
                                }
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
                                device_states[i].consumed_keys.insert(keycode);
                            } else {
                                if debug_logging {
                                    log_info(&format!(
                                        "[vietc] grabbed forward: ch='{}' (no commands)",
                                        ch.escape_default(),
                                    ));
                                }
                                // Send atomic tap (down + up) so printable keys are never left
                                // held down on uinput during fast rollover or Wayland key-repeat
                                device_states[i].consumed_keys.insert(keycode);
                                injector.send_key_event(keycode, 1);
                                injector.send_key_event(keycode, 0);
                            }
                        } else {
                            injector.send_key_event(keycode, 1);
                        }
                    } else if value == 2 {
                        if let Some(ch) = key_to_char(key) {
                            // Allow intentional key-hold repeat for flush/navigation chars (Space, Enter, '-', '=', '.')
                            if is_flush_char(ch) {
                                daemon.engine.reset();
                                injector.send_key_event(keycode, 1);
                                injector.send_key_event(keycode, 0);
                            }
                            continue;
                        }
                        if device_states[i].consumed_keys.contains(&keycode) {
                            continue;
                        }
                        injector.send_key_event(keycode, 2);
                    } else if value == 0 {
                        if device_states[i].consumed_keys.remove(&keycode) {
                            continue;
                        }
                        injector.send_key_event(keycode, 0);
                    }
                }
            }
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

/// Discover keyboard devices present in /dev/input that are not already known.
/// Used for hotplug so newly connected keyboards (USB/wireless/virtual) are attached dynamically.
fn discover_new_keyboards(existing_paths: &HashSet<String>) -> Vec<(evdev::Device, String)> {
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
                        if existing_paths.contains(&p) || !added_paths.insert(p) {
                            continue;
                        }
                        if let Ok(device) = evdev::Device::open(&real_path) {
                            let dev_name = device.name().unwrap_or("unknown").to_string();
                            if crate::device::is_valid_keyboard(&device) {
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
                        if existing_paths.contains(&p) || !added_paths.insert(p) {
                            continue;
                        }
                        if let Ok(device) = evdev::Device::open(&real_path) {
                            let dev_name = device.name().unwrap_or("unknown").to_string();
                            if crate::device::is_valid_keyboard(&device) {
                                out.push((device, format!("{} ({})", real_path.display(), dev_name)));
                            }
                        }
                    }
                }
            }
        }
    }

    // 3. Scan /dev/input/event* for Bluetooth, NKRO secondary interfaces, & direct devices
    let dir = std::path::Path::new("/dev/input");
    if dir.exists() {
        if let Ok(rd) = fs::read_dir(dir) {
            for entry in rd.flatten() {
                let name_str = entry.file_name().to_string_lossy().to_string();
                if name_str.starts_with("event") {
                    if let Ok(real_path) = fs::canonicalize(entry.path()) {
                        let p = real_path.to_string_lossy().to_string();
                        if existing_paths.contains(&p) || !added_paths.insert(p) {
                            continue;
                        }
                        if let Ok(device) = evdev::Device::open(&real_path) {
                            let dev_name = device.name().unwrap_or("unknown").to_string();
                            if crate::device::is_valid_keyboard(&device) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ctrl_c_and_ctrl_v_emit_single_atomic_tap_and_drop_bounce() {
        let mut guard = ChordGuard::default();
        let mut consumed = HashSet::new();
        let t0 = Instant::now();

        // Ctrl down forwards raw
        assert_eq!(
            guard.handle_modifier_chord(evdev::Key::KEY_LEFTCTRL, 1, &mut consumed, t0),
            (ChordAction::ForwardRaw(evdev::Key::KEY_LEFTCTRL.code(), 1), false)
        );

        // First C press -> single AtomicTap + reset engine
        assert_eq!(
            guard.handle_modifier_chord(
                evdev::Key::KEY_C,
                1,
                &mut consumed,
                t0 + Duration::from_millis(20)
            ),
            (ChordAction::AtomicTap(evdev::Key::KEY_C.code()), true)
        );

        // Switch bounce on C 15ms later while Ctrl still held -> dropped!
        assert_eq!(
            guard.handle_modifier_chord(
                evdev::Key::KEY_C,
                0,
                &mut consumed,
                t0 + Duration::from_millis(30)
            ),
            (ChordAction::Drop, false)
        );
        assert_eq!(
            guard.handle_modifier_chord(
                evdev::Key::KEY_C,
                1,
                &mut consumed,
                t0 + Duration::from_millis(38)
            ),
            (ChordAction::Drop, false)
        );

        // Holding Ctrl+C past 250ms repeat_delay never repeats Copy
        assert_eq!(
            guard.handle_modifier_chord(
                evdev::Key::KEY_C,
                2,
                &mut consumed,
                t0 + Duration::from_millis(300)
            ),
            (ChordAction::Drop, false)
        );
        assert_eq!(
            guard.handle_modifier_chord(
                evdev::Key::KEY_C,
                0,
                &mut consumed,
                t0 + Duration::from_millis(310)
            ),
            (ChordAction::Drop, false)
        );

        // Ctrl+V -> single AtomicTap, no double paste at 250ms, post-chord bounce after Ctrl up is dropped
        let tv = t0 + Duration::from_millis(500);
        assert_eq!(
            guard.handle_modifier_chord(evdev::Key::KEY_V, 1, &mut consumed, tv),
            (ChordAction::AtomicTap(evdev::Key::KEY_V.code()), true)
        );
        // Repeat at 260ms AND 900ms is dropped (Ctrl+V never autorepeats multiple copies)
        assert_eq!(
            guard.handle_modifier_chord(
                evdev::Key::KEY_V,
                2,
                &mut consumed,
                tv + Duration::from_millis(260)
            ),
            (ChordAction::Drop, false)
        );
        assert_eq!(
            guard.handle_modifier_chord(
                evdev::Key::KEY_V,
                2,
                &mut consumed,
                tv + Duration::from_millis(900)
            ),
            (ChordAction::Drop, false)
        );
        // Release V at +920ms, then mechanical break-bounce on V at +940ms -> dropped!
        assert_eq!(
            guard.handle_modifier_chord(
                evdev::Key::KEY_V,
                0,
                &mut consumed,
                tv + Duration::from_millis(920)
            ),
            (ChordAction::Drop, false)
        );
        assert_eq!(
            guard.handle_modifier_chord(
                evdev::Key::KEY_V,
                1,
                &mut consumed,
                tv + Duration::from_millis(940)
            ),
            (ChordAction::Drop, false)
        );
        // User lifts Ctrl at +70ms, then V bounces at +95ms -> post-chord guard drops trailing 'v'
        let tv2 = tv + Duration::from_millis(1000);
        assert_eq!(
            guard.handle_modifier_chord(evdev::Key::KEY_V, 1, &mut consumed, tv2),
            (ChordAction::AtomicTap(evdev::Key::KEY_V.code()), true)
        );
        assert!(guard.is_post_chord_bounce(
            evdev::Key::KEY_V.code(),
            tv2 + Duration::from_millis(60)
        ));
        assert!(!guard.is_post_chord_bounce(
            evdev::Key::KEY_V.code(),
            tv2 + Duration::from_millis(250)
        ));
    }

    #[test]
    fn cross_interface_dedup_drops_sibling_echo_but_keeps_same_device_double_tap() {
        let mut dedup = CrossInterfaceDedup::default();
        let t0 = Instant::now();

        // Rapid Telex double-tap 'e' + 'e' on the SAME interface (/dev/input/event7) is NOT dropped
        assert!(!dedup.should_drop(Some("phys:usb-1"), "/dev/input/event7", 18, t0));
        assert!(!dedup.should_drop(
            Some("phys:usb-1"),
            "/dev/input/event7",
            18,
            t0 + Duration::from_millis(10)
        ));

        // Simultaneous echo on sibling NKRO interface (/dev/input/event10) within 50ms IS dropped
        assert!(dedup.should_drop(
            Some("phys:usb-1"),
            "/dev/input/event10",
            18,
            t0 + Duration::from_millis(14)
        ));

        // Different physical keyboard is NOT dropped
        assert!(!dedup.should_drop(
            Some("phys:usb-2"),
            "/dev/input/event6",
            18,
            t0 + Duration::from_millis(15)
        ));
    }

    #[test]
    fn nav_guard_steps_arrow_keys_one_by_one_and_drops_switch_bounce() {
        let mut nav = NavGuard::default();
        let mut consumed = HashSet::new();
        let t0 = Instant::now();
        let left = evdev::Key::KEY_LEFT;

        // 1. Initial press -> single AtomicTap + resets engine
        assert_eq!(
            nav.handle_nav_key(0, left, 1, &mut consumed, t0),
            (ChordAction::AtomicTap(left.code()), true)
        );

        // 2. Make-bounce within 85ms -> dropped
        assert_eq!(
            nav.handle_nav_key(0, left, 0, &mut consumed, t0 + Duration::from_millis(10)),
            (ChordAction::Drop, false)
        );
        assert_eq!(
            nav.handle_nav_key(0, left, 1, &mut consumed, t0 + Duration::from_millis(22)),
            (ChordAction::Drop, false)
        );
        assert_eq!(
            nav.handle_nav_key(0, left, 0, &mut consumed, t0 + Duration::from_millis(30)),
            (ChordAction::Drop, false)
        );

        // 3. Brisk intentional second tap (+140ms, >40ms after release) -> steps cleanly 1 char
        let t1 = t0 + Duration::from_millis(140);
        assert_eq!(
            nav.handle_nav_key(0, left, 1, &mut consumed, t1),
            (ChordAction::AtomicTap(left.code()), true)
        );

        // 4. Casual 320ms press crosses kernel 250ms repeat_delay -> still moves ONLY 1 char!
        assert_eq!(
            nav.handle_nav_key(0, left, 2, &mut consumed, t1 + Duration::from_millis(250)),
            (ChordAction::Drop, false)
        );
        assert_eq!(
            nav.handle_nav_key(0, left, 2, &mut consumed, t1 + Duration::from_millis(320)),
            (ChordAction::Drop, false)
        );
        assert_eq!(
            nav.handle_nav_key(0, left, 0, &mut consumed, t1 + Duration::from_millis(330)),
            (ChordAction::Drop, false)
        );

        // 5. Break-bounce 15ms after release -> dropped!
        assert_eq!(
            nav.handle_nav_key(0, left, 1, &mut consumed, t1 + Duration::from_millis(345)),
            (ChordAction::Drop, false)
        );
        assert_eq!(
            nav.handle_nav_key(0, left, 0, &mut consumed, t1 + Duration::from_millis(352)),
            (ChordAction::Drop, false)
        );

        // 6. Different arrow key (KEY_DOWN) 10ms later -> accepted immediately with zero lag
        let down = evdev::Key::KEY_DOWN;
        assert_eq!(
            nav.handle_nav_key(0, down, 1, &mut consumed, t1 + Duration::from_millis(362)),
            (ChordAction::AtomicTap(down.code()), true)
        );
        assert_eq!(
            nav.handle_nav_key(0, down, 0, &mut consumed, t1 + Duration::from_millis(400)),
            (ChordAction::Drop, false)
        );
    }

    #[test]
    fn nav_guard_repeats_smoothly_on_long_hold_and_never_repeats_home() {
        let mut nav = NavGuard::default();
        let mut consumed = HashSet::new();
        let t0 = Instant::now();
        let right = evdev::Key::KEY_RIGHT;

        assert_eq!(
            nav.handle_nav_key(0, right, 1, &mut consumed, t0),
            (ChordAction::AtomicTap(right.code()), true)
        );
        // At 410ms (>= 400ms hold delay): first smooth repeat fires
        assert_eq!(
            nav.handle_nav_key(0, right, 2, &mut consumed, t0 + Duration::from_millis(410)),
            (ChordAction::AtomicTap(right.code()), false)
        );
        // Next kernel tick 33ms later (443ms, < 65ms interval): throttled
        assert_eq!(
            nav.handle_nav_key(0, right, 2, &mut consumed, t0 + Duration::from_millis(443)),
            (ChordAction::Drop, false)
        );
        // Next kernel tick at 476ms (66ms >= 65ms interval): second smooth repeat fires
        assert_eq!(
            nav.handle_nav_key(0, right, 2, &mut consumed, t0 + Duration::from_millis(476)),
            (ChordAction::AtomicTap(right.code()), false)
        );
        assert_eq!(
            nav.handle_nav_key(0, right, 0, &mut consumed, t0 + Duration::from_millis(500)),
            (ChordAction::Drop, false)
        );

        // One-shot navigation keys (Home, Esc) never autorepeat on hold
        let t2 = t0 + Duration::from_millis(700);
        let home = evdev::Key::KEY_HOME;
        assert_eq!(
            nav.handle_nav_key(0, home, 1, &mut consumed, t2),
            (ChordAction::AtomicTap(home.code()), true)
        );
        assert_eq!(
            nav.handle_nav_key(0, home, 2, &mut consumed, t2 + Duration::from_millis(600)),
            (ChordAction::Drop, false)
        );
    }
}
