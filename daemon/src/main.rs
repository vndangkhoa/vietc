use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use vietc_engine::{Engine, EngineEvent, InputMethod};

/// Pin current thread to performance cores (0-3) and boost priority.
/// Inspired by VMK's approach to minimize input latency on Intel hybrid CPUs.
fn boost_thread_priority() {
    unsafe {
        // Set nice value to -10 (higher priority than normal)
        libc::setpriority(libc::PRIO_PROCESS, 0, -10);

        // Try to pin to P-cores (cores 0-3 on Intel hybrid)
        #[cfg(target_os = "linux")]
        {
            let mut cpuset: libc::cpu_set_t = std::mem::zeroed();
            // Pin to cores 0-3 (P-cores on Intel 12th gen+)
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

mod app_state;
mod config;
mod display;

use app_state::AppStateManager;
use config::Config;

#[cfg(feature = "x11")]
use vietc_protocol::x11_capture::X11Capture;
use vietc_protocol::x11_capture::SKIP_RECORD_EVENTS;
#[cfg(feature = "x11")]
use vietc_protocol::x11_inject::X11Injector;

fn get_log_path() -> Option<PathBuf> {
    dirs::config_dir().map(|p| p.join("vietc").join("vietc.log"))
}

fn get_timestamp() -> String {
    if let Ok(n) = std::time::SystemTime::now().duration_since(std::time::SystemTime::UNIX_EPOCH) {
        let secs = n.as_secs();
        let millis = n.subsec_millis();
        unsafe {
            let t = secs as libc::time_t;
            let mut tm = std::mem::zeroed::<libc::tm>();
            if !libc::localtime_r(&t, &mut tm).is_null() {
                return format!(
                    "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}",
                    tm.tm_year + 1900,
                    tm.tm_mon + 1,
                    tm.tm_mday,
                    tm.tm_hour,
                    tm.tm_min,
                    tm.tm_sec,
                    millis
                );
            }
        }
    }
    "".to_string()
}

fn log_info(msg: &str) {
    eprintln!("{}", msg);

    if let Some(log_path) = get_log_path() {
        if let Some(parent) = log_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        // Rotate log if it exceeds 10MB
        if let Ok(metadata) = fs::metadata(&log_path) {
            if metadata.len() > 10 * 1024 * 1024 {
                let backup_path = log_path.with_extension("log.old");
                let _ = fs::rename(&log_path, backup_path);
            }
        }

        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
        {
            use std::io::Write;
            let timestamp = get_timestamp();
            let _ = writeln!(file, "[{}] {}", timestamp, msg);
        }
    }
}

struct Daemon {
    engine: Engine,
    config: Config,
    config_path: PathBuf,
    config_modified: std::time::SystemTime,
    app_state: AppStateManager,
    engine_enabled: Arc<AtomicBool>,
    grab_enabled: bool,
    /// Backspace-Replay: all keystrokes in the current word being composed.
    /// On each keypress, we replay the entire history through a fresh engine
    /// to compute the correct screen output, eliminating state desync.
    keystroke_history: Vec<char>,
    /// What's currently displayed on screen for the current word.
    /// Used to calculate how many backspaces we need before retyping.
    screen_output: String,
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
            config.app_state.bypass_apps.clone(),
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
            keystroke_history: Vec::new(),
            screen_output: String::new(),
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
                    new_config.app_state.bypass_apps.clone(),
                );

                self.grab_enabled = new_config.grab;
                self.config = new_config;
                self.config_modified = modified;
                true
            }
            Err(_) => false,
        }
    }

    fn process_key(&mut self, ch: char) -> Vec<OutputCommand> {
        let mut commands = Vec::new();

        if let Some(event) = self.engine.process_key(ch) {
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
                EngineEvent::UndoTones {
                    backspaces,
                    restored,
                } => {
                    commands.push(OutputCommand::Backspace(backspaces));
                    commands.push(OutputCommand::Type(restored));
                }
                EngineEvent::Paste(text) => {
                    self.engine.exit_paste_mode();
                    commands.push(OutputCommand::Type(text));
                }
            }
        } else {
            // No event — key was consumed or ignored by engine
        }

        commands
    }

    fn toggle(&mut self) {
        let new_state = self.app_state.toggle_current_app();

        self.engine.set_enabled(new_state);
        self.write_status();

        // Reset engine buffer when enabling Vietnamese mode to clear stale state
        if new_state {
            self.engine.reset();
        }
    }

    fn is_current_app_bypassed(&self) -> bool {
        if !self.config.app_state.enabled {
            return false;
        }
        self.app_state.is_current_app_bypassed()
    }

    /// Backspace-Replay: replay the entire keystroke history through a fresh
    /// engine, compute what should be on screen, and return the commands
    /// (backspaces to erase old + new text to type).
    fn replay_and_inject(&mut self, ch: char) -> Vec<OutputCommand> {
        let mut commands = Vec::new();

        // Flush characters: commit current word, type the character, clear state
        if is_flush_char(ch) {
            if !self.screen_output.is_empty() {
                let backspaces = self.screen_output.chars().count();
                commands.push(OutputCommand::Backspace(backspaces));
                commands.push(OutputCommand::Type(self.screen_output.clone()));
            }
            // Type the flush character itself
            commands.push(OutputCommand::Type(ch.to_string()));
            self.keystroke_history.clear();
            self.screen_output.clear();
            return commands;
        }

        // Add the new keystroke to history
        self.keystroke_history.push(ch);

        // Replay through fresh engine
        let method = match self.config.input_method.as_str() {
            "vni" => InputMethod::Vni,
            _ => InputMethod::Telex,
        };
        let (new_output, did_flush) = Engine::replay_keystrokes(
            method,
            &self.config.macros,
            &self.keystroke_history,
        );

        if did_flush {
            // Engine flushed a word — commit it and clear state
            // The flush char (space/period/etc) was NOT in history, so we need to
            // type whatever was on screen + the flush char
            if !self.screen_output.is_empty() {
                let backspaces = self.screen_output.chars().count();
                commands.push(OutputCommand::Backspace(backspaces));
                commands.push(OutputCommand::Type(self.screen_output.clone()));
            }
            self.keystroke_history.clear();
            self.screen_output.clear();
            return commands;
        }

        if new_output != self.screen_output {
            let backspaces = self.screen_output.chars().count();
            if backspaces > 0 {
                commands.push(OutputCommand::Backspace(backspaces));
            }
            if !new_output.is_empty() {
                commands.push(OutputCommand::Type(new_output.clone()));
            }
            self.screen_output = new_output;
        }

        commands
    }

    /// Backspace-Replay: pop from history, replay, and return commands to fix screen.
    fn replay_backspace(&mut self) -> Vec<OutputCommand> {
        let mut commands = Vec::new();

        if self.keystroke_history.is_empty() {
            // Nothing in history — just forward the backspace
            commands.push(OutputCommand::Backspace(1));
            return commands;
        }

        // Remove last keystroke from history
        self.keystroke_history.pop();

        // Replay through fresh engine
        let method = match self.config.input_method.as_str() {
            "vni" => InputMethod::Vni,
            _ => InputMethod::Telex,
        };
        let (new_output, _) = if self.keystroke_history.is_empty() {
            (String::new(), false)
        } else {
            Engine::replay_keystrokes(
                method,
                &self.config.macros,
                &self.keystroke_history,
            )
        };

        // Calculate diff
        let backspaces = self.screen_output.chars().count();
        if backspaces > 0 {
            commands.push(OutputCommand::Backspace(backspaces));
        }
        if !new_output.is_empty() {
            commands.push(OutputCommand::Type(new_output.clone()));
        }
        self.screen_output = new_output;

        commands
    }

    /// Reset the replay state (on flush, focus loss, modifier key, etc.)
    fn replay_reset(&mut self) {
        self.keystroke_history.clear();
        self.screen_output.clear();
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

/// Characters that flush the current word and start a new one.
fn is_flush_char(ch: char) -> bool {
    matches!(ch, ' ' | '.' | ',' | '!' | '?' | ';' | ':' | '\t' | '\n')
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
        if daemon.config.app_state.enabled {
            "ON"
        } else {
            "OFF"
        }
    ));

    // Boost thread priority for low-latency input (VMK technique)
    boost_thread_priority();

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

    // Try evdev first (more reliable than X11 XRecord)
    match open_keyboard_device() {
        Ok((device, path)) => {
            log_info(&format!("[vietc] Keyboard device: {}", path));
            return run_with_evdev(
                device,
                &mut daemon,
                shared_active_window,
                config_changed,
                status_changed,
                engine_enabled,
                display,
            );
        }
        Err(e) => {
            log_info(&format!("[vietc] evdev not available: {}", e));
        }
    }

    #[cfg(feature = "x11")]
    if display != display::DisplayServer::Wayland {
        if let Some(capture) = X11Capture::new() {
            log_info("[vietc] X11 XRecord capture active — using X11 capture/injection");
            return run_with_x11(
                capture,
                &mut daemon,
                shared_active_window.clone(),
                config_changed.clone(),
                status_changed.clone(),
                engine_enabled.clone(),
            );
        } else {
            log_info("[vietc] X11 not available, falling back");
        }
    }

    log_info("[vietc] Running in stdin test mode");
    run_stdin_mode(
        &mut daemon,
        shared_active_window,
        config_changed,
        status_changed,
        engine_enabled,
        display,
    )?;

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
                    if device
                        .supported_keys()
                        .is_some_and(|k| k.contains(evdev::Key::KEY_A))
                    {
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
            )
            .into())
        } else {
            Err(format!(
                "Permission denied on {}/{} devices. Add your user to the 'input' group: \
                 sudo usermod -aG input $USER && sudo usermod -aG vinput $USER, \
                 then log out and log back in.",
                permission_denied_count, total_event_count
            )
            .into())
        }
    } else {
        Err("No keyboard device found".into())
    }
}

