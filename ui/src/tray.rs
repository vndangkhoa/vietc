// SPDX-License-Identifier: MIT
use crate::config;
use ksni::{menu::*, MenuItem, Tray};

fn is_flatpak() -> bool {
    std::path::Path::new("/app/bin/vietc-daemon").exists()
}

fn write_status(state: &str) {
    if let Some(config_dir) = dirs::config_dir() {
        let _ = std::fs::write(config_dir.join("vietc").join("status"), state);
    }
}

fn read_method() -> String {
    let path = dirs::config_dir()
        .map(|d| d.join("vietc").join("method"))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/vietc-method"));
    std::fs::read_to_string(&path)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| {
            config::Config::load().input_method
        })
}

fn write_method(method: &str) {
    if let Some(config_dir) = dirs::config_dir() {
        let _ = std::fs::write(config_dir.join("vietc").join("method"), method);
    }
}

fn read_status() -> String {
    let path = dirs::config_dir()
        .map(|d| d.join("vietc").join("status"))
        .unwrap_or_else(|| std::path::PathBuf::from("/tmp/vietc-status"));

    std::fs::read_to_string(&path)
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|_| {
            let cfg = config::Config::load();
            if cfg.start_enabled {
                "vn".into()
            } else {
                "en".into()
            }
        })
}

fn current_im() -> String {
    config::Config::load().input_method
}



fn ensure_icons() {
    // SVG content for Viet+ icons
    let svg_vn = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128">
  <rect x="8" y="8" width="112" height="112" rx="24" fill="#e02424"/>
  <text x="64" y="96" text-anchor="middle" fill="#ffffff" font-size="48" font-weight="bold" font-family="system-ui, sans-serif">VN</text>
</svg>"##;

    let svg_tlx = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128">
  <rect x="8" y="8" width="112" height="112" rx="24" fill="#2563eb"/>
  <text x="64" y="96" text-anchor="middle" fill="#ffffff" font-size="48" font-weight="bold" font-family="system-ui, sans-serif">TLX</text>
</svg>"##;

    let svg_en = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 128 128">
  <rect x="8" y="8" width="112" height="112" rx="24" fill="#4b5563"/>
  <text x="64" y="96" text-anchor="middle" fill="#ffffff" font-size="48" font-weight="bold" font-family="system-ui, sans-serif">EN</text>
</svg>"##;

    if let Some(home) = dirs::home_dir() {
        let base = home.join(".local/share/icons");
        let _ = std::fs::create_dir_all(&base);
        let _ = std::fs::write(base.join("vietc-vn.svg"), svg_vn);
        let _ = std::fs::write(base.join("vietc-tlx.svg"), svg_tlx);
        let _ = std::fs::write(base.join("vietc-en.svg"), svg_en);

        for dir in &["scalable", "256x256", "128x128", "64x64", "48x48", "32x32"] {
            let apps = base.join("hicolor").join(dir).join("apps");
            let _ = std::fs::create_dir_all(&apps);
            let _ = std::fs::write(apps.join("vietc-vn.svg"), svg_vn);
            let _ = std::fs::write(apps.join("vietc-tlx.svg"), svg_tlx);
            let _ = std::fs::write(apps.join("vietc-en.svg"), svg_en);
        }
    }
}

fn show_notification(title: &str, body: &str) {
    let _ = std::process::Command::new("notify-send")
        .args([title, body])
        .status();
}

#[derive(serde::Deserialize, Clone, Debug)]
struct Asset {
    name: String,
    browser_download_url: String,
}

#[derive(serde::Deserialize, Clone, Debug)]
struct Release {
    tag_name: String,
    html_url: String,
    assets: Vec<Asset>,
}

struct VietTray {
    mode: String,
    im: String,
    autostart: bool,
    update_available: Option<Release>,
    updating: bool,
    handle: std::sync::Arc<std::sync::Mutex<Option<ksni::Handle<Self>>>>,
}

impl VietTray {
    fn check_for_updates(&self, handle: &ksni::Handle<Self>, verbose: bool) {
        let handle = handle.clone();
        std::thread::spawn(move || {
            if verbose {
                show_notification(
                    "Checking for updates...",
                    "Contacting git.khoavo.myds.me...",
                );
            }
            let output = std::process::Command::new("curl")
                .args([
                    "-s",
                    "https://git.khoavo.myds.me/api/v1/repos/vndangkhoa/vietc/releases",
                ])
                .output();

            match output {
                Ok(out) if out.status.success() => {
                    if let Ok(releases) = serde_json::from_slice::<Vec<Release>>(&out.stdout) {
                        if let Some(latest) = releases.first() {
                            let latest_ver = latest.tag_name.trim_start_matches('v');
                            let curr_ver = env!("CARGO_PKG_VERSION");
                            if latest_ver != curr_ver {
                                show_notification(
                                    "Viet+ Update Available",
                                    &format!("Version {} is available! Select 'Update to {}' in the tray menu.", latest.tag_name, latest.tag_name)
                                );
                                let rel = latest.clone();
                                let _ = handle.update(move |t| {
                                    t.update_available = Some(rel);
                                });
                                return;
                            }
                        }
                    }
                }
                _ => {}
            }
            if verbose {
                show_notification("Viet+ is Up-to-Date", "You are running the latest version.");
            }
        });
    }

