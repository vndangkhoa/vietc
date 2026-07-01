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

fn draw_line(data: &mut [u8], x0: i32, y0: i32, x1: i32, y1: i32, color: [u8; 4]) {
    let dx = (x1 - x0).abs();
    let dy = (y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx - dy;
    let mut x = x0;
    let mut y = y0;
    loop {
        if x >= 0 && x < 32 && y >= 0 && y < 32 {
            let idx = ((y * 32 + x) * 4) as usize;
            data[idx..idx + 4].copy_from_slice(&color);
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 > -dy {
            err -= dy;
            x += sx;
        }
        if e2 < dx {
            err += dx;
            y += sy;
        }
    }
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

    // Write to standard user theme path (for Wayland compositors)
    let home = dirs::home_dir().map(|d| d.join(".local/share/icons"));
    if let Some(home_icons) = &home {
        let _ = std::fs::create_dir_all(&home_icons);
        let vn_path = home_icons.join("vietc-vn.svg");
        let tlx_path = home_icons.join("vietc-tlx.svg");
        let en_path = home_icons.join("vietc-en.svg");

        if !vn_path.exists() {
            let _ = std::fs::write(&vn_path, svg_vn);
        }
        if !tlx_path.exists() {
            let _ = std::fs::write(&tlx_path, svg_tlx);
        }
        if !en_path.exists() {
            let _ = std::fs::write(&en_path, svg_en);
        }
    }

    // Also write to config dir for AppImage compatibility (fallback)
    let config_dir = dirs::config_dir();
    if let Some(config_dir) = &config_dir {
        let icons_dir = config_dir.join("vietc").join("icons");
        let _ = std::fs::create_dir_all(&icons_dir);

        let vn_theme = icons_dir.join("hicolor/scalable/apps/vietc-vn.svg");
        let tlx_theme = icons_dir.join("hicolor/scalable/apps/vietc-tlx.svg");
        let en_theme = icons_dir.join("hicolor/scalable/apps/vietc-en.svg");

        if !vn_theme.exists() {
            let _ = std::fs::write(&vn_theme, svg_vn);
        }
        if !tlx_theme.exists() {
            let _ = std::fs::write(&tlx_theme, svg_tlx);
        }
        if !en_theme.exists() {
            let _ = std::fs::write(&en_theme, svg_en);
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
        let is_vn = self.mode == "vn";
        let is_tlx = self.mode == "vn" && self.im == "telex";
        let bg_color = if is_vn && !is_tlx {
            [255, 224, 36, 36] // Red for VNI
        } else if is_tlx {
            [255, 37, 99, 235] // Blue for Telex
        } else {
            [255, 75, 85, 99]  // Gray for English
        };
        let fg_color = [255, 255, 255, 255];

        let mut data = vec![0u8; 32 * 32 * 4];
        for y in 0..32 {
            for x in 0..32 {
                let mut inside = true;
                if x < 7 && y < 7 {
                    if (x - 7) * (x - 7) + (y - 7) * (y - 7) > 36 {
                        inside = false;
                    }
                } else if x > 24 && y < 7 {
                    if (x - 24) * (x - 24) + (y - 7) * (y - 7) > 36 {
                        inside = false;
                    }
                } else if x < 7 && y > 24 {
                    if (x - 7) * (x - 7) + (y - 24) * (y - 24) > 36 {
                        inside = false;
                    }
                } else if x > 24 && y > 24 {
                    if (x - 24) * (x - 24) + (y - 24) * (y - 24) > 36 {
                        inside = false;
                    }
                }

                let idx = ((y * 32 + x) * 4) as usize;
                if inside {
                    data[idx] = bg_color[0];
                    data[idx + 1] = bg_color[1];
                    data[idx + 2] = bg_color[2];
                    data[idx + 3] = bg_color[3];
                }
            }
        }

        if is_tlx {
            // T
            draw_line(&mut data, 6, 10, 15, 10, fg_color);
            draw_line(&mut data, 6, 11, 15, 11, fg_color);
            draw_line(&mut data, 10, 10, 10, 21, fg_color);
            draw_line(&mut data, 11, 10, 11, 21, fg_color);
            // X
            draw_line(&mut data, 18, 10, 26, 21, fg_color);
            draw_line(&mut data, 19, 10, 27, 21, fg_color);
            draw_line(&mut data, 26, 10, 18, 21, fg_color);
            draw_line(&mut data, 27, 10, 19, 21, fg_color);
        } else if is_vn {
            // V
            draw_line(&mut data, 6, 10, 11, 21, fg_color);
            draw_line(&mut data, 7, 10, 12, 21, fg_color);
            draw_line(&mut data, 11, 21, 15, 10, fg_color);
            draw_line(&mut data, 12, 21, 16, 10, fg_color);
            // N
            draw_line(&mut data, 18, 10, 18, 21, fg_color);
            draw_line(&mut data, 19, 10, 19, 21, fg_color);
            draw_line(&mut data, 18, 10, 26, 21, fg_color);
            draw_line(&mut data, 19, 10, 27, 21, fg_color);
            draw_line(&mut data, 26, 10, 26, 21, fg_color);
            draw_line(&mut data, 27, 10, 27, 21, fg_color);
        } else {
            // E
            draw_line(&mut data, 6, 10, 6, 21, fg_color);
            draw_line(&mut data, 7, 10, 7, 21, fg_color);
            draw_line(&mut data, 6, 10, 15, 10, fg_color);
            draw_line(&mut data, 6, 11, 15, 11, fg_color);
            draw_line(&mut data, 6, 15, 13, 15, fg_color);
            draw_line(&mut data, 6, 16, 13, 16, fg_color);
            draw_line(&mut data, 6, 20, 15, 20, fg_color);
            draw_line(&mut data, 6, 21, 15, 21, fg_color);
            // N
            draw_line(&mut data, 18, 10, 18, 21, fg_color);
            draw_line(&mut data, 19, 10, 19, 21, fg_color);
            draw_line(&mut data, 18, 10, 26, 21, fg_color);
            draw_line(&mut data, 19, 10, 27, 21, fg_color);
            draw_line(&mut data, 26, 10, 26, 21, fg_color);
            draw_line(&mut data, 27, 10, 27, 21, fg_color);
        }

        vec![ksni::Icon {
            width: 32,
            height: 32,
            data,
        }]
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
        let im_index = if self.im == "telex" { 0_usize } else { 1_usize };

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
            CheckmarkItem {
                label: "Vietnamese Mode".into(),
                checked: is_vn,
                activate: Box::new(|this: &mut VietTray| {
                    let next = if this.mode == "vn" { "en" } else { "vn" };
                    write_status(&next);
                    let mut cfg = config::Config::load();
                    cfg.start_enabled = next == "vn";
                    let _ = cfg.save();
                    this.mode = next.to_string();
                }),
                ..Default::default()
            }
            .into(),
            SubMenu {
                label: "Input Method".into(),
                submenu: vec![RadioGroup {
                    selected: im_index,
                    select: Box::new(|this: &mut VietTray, idx: usize| {
                        let im = if idx == 0 { "telex" } else { "vni" };
                        let mut cfg = config::Config::load();
                        cfg.input_method = im.into();
                        let _ = cfg.save();
                        write_method(im);
                        this.im = im.into();
                    }),
                    options: vec![
                        RadioItem {
                            label: "Telex".into(),
                            ..Default::default()
                        },
                        RadioItem {
                            label: "VNI".into(),
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
                    let _ = std::process::Command::new("pkill")
                        .arg("-x")
                        .arg("vietc")
                        .status();
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

    // Check updates silently on startup
    {
        let tray_dummy = VietTray {
            mode: read_status(),
            im: current_im(),
            autostart: config::is_autostart_installed(),
            update_available: None,
            updating: false,
            handle: handle_holder.clone(),
        };
        tray_dummy.check_for_updates(&handle, false);
    }

    // Poll for changes (shorter interval for faster icon updates)
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            let mode = read_status();
            let im = read_method();
            let autostart = config::is_autostart_installed();
            // Also check status_changed flag for immediate updates
            let _ = handle.update(move |t| {
                t.mode = mode;
                t.im = im;
                t.autostart = autostart;
                // Force icon redraw on update by updating pixmap-related state
            });
        }
    });

    loop {
        std::thread::park();
    }
}
