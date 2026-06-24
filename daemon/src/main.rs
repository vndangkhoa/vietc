use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use vietc_engine::{Engine, EngineEvent, InputMethod};

mod config;
mod app_state;
mod display;

use config::Config;
use app_state::AppStateManager;

struct Daemon {
    engine: Engine,
    config: Config,
    config_path: PathBuf,
    config_modified: std::time::SystemTime,
    app_state: AppStateManager,
    engine_enabled: Arc<AtomicBool>,
    grab_enabled: bool,
}

impl Daemon {
    fn new(config: Config, config_path: PathBuf, engine_enabled: Arc<AtomicBool>) -> Self {
        let method = match config.input_method.as_str() {
            "vni" => InputMethod::Vni,
            _ => InputMethod::Telex,
        };
        let mut engine = Engine::new(method);
        engine.set_enabled(config.start_enabled);
        engine_enabled.store(config.start_enabled, Ordering::SeqCst);

        for (shortcut, expansion) in &config.macros {
            engine.add_macro(shortcut.clone(), expansion.clone());
        }

        let mut app_state = AppStateManager::new(
            config.app_state.english_apps.clone(),
            config.app_state.vietnamese_apps.clone(),
            config.start_enabled,
        );
        app_state.load_overrides();

        let config_modified = fs::metadata(&config_path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::now());

        Self {
            grab_enabled: config.grab,
            engine,
            config,
            config_path,
            config_modified,
            app_state,
            engine_enabled,
        }
    }

    fn write_status(&self) {
        if let Some(parent) = self.config_path.parent() {
            let status_path = parent.join("status");
            let enabled = self.engine.is_enabled();
            self.engine_enabled.store(enabled, Ordering::SeqCst);
            let status_str = if enabled { "vn" } else { "en" };
            let _ = std::fs::write(status_path, status_str);
        }
    }

    fn sync_status_file(&mut self) {
        if let Some(parent) = self.config_path.parent() {
            let status_path = parent.join("status");
            if let Ok(content) = fs::read_to_string(&status_path) {
                let expect_enabled = content.trim() == "vn";
                if self.engine.is_enabled() != expect_enabled {
                    eprintln!("[vietc] Syncing enabled status from file: {}", expect_enabled);
                    self.engine.set_enabled(expect_enabled);
                    self.engine_enabled.store(expect_enabled, Ordering::SeqCst);
                }
            }
        }
    }

    fn reload_config(&mut self) -> bool {
        let modified = fs::metadata(&self.config_path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::now());

        if modified <= self.config_modified {
            return false;
        }

        eprintln!("[vietc] Config changed, reloading...");
        match Config::load_from(&self.config_path) {
            Ok(new_config) => {
                let method = match new_config.input_method.as_str() {
                    "vni" => InputMethod::Vni,
                    _ => InputMethod::Telex,
                };
                self.engine.set_method(method);

                self.engine.clear_macros();
                for (shortcut, expansion) in &new_config.macros {
                    self.engine.add_macro(shortcut.clone(), expansion.clone());
                }

                self.app_state.update_lists(
                    new_config.app_state.english_apps.clone(),
                    new_config.app_state.vietnamese_apps.clone(),
                );

                self.grab_enabled = new_config.grab;
                self.config = new_config;
                self.config_modified = modified;
                eprintln!("[vietc] Config reloaded successfully");
                true
            }
            Err(e) => {
                eprintln!("[vietc] Failed to reload config: {}", e);
                false
            }
        }
    }

    fn process_key(&mut self, ch: char) -> Vec<OutputCommand> {
        let mut commands = Vec::new();

        if let Some(event) = self.engine.process_key(ch) {
            eprintln!("[vietc] key='{}' buf='{}' -> {:?}", ch, self.engine.buffer(), event);
            match event {
                EngineEvent::Flush(text) => {
                    commands.push(OutputCommand::Type(text));
                }
                EngineEvent::Insert(text) => {
                    commands.push(OutputCommand::Type(text));
                }
                EngineEvent::AutoRestore(word) => {
                    let len = word.len();
                    commands.push(OutputCommand::Backspace(len));
                    commands.push(OutputCommand::Type(word));
                }
                EngineEvent::Replace { backspaces, insert } => {
                    commands.push(OutputCommand::Backspace(backspaces));
                    commands.push(OutputCommand::Type(insert));
                }
                EngineEvent::UndoTones { backspaces, restored } => {
                    commands.push(OutputCommand::Backspace(backspaces));
                    commands.push(OutputCommand::Type(restored));
                }
            }
        } else {
            eprintln!("[vietc] key='{}' -> (no event, buf='{}')", ch, self.engine.buffer());
        }

        commands
    }