    fn trigger_update(&self, handle: &ksni::Handle<Self>, release: Release) {
        let handle = handle.clone();
        let _ = handle.update(|t| t.updating = true);
        std::thread::spawn(move || {
            show_notification(
                "Downloading update...",
                &format!("Updating Viet+ to {}...", release.tag_name),
            );
            let appimage_asset = release
                .assets
                .iter()
                .find(|a| a.name.ends_with(".AppImage"));
            if let Some(asset) = appimage_asset {
                if let Ok(appimage_path) = std::env::var("APPIMAGE") {
                    let temp_path = format!("{}.tmp-update", appimage_path);
                    let status = std::process::Command::new("curl")
                        .args(["-L", "-o", &temp_path, &asset.browser_download_url])
                        .status();
                    match status {
                        Ok(s) if s.success() => {
                            use std::os::unix::fs::PermissionsExt;
                            if let Ok(_) = std::fs::set_permissions(
                                &temp_path,
                                std::fs::Permissions::from_mode(0o755),
                            ) {
                                if let Ok(_) = std::fs::rename(&temp_path, &appimage_path) {
                                    show_notification(
                                        "Update Succeeded",
                                        "Viet+ has been updated! Please restart the application.",
                                    );
                                    let _ = handle.update(|t| {
                                        t.updating = false;
                                        t.update_available = None;
                                    });
                                    return;
                                }
                            }
                        }
                        _ => {}
                    }
                    let _ = std::fs::remove_file(&temp_path);
                    show_notification("Update Failed", "Could not overwrite the AppImage file.");
                } else {
                    let _ = std::process::Command::new("xdg-open")
                        .arg(&release.html_url)
                        .status();
                    show_notification(
                        "Opening Releases Page",
                        "Please download the update manually.",
                    );
                }
            } else {
                show_notification("Update Failed", "No AppImage asset found in this release.");
            }
            let _ = handle.update(|t| t.updating = false);
        });
    }
}

impl Tray for VietTray {
    fn id(&self) -> String {
        "io.github.vietc.Tray".into()
    }
    fn title(&self) -> String {
        "Viet+".into()
    }

    fn icon_name(&self) -> String {
        let is_tlx = self.mode == "vn" && self.im == "telex";
        if is_flatpak() {
            if is_tlx {
                "io.github.vietc.VietPlus.vietc-tlx".into()
            } else if self.mode == "vn" {
                "io.github.vietc.VietPlus.vietc-vn".into()
            } else {
                "io.github.vietc.VietPlus.vietc-en".into()
            }
        } else if is_tlx {
            "vietc-tlx".into()
        } else if self.mode == "vn" {
            "vietc-vn".into()
        } else {
            "vietc-en".into()
        }
    }

