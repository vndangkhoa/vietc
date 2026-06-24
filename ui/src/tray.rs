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

struct VietTray {
    mode: String,
    im: String,
    autostart: bool,
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

        vec![
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
            MenuItem::Separator,
            StandardItem {
                label: "Quit".into(),
                activate: Box::new(|_| {
                    let _ = std::process::Command::new("pkill")
                        .arg("-x").arg("vietc").status();
                    std::process::exit(0);
                }),
                ..Default::default()
            }.into(),
        ]
    }
}

pub fn run() {
    ensure_icons();

    let tray = VietTray {
        mode: read_status(),
        im: current_im(),
        autostart: config::is_autostart_installed(),
    };

    let service = ksni::TrayService::new(tray);
    let handle = service.handle();
    service.spawn();

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