#[cfg(feature = "x11")]
fn run_with_x11(
    mut capture: X11Capture,
    daemon: &mut Daemon,
    shared_active_window: Arc<Mutex<String>>,
    config_changed: Arc<AtomicBool>,
    status_changed: Arc<AtomicBool>,
    _engine_enabled: Arc<AtomicBool>,
) -> Result<(), Box<dyn std::error::Error>> {
    let injector: Box<dyn vietc_protocol::KeyInjector> = Box::new(X11Injector::new()?);
    let mut last_active_window = String::new();
    // Track physically-held keys so we only inject press on KeyPress
    // and release on KeyRelease — without this, every KeyPress injects
    // press+release immediately, breaking held-key combos (Ctrl+C, Alt+Tab…).
    let mut pressed_keys: HashSet<u32> = HashSet::new();

    eprintln!("[vietc] X11 event loop starting");

    loop {
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
                daemon.replay_reset();
            }
        }

        if daemon.config.app_state.enabled {
            let active_window = shared_active_window.lock().unwrap().clone();
            daemon.check_app_change_with(active_window);
        }

        // Reset on focus loss (VMK technique)
        if capture.focus_lost {
            daemon.replay_reset();
            pressed_keys.clear();
            capture.focus_lost = false;
        }

        // Wait for events with 100ms timeout.
        // SKIP_RECORD_EVENTS may still be true from a previous injection —
        // drain_pipe drops any stale injected events while flag is true.
        let _got_data = capture.wait_for_event(100);
        // NOW safe to clear: any injected events from last iteration were dropped.
        SKIP_RECORD_EVENTS.store(false, Ordering::Relaxed);
        let evt = capture.next_event();
        if evt.is_none() {
            continue;
        }
        let event = evt.unwrap();

        // Process this event
        {
            if event.pressed {
                // Skip autorepeat
                if !pressed_keys.insert(event.keycode) {
                    continue;
                }

                // Toggle key: Ctrl+Space
                if let Some(' ') = event.ch {
                    if (event.state & 4) != 0 {
                        pressed_keys.remove(&event.keycode);
                        daemon.replay_reset();
                        daemon.toggle();
                        continue;
                    }
                }

                // Modifier or non-character key → forward press only, reset replay
                if capture.is_modifier_pressed(event.state) || event.ch.is_none() {
                    daemon.replay_reset();
                    SKIP_RECORD_EVENTS.store(true, Ordering::Relaxed);
                    let _ = injector.send_key_event(event.keycode as u16, 1);
                    // Flag stays true — cleared at top of next iteration after drain
                    continue;
                }

                // Character key — use Backspace-Replay
                if let Some(ch) = event.ch {
                    match ch {
                        '\x08' => {
                            let commands = daemon.replay_backspace();
                            pressed_keys.remove(&event.keycode);
                            SKIP_RECORD_EVENTS.store(true, Ordering::Relaxed);
                            execute_commands(&*injector, &commands, true);
                            if daemon.keystroke_history.is_empty() && commands.is_empty() {
                                let _ = injector.send_backspace();
                            }
                        }
                        '\n' => {
                            pressed_keys.remove(&event.keycode);
                            daemon.replay_reset();
                            SKIP_RECORD_EVENTS.store(true, Ordering::Relaxed);
                            let _ = injector.send_key_event(event.keycode as u16, 1);
                            let _ = injector.send_key_event(event.keycode as u16, 0);
                        }
                        _ => {
                            let commands = daemon.replay_and_inject(ch);
                            pressed_keys.remove(&event.keycode);
                            SKIP_RECORD_EVENTS.store(true, Ordering::Relaxed);
                            execute_commands(&*injector, &commands, true);
                        }
                    }
                }
            } else {
                // Key release — only inject if we were tracking this key
                if pressed_keys.remove(&event.keycode) {
                    SKIP_RECORD_EVENTS.store(true, Ordering::Relaxed);
                    let _ = injector.send_key_event(event.keycode as u16, 0);
                }
            }
        }
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
                log_info("[vietc] Keyboard grabbed — race condition eliminated");
                true
            }
            Err(e) => {
                log_info(&format!(
                    "[vietc] Could not grab keyboard: {} (run as root for grab)",
                    e
                ));
                log_info("[vietc] Falling back to non-grabbing mode (may have race)");
                false
            }
        }
    } else {
        log_info("[vietc] Keyboard grab disabled (config grab = false)");
        log_info("[vietc] Set grab = true in vietc.toml to enable (needs root)");
        false
    };

    let mut consumed_keys: HashSet<u16> = HashSet::new();
    let mut last_active_window = String::new();
    // Skip counter: after Unicode injection, skip N upcoming events
    // (they're auto-repeat pile-up from the injection delay)
    let mut skip_count = 0u32;

    // Safety: if grab is active and no events arrive for 30 seconds,
    // release the grab so the user isn't locked out.
    let mut last_event_time = std::time::Instant::now();

    loop {
        // Check for event timeout (grab safety)
        if grabbed && last_event_time.elapsed() > std::time::Duration::from_secs(30) {
            log_info(
                "[vietc] No events for 30s — releasing grab timeout, releasing grab for safety",
            );
            let _ = device.ungrab();
            return Ok(());
        }

        let caps = is_caps_lock_on(&device);
        let mut key_state = device
            .get_key_state()
            .ok()
            .unwrap_or_else(evdev::AttributeSet::new);
        let events = device.fetch_events()?;
        last_event_time = std::time::Instant::now();

        // Check for status changes instantly
        if status_changed.load(Ordering::SeqCst) {
            daemon.sync_status_file();
            status_changed.store(false, Ordering::SeqCst);
        }

        // Track window changes and reset engine buffer
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
                let keycode = key.0;

                // Update key state dynamically
                if value == 1 {
                    key_state.insert(key);
                } else if value == 0 {
                    key_state.remove(key);
                }

                // Completely bypass all IME processing/interception for terminal emulators, IDE terminals, and games
                if daemon.is_current_app_bypassed() {
                    if grabbed {
                        injector.send_key_event(keycode, value);
                    }
                    continue;
                }

                if value == 1 && is_toggle_combination_state(&key_state, &daemon.config.toggle_key)
                {
                    daemon.toggle();
                    continue;
                }

                if !grabbed {
                    // Legacy mode: only forward to engine on press events
                    if value != 1 {
                        continue;
                    }
                    if is_modifier_pressed(&key_state) {
                        continue;
                    }
                    if let Some(ch) = key_to_char(key) {
                        let commands = daemon.process_key(ch);
                        execute_commands(&*injector, &commands, false);
                    }
                } else {
                    // Grabbing mode: all output goes through uinput only.

                    // If Ctrl, Alt, or Meta/Super is pressed, bypass the engine completely and forward raw key events.
                    if is_modifier_pressed(&key_state) {
                        injector.send_key_event(keycode, value);
                        continue;
                    }

                    // Backspace in grab mode: pop engine, inject via uinput.
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
                        // Press: process through engine
                        if consumed_keys.contains(&keycode) {
                            consumed_keys.remove(&keycode);
                        }
                        if let Some(mut ch) = key_to_char(key) {
                            let shift = is_modifier_held_shift(&key_state);
                            if ch.is_ascii_alphabetic() && (shift ^ caps) {
                                ch = ch.to_ascii_uppercase();
                            }
                            let commands = daemon.process_key(ch);
                            if !commands.is_empty() {
                                consumed_keys.insert(keycode);
                                execute_commands(&*injector, &commands, false);
                                // Skip upcoming auto-repeat pile-up from injection delay
                                skip_count = 3;
                            } else if is_vn_control_key(&daemon.config.input_method, ch) {
                                // Tone/mark key with no effect — consume silently
                                consumed_keys.insert(keycode);
                            } else {
                                injector.send_key_event(keycode, 1);
                            }
                        } else {
                            injector.send_key_event(keycode, 1);
                        }
                    } else if value == 2 {
                        // Auto-repeat: skip if consumed or during injection drain
                        if consumed_keys.contains(&keycode) || skip_count > 0 {
                            if skip_count > 0 { skip_count -= 1; }
                            continue;
                        }
                        injector.send_key_event(keycode, 2);
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
    use std::io::{self, IsTerminal, Read};

    if !io::stdin().is_terminal() {
        log_info("[vietc] Warning: No keyboard device and no terminal.");
        log_info("[vietc] Retrying keyboard access every 5 seconds...");
        log_info("[vietc] Ensure you are in the 'input' group:");
        log_info("      sudo usermod -aG input $USER");
        log_info("  Then log out and back in.");

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
                log_info(&format!("[vietc] Keyboard device found: {}", path));
                return run_with_evdev(
                    device,
                    daemon,
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
    let mut last_active_window = String::new();

    log_info("[vietc] Type to test, Ctrl+C to exit");

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    loop {
        // Check for status changes instantly
        if status_changed.load(Ordering::SeqCst) {
            daemon.sync_status_file();
            status_changed.store(false, Ordering::SeqCst);
        }

        // Track window changes and reset engine buffer
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
                log_info(&format!("[vietc] Read error: {}", e));
                break;
            }
        }
    }

    Ok(())
}

/// Execute commands — accumulate backspaces and text, then inject through
/// a single channel (ydotool or wtype) to avoid reordering between backspaces
/// (uinput) and text (ydotool).
fn execute_commands(
    injector: &dyn vietc_protocol::KeyInjector,
    commands: &[OutputCommand],
    grabbed: bool,
) {
    let mut pending_backspaces: usize = 0;
    let mut pending_text = String::new();

    for cmd in commands {
        match cmd {
            OutputCommand::Backspace(count) => {
                let adjusted = if grabbed {
                    count.saturating_sub(1)
                } else {
                    *count
                };
                pending_backspaces += adjusted;
            }
            OutputCommand::Type(text) => {
                pending_text.push_str(text);
            }
        }
    }

    if pending_backspaces > 0 || !pending_text.is_empty() {
        let _ = injector.inject_replacement(pending_backspaces, &pending_text);
    } else if !commands.is_empty() {
        let _ = injector.inject_replacement(pending_backspaces, &pending_text);
    }

    injector.flush();

    // Sleep briefly to let the display server and target application process the
    // injected key strokes and clear any modifier states before we handle subsequent physical keys.
    if grabbed && !commands.is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

fn create_injector(
    display: display::DisplayServer,
) -> Result<Box<dyn vietc_protocol::KeyInjector>, Box<dyn std::error::Error>> {
    // Try uinputd socket first
    if vietc_protocol::uinput_client::UinputClient::is_available() {
        log_info("[vietc] Using uinputd socket injection");
        return Ok(Box::new(vietc_protocol::uinput_client::UinputClient));
    }

    // Use uinput as primary — correct Linux keycodes for ASCII + backspace.
    // For Unicode (Vietnamese diacritics), falls back to xclip via subprocess
    // or direct X11 clipboard via X11Injector.
    match vietc_protocol::uinput_monitor::UinputInjector::new("vietc") {
        Ok(injector) => {
            log_info("[vietc] Using uinput injection (primary)");
            return Ok(Box::new(injector));
        }
        Err(e) => {
            log_info(&format!("[vietc] uinput not available: {}", e));
        }
    }

    // Fall back to X11 injection (only if uinput fails)
    #[cfg(feature = "x11")]
    {
        if display != display::DisplayServer::Wayland {
            match vietc_protocol::x11_inject::X11Injector::new() {
                Ok(injector) => {
                    log_info("[vietc] Using X11 injection (fallback)");
                    return Ok(Box::new(injector));
                }
                Err(e) => {
                    log_info(&format!("[vietc] X11 not available: {}", e));
                }
            }
        }
    }

    Err("No injection backend available".into())
}

fn is_vn_control_key(method: &str, ch: char) -> bool {
    match method {
        "telex" => matches!(ch.to_ascii_lowercase(), 'f' | 's' | 'r' | 'x' | 'j' | 'w'),
        "vni" => matches!(ch, '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '0'),
        _ => false,
    }
}

fn is_modifier_pressed(key_state: &evdev::AttributeSet<evdev::Key>) -> bool {
    key_state.contains(evdev::Key::KEY_LEFTCTRL)
        || key_state.contains(evdev::Key::KEY_RIGHTCTRL)
        || key_state.contains(evdev::Key::KEY_LEFTALT)
        || key_state.contains(evdev::Key::KEY_RIGHTALT)
        || key_state.contains(evdev::Key::KEY_LEFTMETA)
        || key_state.contains(evdev::Key::KEY_RIGHTMETA)
}

fn is_modifier_held_shift(key_state: &evdev::AttributeSet<evdev::Key>) -> bool {
    key_state.contains(evdev::Key::KEY_LEFTSHIFT) || key_state.contains(evdev::Key::KEY_RIGHTSHIFT)
}

fn is_caps_lock_on(device: &evdev::Device) -> bool {
    if let Ok(leds) = device.get_led_state() {
        leds.contains(evdev::LedType::LED_CAPSL)
    } else {
        false
    }
}

fn is_toggle_combination_state(key_state: &evdev::AttributeSet<evdev::Key>, key: &str) -> bool {
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