    fn toggle(&mut self) {
        let new_state = self.app_state.toggle_current_app();
        self.engine.set_enabled(new_state);
        self.write_status();
    }

    fn check_app_change_with(&mut self, new_class: String) {
        if let Some(should_enable) = self.app_state.update_with_app(new_class) {
            self.engine.set_enabled(should_enable);
            self.write_status();
        }
    }
}

#[derive(Debug)]
enum OutputCommand {
    Type(String),
    Backspace(usize),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config_path = config::find_config_path();
    let config = Config::load()?;
    let engine_enabled = Arc::new(AtomicBool::new(config.start_enabled));
    let mut daemon = Daemon::new(config, config_path.clone(), engine_enabled.clone());

    // Write initial status file
    daemon.write_status();

    let display = display::detect_display_server();
    let compositor = display::detect_compositor();

    eprintln!("Viet+ Daemon v{}", env!("CARGO_PKG_VERSION"));
    eprintln!("Display: {:?} ({})", display, compositor.unwrap_or_else(|| "unknown".into()));
    eprintln!("Input method: {:?}", daemon.config.input_method);
    eprintln!("Toggle key: Ctrl+{}", daemon.config.toggle_key.to_uppercase());
    eprintln!("App memory: {}", if daemon.config.app_state.enabled { "ON" } else { "OFF" });

    // Spawn background monitor for active window, config changes, and status changes
    let shared_active_window = Arc::new(Mutex::new(String::new()));
    let config_changed = Arc::new(AtomicBool::new(false));
    let status_changed = Arc::new(AtomicBool::new(false));

    {
        let shared_active_window = shared_active_window.clone();
        let config_changed = config_changed.clone();
        let config_path = config_path.clone();
        let status_changed = status_changed.clone();
        let engine_enabled = engine_enabled.clone();
        let mut last_modified = fs::metadata(&config_path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::now());

        thread::spawn(move || {
            let mut window_check_counter = 0;
            let status_path = config_path.parent().unwrap().join("status");
            loop {
                // Check active window class every 250ms
                if let Some(class) = app_state::get_focused_window_class() {
                    let mut lock = shared_active_window.lock().unwrap();
                    if *lock != class {
                        *lock = class;
                    }
                }

                // Check status file content changes every 250ms
                if let Ok(content) = fs::read_to_string(&status_path) {
                    let is_vn = content.trim() == "vn";
                    let current_enabled = engine_enabled.load(Ordering::SeqCst);
                    if is_vn != current_enabled {
                        status_changed.store(true, Ordering::SeqCst);
                    }
                }
                
                // Check config modified every 1.5 seconds (6 * 250ms)
                window_check_counter += 1;
                if window_check_counter >= 6 {
                    window_check_counter = 0;
                    if let Ok(metadata) = fs::metadata(&config_path) {
                        if let Ok(modified) = metadata.modified() {
                            if modified > last_modified {
                                last_modified = modified;
                                config_changed.store(true, Ordering::SeqCst);
                            }
                        }
                    }
                }

                thread::sleep(Duration::from_millis(250));
            }
        });
    }

    match open_keyboard_device() {
        Ok((device, path)) => {
            eprintln!("[vietc] Keyboard device: {}", path);
            run_with_evdev(
                device,
                &mut daemon,
                shared_active_window,
                config_changed,
                status_changed,
                engine_enabled,
                display,
            )?;
        }
        Err(e) => {
            eprintln!("[vietc] No keyboard device: {}", e);
            eprintln!("[vietc] Running in stdin test mode");
            run_stdin_mode(
                &mut daemon,
                shared_active_window,
                config_changed,
                status_changed,
                engine_enabled,
                display,
            )?;
        }
    }

    Ok(())
}

