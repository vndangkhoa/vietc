use std::fs;
use std::path::PathBuf;

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
}

impl Daemon {
    fn new(config: Config, config_path: PathBuf) -> Self {
        let method = match config.input_method.as_str() {
            "vni" => InputMethod::Vni,
            _ => InputMethod::Telex,
        };
        let mut engine = Engine::new(method);
        engine.set_enabled(config.start_enabled);

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
            engine,
            config,
            config_path,
            config_modified,
            app_state,
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
        }

        commands
    }

    fn toggle(&mut self) {
        let new_state = self.app_state.toggle_current_app();
        self.engine.set_enabled(new_state);
    }

    fn check_app_change(&mut self) {
        if let Some(should_enable) = self.app_state.update() {
            self.engine.set_enabled(should_enable);
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
    let mut daemon = Daemon::new(config, config_path);

    let display = display::detect_display_server();
    let compositor = display::detect_compositor();

    eprintln!("Viet+ Daemon v{}", env!("CARGO_PKG_VERSION"));
    eprintln!("Display: {:?} ({})", display, compositor.unwrap_or_else(|| "unknown".into()));
    eprintln!("Input method: {:?}", daemon.config.input_method);
    eprintln!("Toggle key: Ctrl+{}", daemon.config.toggle_key.to_uppercase());
    eprintln!("App memory: {}", if daemon.config.app_state.enabled { "ON" } else { "OFF" });

    match open_keyboard_device() {
        Ok((device, path)) => {
            eprintln!("[vietc] Keyboard device: {}", path);
            run_with_evdev(device, &mut daemon)?;
        }
        Err(e) => {
            eprintln!("[vietc] No keyboard device: {}", e);
            eprintln!("[vietc] Running in stdin test mode");
            run_stdin_mode(&mut daemon)?;
        }
    }

    Ok(())
}

fn open_keyboard_device() -> Result<(evdev::Device, String), Box<dyn std::error::Error>> {
    let dir = std::path::Path::new("/dev/input");
    if !dir.exists() {
        return Err("No /dev/input directory".into());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();

        if name_str.starts_with("event") {
            match evdev::Device::open(entry.path()) {
                Ok(device) => {
                    let dev_name = device.name().unwrap_or("unknown").to_string();
                    if device.supported_keys().is_some_and(|k| {
                        k.contains(evdev::Key::KEY_A)
                    }) {
                        return Ok((device, format!("{} ({})", entry.path().display(), dev_name)));
                    }
                }
                Err(_) => continue,
            }
        }
    }

    Err("No keyboard device found".into())
}

fn run_with_evdev(
    mut device: evdev::Device,
    daemon: &mut Daemon,
) -> Result<(), Box<dyn std::error::Error>> {
    let injector = create_injector()?;
    let mut event_count = 0u64;

    loop {
        let key_state = device.get_key_state().ok();
        let events = device.fetch_events()?;

        // Check for app changes and config reload periodically
        event_count += 1;
        if event_count.is_multiple_of(100) {
            if daemon.config.app_state.enabled {
                daemon.check_app_change();
            }
            daemon.reload_config();
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

                if value != 1 {
                    continue;
                }

                if let Some(ch) = key_to_char(key) {
                    let commands = daemon.process_key(ch);
                    execute_commands(&*injector, &commands);
                }
            }
        }
    }
}

fn run_stdin_mode(daemon: &mut Daemon) -> Result<(), Box<dyn std::error::Error>> {
    use std::io::{self, Read};

    let injector = create_injector()?;
    let mut buffer = [0u8; 1];

    eprintln!("[vietc] Type to test, Ctrl+C to exit");

    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut byte_count = 0u64;

    loop {
        match handle.read(&mut buffer) {
            Ok(0) => break,
            Ok(_) => {
                let ch = buffer[0] as char;
                let commands = daemon.process_key(ch);
                execute_commands(&*injector, &commands);

                byte_count += 1;
                if byte_count.is_multiple_of(50) {
                    daemon.reload_config();
                }
            }
            Err(e) => {
                eprintln!("[vietc] Read error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn execute_commands(injector: &dyn vietc_protocol::KeyInjector, commands: &[OutputCommand]) {
    for cmd in commands {
        match cmd {
            OutputCommand::Type(text) => {
                injector.send_string(text);
            }
            OutputCommand::Backspace(count) => {
                injector.send_backspaces(*count);
            }
        }
    }
    injector.flush();
}

fn create_injector() -> Result<Box<dyn vietc_protocol::KeyInjector>, Box<dyn std::error::Error>> {
    // Try Wayland input method first (if compiled with wayland feature)
    #[cfg(feature = "wayland")]
    {
        // WaylandIMContext is always available — actual protocol binding
        // happens via the compositor's zwp_input_method_v2 protocol
        let _ctx = vietc_protocol::wayland_im::WaylandIMContext::new();
        eprintln!("[vietc] Wayland input method context initialized");
    }

    // Try X11 first (if compiled with x11 feature)
    #[cfg(feature = "x11")]
    {
        match vietc_protocol::x11_inject::X11Injector::new() {
            Ok(injector) => {
                eprintln!("[vietc] Using X11 injection (XTEST)");
                return Ok(Box::new(injector));
            }
            Err(e) => {
                eprintln!("[vietc] X11 not available: {}", e);
            }
        }
    }

    // Fall back to uinput (works on both X11 and Wayland)
    match vietc_protocol::uinput_monitor::UinputInjector::new("vietc") {
        Ok(injector) => {
            eprintln!("[vietc] Using uinput injection");
            Ok(Box::new(injector))
        }
        Err(e) => Err(format!("No injection backend available: {}", e).into()),
    }
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
