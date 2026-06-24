use ksni::{Tray, MenuItem, menu::*};
mod config;
use config::Config;

/// Get the directory where the current executable lives.
/// This handles AppImage, DEB installs, and dev builds correctly.
fn exe_dir() -> std::path::PathBuf {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| std::path::PathBuf::from("/usr/bin"))
}

/// Find a sibling binary (in the same directory as the current executable).
/// Also searches the workspace target directory for development.
/// Falls back to searching PATH if not found next to the executable.
fn find_sibling_binary(name: &str) -> String {
    // 1. Same directory
    let sibling = exe_dir().join(name);
    if sibling.exists() {
        return sibling.to_string_lossy().into_owned();
    }

    // 2. Dev target/debug relative path (from ui/target/debug)
    let dev_debug = exe_dir().join("..").join("..").join("..").join("target").join("debug").join(name);
    if dev_debug.exists() {
        return dev_debug.to_string_lossy().into_owned();
    }

    // 3. Dev target/release relative path (from ui/target/release)
    let dev_release = exe_dir().join("..").join("..").join("..").join("target").join("release").join(name);
    if dev_release.exists() {
        return dev_release.to_string_lossy().into_owned();
    }

    name.to_string()
}

struct VietcTray {
    active_mode: String,
    autostart_enabled: bool,
}

impl Tray for VietcTray {
    fn id(&self) -> String {
        "io.github.vietc.Tray".into()
    }

    fn title(&self) -> String {
        "Viet+".into()
    }

    fn icon_name(&self) -> String {
        if self.active_mode == "vn" {
            "vietc-vn".into()
        } else {
            "vietc-en".into()
        }
    }

    fn icon_theme_path(&self) -> String {
        if let Some(config_dir) = dirs::config_dir() {
            config_dir.join("vietc").join("icons").to_string_lossy().into_owned()
        } else {
            "".into()
        }
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let is_vn = self.active_mode == "vn";
        vec![
            CheckmarkItem {
                label: "Vietnamese Mode".into(),
                checked: is_vn,
                activate: Box::new(|this: &mut VietcTray| {
                    let next_state = if this.active_mode == "vn" { "en" } else { "vn" };
                    if let Some(config_dir) = dirs::config_dir() {
                        let status_path = config_dir.join("vietc").join("status");
                        let _ = std::fs::write(&status_path, next_state);
                    }
                    
                    // Also save start_enabled to config, so it persists across reboots
                    let mut config = Config::load();
                    config.start_enabled = next_state == "vn";
                    let _ = config.save();
                }),
                ..Default::default()
            }.into(),
            CheckmarkItem {
                label: "Autostart on Boot".into(),
                checked: self.autostart_enabled,
                activate: Box::new(|this: &mut VietcTray| {
                    if this.autostart_enabled {
                        config::uninstall_autostart();
                    } else {
                        config::install_autostart_force();
                    }
                }),
                ..Default::default()
            }.into(),
            MenuItem::Separator,
            StandardItem {
                label: "Settings...".into(),
                activate: Box::new(|_| {
                    let settings_bin = find_sibling_binary("vietc-settings");
                    eprintln!("[vietc-tray] Launching settings: {}", settings_bin);
                    match std::process::Command::new(&settings_bin).spawn() {
                        Ok(_) => {},
                        Err(e) => eprintln!("[vietc-tray] Failed to launch settings: {}", e),
                    }
                }),
                ..Default::default()
            }.into(),
            MenuItem::Separator,
            StandardItem {
                label: "Quit Viet+".into(),
                activate: Box::new(|_| {
                    let _ = std::process::Command::new("pkill")
                        .arg("-x")
                        .arg("vietc")
                        .status();
                    std::process::exit(0);
                }),
                ..Default::default()
            }.into(),
        ]
    }
}

fn is_daemon_running() -> bool {
    std::process::Command::new("pgrep")
        .arg("-x")
        .arg("vietc")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

fn ensure_icons_exist() {
    if let Some(config_dir) = dirs::config_dir() {
        let icons_dir = config_dir.join("vietc").join("icons");
        let _ = std::fs::create_dir_all(&icons_dir);

        let vn_path = icons_dir.join("vietc-vn.svg");
        let en_path = icons_dir.join("vietc-en.svg");

        let vn_svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="32" height="32">
  <rect x="2" y="2" width="28" height="28" rx="6" fill="#e02424"/>
  <text x="16" y="21" text-anchor="middle" fill="#ffffff" font-size="13" font-weight="900" font-family="system-ui, -apple-system, sans-serif" letter-spacing="0.5">VN</text>
</svg>"##;

        let en_svg = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="32" height="32">
  <rect x="2" y="2" width="28" height="28" rx="6" fill="#4b5563"/>
  <text x="16" y="21" text-anchor="middle" fill="#ffffff" font-size="13" font-weight="900" font-family="system-ui, -apple-system, sans-serif" letter-spacing="0.5">EN</text>
</svg>"##;

        let _ = std::fs::write(&vn_path, vn_svg);
        let _ = std::fs::write(&en_path, en_svg);

        let hicolor_apps_dir = icons_dir.join("hicolor").join("scalable").join("apps");
        let _ = std::fs::create_dir_all(&hicolor_apps_dir);
        let _ = std::fs::write(hicolor_apps_dir.join("vietc-vn.svg"), vn_svg);
        let _ = std::fs::write(hicolor_apps_dir.join("vietc-en.svg"), en_svg);
    }
}

fn main() {
    eprintln!("[vietc-tray] Starting tray (exe dir: {:?})", exe_dir());

    ensure_icons_exist();

    if !is_daemon_running() {
        let daemon_bin = find_sibling_binary("vietc");
        eprintln!("[vietc-tray] Starting daemon: {}", daemon_bin);
        match std::process::Command::new(&daemon_bin).spawn() {
            Ok(child) => eprintln!("[vietc-tray] Daemon started (PID {})", child.id()),
            Err(e) => eprintln!("[vietc-tray] Failed to start daemon: {}", e),
        }
    } else {
        eprintln!("[vietc-tray] Daemon already running");
    }

    let tray = VietcTray {
        active_mode: "en".into(),
        autostart_enabled: config::is_autostart_installed(),
    };

    let service = ksni::TrayService::new(tray);
    let handle = service.handle();
    service.spawn();

    let handle_clone = handle.clone();
    std::thread::spawn(move || {
        let status_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/tmp"))
            .join("vietc")
            .join("status");

        loop {
            let active_mode = if let Ok(content) = std::fs::read_to_string(&status_path) {
                content.trim().to_string()
            } else {
                let config = Config::load();
                if config.start_enabled { "vn".to_string() } else { "en".to_string() }
            };

            let autostart_enabled = config::is_autostart_installed();

            let _ = handle_clone.update(move |t| {
                t.active_mode = active_mode;
                t.autostart_enabled = autostart_enabled;
            });

            std::thread::sleep(std::time::Duration::from_millis(250));
        }
    });

    if config::is_autostart_installed() {
        config::install_autostart_force();
    }

    loop {
        std::thread::park();
    }
}