fn open_keyboard_device() -> Result<(evdev::Device, String), Box<dyn std::error::Error>> {
    let dir = std::path::Path::new("/dev/input");
    if !dir.exists() {
        return Err("No /dev/input directory".into());
    }

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
                    // Skip our own uinput device, lid switches, power buttons, etc.
                    if dev_name.eq_ignore_ascii_case("vietc") {
                        continue;
                    }
                    if device.supported_keys().is_some_and(|k| {
                        k.contains(evdev::Key::KEY_A)
                    }) {
                        return Ok((device, format!("{} ({})", entry.path().display(), dev_name)));
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

    if permission_denied_count > 0 {
        // Check if user is in the group but session hasn't refreshed
        let in_group_db = std::process::Command::new("groups")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("input"))
            .unwrap_or(false);

        if in_group_db {
            Err(format!(
                "Permission denied on {}/{} devices. Your user IS in the 'input' group, \
                 but your current session hasn't picked it up yet. \
                 Please LOG OUT and LOG BACK IN to activate group permissions.",
                permission_denied_count, total_event_count
            ).into())
        } else {
            Err(format!(
                "Permission denied on {}/{} devices. Add your user to the 'input' group: \
                 sudo usermod -aG input $USER && sudo usermod -aG vinput $USER, \
                 then log out and log back in.",
                permission_denied_count, total_event_count
            ).into())
        }
    } else {
        Err("No keyboard device found".into())
    }
}

fn run_with_evdev(
    mut device: evdev::Device,
    daemon: &mut Daemon,
    shared_active_window: Arc<Mutex<String>>,
    config_changed: Arc<AtomicBool>,
    status_changed: Arc<AtomicBool>,
    _engine_enabled: Arc<AtomicBool>,
    display: display::DisplayServer,
) -> Result<(), Box<dyn std::error::Error>> {
    let injector = create_injector(display)?;

    let grabbed = if daemon.grab_enabled {
        match device.grab() {
            Ok(()) => {
                eprintln!("[vietc] Keyboard grabbed — race condition eliminated");
                true
            }
            Err(e) => {
                eprintln!("[vietc] Could not grab keyboard: {} (run as root for grab)", e);
                eprintln!("[vietc] Falling back to non-grabbing mode (may have race)");
                false
            }
        }
    } else {
        eprintln!("[vietc] Keyboard grab disabled (config grab = false)");
        eprintln!("[vietc] Set grab = true in vietc.toml to enable (needs root)");
        false
    };

    let mut consumed_keys: HashSet<u16> = HashSet::new();

    // Safety: if grab is active and no events arrive for 30 seconds,
    // release the grab so the user isn't locked out.
    let mut last_event_time = std::time::Instant::now();

    loop {
        // Check for event timeout (grab safety)
        if grabbed && last_event_time.elapsed() > std::time::Duration::from_secs(30) {
            eprintln!("[vietc] No events for 30s — releasing grab timeout, releasing grab for safety");
            let _ = device.ungrab();
            return Ok(());
        }

        let key_state = device.get_key_state().ok();
        let events = device.fetch_events()?;
        last_event_time = std::time::Instant::now();

        // Check for status changes instantly
        if status_changed.load(Ordering::SeqCst) {
            daemon.sync_status_file();
            status_changed.store(false, Ordering::SeqCst);
        }

        // Check for app changes instantly using the cached state from background thread
        if daemon.config.app_state.enabled {
            let active_window = shared_active_window.lock().unwrap().clone();
            daemon.check_app_change_with(active_window);
        }

        // Check for config reload instantly
        if config_changed.load(Ordering::SeqCst) {
            daemon.reload_config();
            config_changed.store(false, Ordering::SeqCst);
        }

        for event in events {
            if let evdev::InputEventKind::Key(key) = event.kind() {
                let value = event.value();

                if value == 1
                    && is_toggle_combination_state(&key_state, &daemon.config.toggle_key)
                {
                    daemon.toggle();
                    continue;
                }

                if !grabbed {
                    // Legacy mode: only forward to engine on press events
                    if value != 1 {
                        continue;
                    }
                    if let Some(ch) = key_to_char(key) {
                        let commands = daemon.process_key(ch);
                        execute_commands(&*injector, &commands, false);
                    }
                } else {
                    // Grabbing mode: all output goes through uinput only.
                    // Physical evdev is grabbed — never forward through it,
                    // as separate channels have no ordering guarantee.
                    let keycode = key.0;

                    if value == 1 {
                        // Press: process through engine
                        if consumed_keys.contains(&keycode) {
                            consumed_keys.remove(&keycode);
                        }
                        if let Some(ch) = key_to_char(key) {
                            let commands = daemon.process_key(ch);
                            if !commands.is_empty() {
                                consumed_keys.insert(keycode);
                                execute_commands(&*injector, &commands, true);
                            } else {
                                injector.send_char(ch);
                            }
                        } else {
                            injector.send_key_event(keycode, 1);
                        }
                    } else if value == 2 {
                        // Auto-repeat: skip if consumed, else forward
                        if consumed_keys.contains(&keycode) {
                            continue;
                        }
                        if let Some(ch) = key_to_char(key) {
                            injector.send_char(ch);
                        } else {
                            injector.send_key_event(keycode, 1);
                            injector.send_key_event(keycode, 0);
                        }
                    } else if value == 0 {
                        // Release: skip if consumed, else forward
                        if consumed_keys.contains(&keycode) {
                            consumed_keys.remove(&keycode);
                            continue;
                        }
                        injector.send_key_event(keycode, 0);
                    }
                }
            }
        }
    }
}

