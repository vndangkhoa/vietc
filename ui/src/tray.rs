use ksni::{Tray, MenuItem, menu::*};
use crate::config;

fn write_status(state: &str) {
    if let Some(config_dir) = dirs::config_dir() {
        let _ = std::fs::write(config_dir.join("vietc").join("status"), state);
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
            if cfg.start_enabled { "vn".into() } else { "en".into() }
        })
}

fn current_im() -> String {
    config::Config::load().input_method
}

fn ensure_icons() {
    let Some(config_dir) = dirs::config_dir() else { return };
    let icons_dir = config_dir.join("vietc").join("icons");
    let _ = std::fs::create_dir_all(&icons_dir);

    let vn_path = icons_dir.join("vietc-vn.svg");
    let en_path = icons_dir.join("vietc-en.svg");

    if !vn_path.exists() {
        let _ = std::fs::write(&vn_path, r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="32" height="32">
  <rect x="2" y="2" width="28" height="28" rx="6" fill="#e02424"/>
  <text x="16" y="22" text-anchor="middle" fill="#ffffff" font-size="14" font-weight="900" font-family="system-ui, sans-serif">VN</text>
</svg>"##);
    }

    if !en_path.exists() {
        let _ = std::fs::write(&en_path, r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32" width="32" height="32">
  <rect x="2" y="2" width="28" height="28" rx="6" fill="#4b5563"/>
  <text x="16" y="22" text-anchor="middle" fill="#ffffff" font-size="14" font-weight="900" font-family="system-ui, sans-serif">EN</text>
</svg>"##);
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
                show_notification("Checking for updates...", "Contacting git.khoavo.myds.me...");
            }
            let output = std::process::Command::new("curl")
                .args(["-s", "https://git.khoavo.myds.me/api/v1/repos/vndangkhoa/vietc/releases"])
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
            show_notification("Downloading update...", &format!("Updating Viet+ to {}...", release.tag_name));
            let appimage_asset = release.assets.iter().find(|a| a.name.ends_with(".AppImage"));
            if let Some(asset) = appimage_asset {
                if let Ok(appimage_path) = std::env::var("APPIMAGE") {
                    let temp_path = format!("{}.tmp-update", appimage_path);
                    let status = std::process::Command::new("curl")
                        .args(["-L", "-o", &temp_path, &asset.browser_download_url])
                        .status();
                    match status {
                        Ok(s) if s.success() => {
                            use std::os::unix::fs::PermissionsExt;
                            if let Ok(_) = std::fs::set_permissions(&temp_path, std::fs::Permissions::from_mode(0o755)) {
                                if let Ok(_) = std::fs::rename(&temp_path, &appimage_path) {
                                    show_notification(
                                        "Update Succeeded",
                                        "Viet+ has been updated! Please restart the application."
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
                        "Please download the update manually."
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
    fn id(&self) -> String { "io.github.vietc.Tray".into() }
    fn title(&self) -> String { "Viet+".into() }

    fn icon_name(&self) -> String {
        if self.mode == "vn" { "vietc-vn".into() } else { "vietc-en".into() }
    }

    fn icon_theme_path(&self) -> String {
        dirs::config_dir()
            .map(|d| d.join("vietc").join("icons").to_string_lossy().into_owned())
            .unwrap_or_default()
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
        let im_index = if self.im == "telex" { 0 } else { 1 };

        let mut items = vec![
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
            }.into(),
            MenuItem::Separator,
            SubMenu {
                label: "Input Method".into(),
                submenu: vec![
                    RadioGroup {
                        selected: im_index,
                        select: Box::new(|this: &mut VietTray, idx: usize| {
                            let im = if idx == 0 { "telex" } else { "vni" };
                            let mut cfg = config::Config::load();
                            cfg.input_method = im.into();
                            let _ = cfg.save();
                            this.im = im.into();
                        }),
                        options: vec![
                            RadioItem { label: "Telex".into(), ..Default::default() },
                            RadioItem { label: "VNI".into(), ..Default::default() },
                        ],
                    }.into(),
                ],
                ..Default::default()
            }.into(),
            MenuItem::Separator,
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
            }.into(),
        ];

        items.push(MenuItem::Separator);
        if let Some(ref release) = self.update_available {
            let label = if self.updating {
                "Updating...".into()
            } else {
                format!("Update to {}", release.tag_name)
            };
            items.push(StandardItem {
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
            }.into());
        } else {
            items.push(StandardItem {
                label: if self.updating { "Updating...".into() } else { "Check for Updates".into() },
                activate: Box::new(|this: &mut VietTray| {
                    if !this.updating {
                        let handle = this.handle.lock().unwrap().clone().unwrap();
                        this.check_for_updates(&handle, true);
                    }
                }),
                ..Default::default()
            }.into());
        }

        items.push(MenuItem::Separator);
        items.push(StandardItem {
            label: "about me - khoavo.myds.me".into(),
            activate: Box::new(|_| {
                let _ = std::process::Command::new("xdg-open")
                    .arg("https://khoavo.myds.me")
                    .status();
            }),
            ..Default::default()
        }.into());

        items.push(MenuItem::Separator);
        items.push(StandardItem {
            label: "Quit".into(),
            activate: Box::new(|_| {
                let _ = std::process::Command::new("pkill")
                    .arg("-x").arg("vietc").status();
                std::process::exit(0);
            }),
            ..Default::default()
        }.into());

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

    // Poll for changes
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));
            let mode = read_status();
            let im = current_im();
            let autostart = config::is_autostart_installed();
            let _ = handle.update(move |t| {
                t.mode = mode;
                t.im = im;
                t.autostart = autostart;
            });
        }
    });

    loop { std::thread::park(); }
}