    fn icon_theme_path(&self) -> String {
        // Use XDG user theme path for icons (works in both native and Flatpak)
        if let Some(home) = dirs::home_dir() {
            let user_path = home.join(".local/share/icons");
            if user_path.exists() {
                return user_path.to_string_lossy().into_owned();
            }
        }
        // Flatpak: icons are in /app/share/icons
        let flatpak_path = std::path::Path::new("/app/share/icons");
        if flatpak_path.exists() {
            return "/app/share/icons".into();
        }
        dirs::data_dir()
            .map(|d| d.join("icons").to_string_lossy().into_owned())
            .unwrap_or_else(|| "/usr/share/icons".into())
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        vec![]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let next = if self.mode == "vn" { "en" } else { "vn" };
        write_status(&next);
        let mut cfg = config::Config::load();
        cfg.start_enabled = next == "vn";
        let _ = cfg.save();
        self.mode = next.to_string();
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let is_vn = self.mode == "vn";
        let is_telex = self.im == "telex";
        let selected_mode = if !is_vn {
            0_usize
        } else if is_telex {
            1_usize
        } else {
            2_usize
        };

        let mut items = vec![
            CheckmarkItem {
                label: "Start with System".into(),
                checked: self.autostart,
                activate: Box::new(|this: &mut VietTray| {
                    if this.autostart {
                        config::uninstall_autostart();
                    } else {
                        config::install_autostart();
                    }
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Switch Mode (Ctrl + Shift)".into(),
                activate: Box::new(|this: &mut VietTray| {
                    let (next_mode, next_im) = if this.mode == "en" {
                        ("vn", "vni")
                    } else if this.im == "vni" {
                        ("vn", "telex")
                    } else {
                        ("en", "telex")
                    };
                    write_status(next_mode);
                    write_method(next_im);
                    let mut cfg = config::Config::load();
                    cfg.start_enabled = next_mode == "vn";
                    cfg.input_method = next_im.into();
                    let _ = cfg.save();
                    this.mode = next_mode.to_string();
                    this.im = next_im.to_string();
                }),
                ..Default::default()
            }
            .into(),
            SubMenu {
                label: "Input Method".into(),
                submenu: vec![RadioGroup {
                    selected: selected_mode,
                    select: Box::new(|this: &mut VietTray, idx: usize| {
                        let (mode, im) = match idx {
                            0 => ("en", this.im.as_str()),
                            1 => ("vn", "telex"),
                            _ => ("vn", "vni"),
                        };
                        write_status(mode);
                        write_method(im);
                        let mut cfg = config::Config::load();
                        cfg.start_enabled = mode == "vn";
                        cfg.input_method = im.into();
                        let _ = cfg.save();
                        this.mode = mode.into();
                        this.im = im.into();
                    }),
                    options: vec![
                        RadioItem {
                            label: "English (ENG)".into(),
                            ..Default::default()
                        },
                        RadioItem {
                            label: "Telex (TLX)".into(),
                            ..Default::default()
                        },
                        RadioItem {
                            label: "VNI (VN)".into(),
                            ..Default::default()
                        },
                    ],
                }
                .into()],
                ..Default::default()
            }
            .into(),
        ];

        items.push(MenuItem::Separator);
        if let Some(ref release) = self.update_available {
            let label = if self.updating {
                "Updating...".into()
            } else {
                format!("Update to {}", release.tag_name)
            };
            items.push(
                StandardItem {
                    label,
                    activate: Box::new(|this: &mut VietTray| {
                        if !this.updating {
                            if let Some(ref rel) = this.update_available.clone() {
                                let handle = this.handle.lock().unwrap().clone().unwrap();
                                this.trigger_update(&handle, rel.clone());
                            }
                        }
                    }),
                    ..Default::default()
                }
                .into(),
            );
        } else {
            items.push(
                StandardItem {
                    label: if self.updating {
                        "Updating...".into()
                    } else {
                        "Check for Updates".into()
                    },
                    activate: Box::new(|this: &mut VietTray| {
                        if !this.updating {
                            let handle = this.handle.lock().unwrap().clone().unwrap();
                            this.check_for_updates(&handle, true);
                        }
                    }),
                    ..Default::default()
                }
                .into(),
            );
        }

        items.push(MenuItem::Separator);
        items.push(
            StandardItem {
                label: "About: Viet+".into(),
                activate: Box::new(|_| {
                    let _ = std::process::Command::new("xdg-open")
                        .arg("https://github.com/vndangkhoa/vietc")
                        .status();
                }),
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);
        items.push(
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|_| {
                    crate::stop_daemon();
                    std::process::exit(0);
                }),
                ..Default::default()
            }
            .into(),
        );

        items
    }
}

pub fn run() {
    ensure_icons();

    // Ensure autostart is installed by default
    if !config::is_autostart_installed() {
        config::install_autostart();
    }

    let handle_holder = std::sync::Arc::new(std::sync::Mutex::new(None));
    let tray = VietTray {
        mode: read_status(),
        im: current_im(),
        autostart: config::is_autostart_installed(),
        update_available: None,
        updating: false,
        handle: handle_holder.clone(),
    };

    let service = ksni::TrayService::new(tray);
    let handle = service.handle();
    *handle_holder.lock().unwrap() = Some(handle.clone());
    service.spawn();

    // Poll for changes
    std::thread::spawn(move || {
        let mut last_mode = String::new();
        let mut last_im = String::new();
        let mut last_autostart = false;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let mode = read_status();
            let im = read_method();
            let autostart = config::is_autostart_installed();
            if mode != last_mode || im != last_im || autostart != last_autostart {
                last_mode = mode.clone();
                last_im = im.clone();
                last_autostart = autostart;
                let _ = handle.update(move |t| {
                    t.mode = mode;
                    t.im = im;
                    t.autostart = autostart;
                });
            }
        }
    });

    loop {
        std::thread::park();
    }
}