fn run_stdin_mode(
    daemon: &mut Daemon,
    shared_active_window: Arc<Mutex<String>>,
    config_changed: Arc<AtomicBool>,
    status_changed: Arc<AtomicBool>,
    _engine_enabled: Arc<AtomicBool>,
    display: display::DisplayServer,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, Read, IsTerminal};


    if !io::stdin().is_terminal() {
        eprintln!("[vietc] Warning: No keyboard device and no terminal.");
        eprintln!("[vietc] Retrying keyboard access every 5 seconds...");
        eprintln!("[vietc] Ensure you are in the 'input' group:");
        eprintln!("      sudo usermod -aG input $USER");
        eprintln!("  Then log out and back in.");

        // Retry loop: periodically attempt to reopen the keyboard device
        loop {
            thread::sleep(Duration::from_secs(5));

            // Check for status changes
            if status_changed.load(Ordering::SeqCst) {
                daemon.sync_status_file();
                status_changed.store(false, Ordering::SeqCst);
            }
            if config_changed.load(Ordering::SeqCst) {
                daemon.reload_config();
                config_changed.store(false, Ordering::SeqCst);
            }

            if let Ok((device, path)) = open_keyboard_device() {
                eprintln!("[vietc] Keyboard device found: {}", path);
                return run_with_evdev(
                    device, daemon,
                    shared_active_window,
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

    eprintln!("[vietc] Type to test, Ctrl+C to exit");

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    loop {
        // Check for status changes instantly
        if status_changed.load(Ordering::SeqCst) {
            daemon.sync_status_file();
            status_changed.store(false, Ordering::SeqCst);
        }

        // Check for app changes instantly using the cached state from background thread
        if daemon.config.app_state.enabled {
            let active_window = shared_active_window.lock().unwrap().clone();
            daemon.check_app_change_with(active_window);
        }

        // Check for config reload instantly
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
                eprintln!("[vietc] Read error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// Execute commands — accumulate backspaces and text, then inject through
/// a single channel (ydotool or wtype) to avoid reordering between backspaces
/// (uinput) and text (ydotool).
fn execute_commands(injector: &dyn vietc_protocol::KeyInjector, commands: &[OutputCommand], grabbed: bool) {
    let mut pending_backspaces: usize = 0;
    let mut pending_text = String::new();

    for cmd in commands {
        match cmd {
            OutputCommand::Backspace(count) => {
                let adjusted = if grabbed { count.saturating_sub(1) } else { *count };
                eprintln!("[vietc] cmd: Backspace({}) -> adjusted={}", count, adjusted);
                pending_backspaces += adjusted;
            }
            OutputCommand::Type(text) => {
                eprintln!("[vietc] cmd: Type(\"{}\")", text);
                pending_text.push_str(text);
            }
        }
    }

    if pending_backspaces > 0 || !pending_text.is_empty() {
        eprintln!("[vietc] inject: BS={} text=\"{}\"", pending_backspaces, pending_text);
        injector.inject_replacement(pending_backspaces, &pending_text);
    }
    injector.flush();
}

fn create_injector(display: display::DisplayServer) -> Result<Box<dyn vietc_protocol::KeyInjector>, Box<dyn std::error::Error>> {
    // Try Wayland input method first (if compiled with wayland feature)
    #[cfg(feature = "wayland")]
    {
        let _ctx = vietc_protocol::wayland_im::WaylandIMContext::new();
        eprintln!("[vietc] Wayland input method context initialized");
    }

    // Use uinput as primary injector — it handles ASCII via direct keycodes
    // and Unicode via ydotool type (uinput-based, no display server needed).
    // Using a single injection channel avoids ordering issues between XTest
    // (ASCII) and ydotool (Unicode) interleaving.
    match vietc_protocol::uinput_monitor::UinputInjector::new("vietc") {
        Ok(injector) => {
            eprintln!("[vietc] Using uinput injection (primary)");
            return Ok(Box::new(injector));
        }
        Err(e) => {
            eprintln!("[vietc] uinput not available: {}", e);
        }
    }

    // Fall back to X11 XTEST (last resort — doesn't handle Unicode well)
    #[cfg(feature = "x11")]
    {
        if display != display::DisplayServer::Wayland {
            match vietc_protocol::x11_inject::X11Injector::new() {
                Ok(injector) => {
                    eprintln!("[vietc] Using X11 injection (XTEST fallback)");
                    return Ok(Box::new(injector));
                }
                Err(e) => {
                    eprintln!("[vietc] X11 not available: {}", e);
                }
            }
        }
    }

    Err("No injection backend available".into())
}

fn is_toggle_combination_state(key_state: &Option<evdev::AttributeSet<evdev::Key>>, key: &str) -> bool {
    let key_state = match key_state {
        Some(ks) => ks,
        None => return false,
    };

    let ctrl_pressed = key_state.contains(evdev::Key::KEY_LEFTCTRL)
        || key_state.contains(evdev::Key::KEY_RIGHTCTRL);

    if !ctrl_pressed {
        return false;
    }

    let target = match key.to_lowercase().as_str() {
        "space" => evdev::Key::KEY_SPACE,
        "shift" => evdev::Key::KEY_LEFTSHIFT,
        "capslock" => evdev::Key::KEY_CAPSLOCK,
        "ctrl" => evdev::Key::KEY_LEFTCTRL,
        "alt" => evdev::Key::KEY_LEFTALT,
        _ => return false,
    };

    key_state.contains(target)
}

fn key_to_char(key: evdev::Key) -> Option<char> {
    match key {
        evdev::Key::KEY_A => Some('a'),
        evdev::Key::KEY_B => Some('b'),
        evdev::Key::KEY_C => Some('c'),
        evdev::Key::KEY_D => Some('d'),
        evdev::Key::KEY_E => Some('e'),
        evdev::Key::KEY_F => Some('f'),
        evdev::Key::KEY_G => Some('g'),
        evdev::Key::KEY_H => Some('h'),
        evdev::Key::KEY_I => Some('i'),
        evdev::Key::KEY_J => Some('j'),
        evdev::Key::KEY_K => Some('k'),
        evdev::Key::KEY_L => Some('l'),
        evdev::Key::KEY_M => Some('m'),
        evdev::Key::KEY_N => Some('n'),
        evdev::Key::KEY_O => Some('o'),
        evdev::Key::KEY_P => Some('p'),
        evdev::Key::KEY_Q => Some('q'),
        evdev::Key::KEY_R => Some('r'),
        evdev::Key::KEY_S => Some('s'),
        evdev::Key::KEY_T => Some('t'),
        evdev::Key::KEY_U => Some('u'),
        evdev::Key::KEY_V => Some('v'),
        evdev::Key::KEY_W => Some('w'),
        evdev::Key::KEY_X => Some('x'),
        evdev::Key::KEY_Y => Some('y'),
        evdev::Key::KEY_Z => Some('z'),
        evdev::Key::KEY_0 => Some('0'),
        evdev::Key::KEY_1 => Some('1'),
        evdev::Key::KEY_2 => Some('2'),
        evdev::Key::KEY_3 => Some('3'),
        evdev::Key::KEY_4 => Some('4'),
        evdev::Key::KEY_5 => Some('5'),
        evdev::Key::KEY_6 => Some('6'),
        evdev::Key::KEY_7 => Some('7'),
        evdev::Key::KEY_8 => Some('8'),
        evdev::Key::KEY_9 => Some('9'),
        evdev::Key::KEY_SPACE => Some(' '),
        evdev::Key::KEY_DOT => Some('.'),
        evdev::Key::KEY_COMMA => Some(','),
        evdev::Key::KEY_MINUS => Some('-'),
        evdev::Key::KEY_EQUAL => Some('='),
        evdev::Key::KEY_SEMICOLON => Some(';'),
        evdev::Key::KEY_APOSTROPHE => Some('\''),
        evdev::Key::KEY_SLASH => Some('/'),
        evdev::Key::KEY_BACKSPACE => Some('\x08'),
        evdev::Key::KEY_ENTER => Some('\n'),
        _ => None,
    }
}
