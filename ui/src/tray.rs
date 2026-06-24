use ksni::Tray;

struct VietcTray;

impl Tray for VietcTray {
    fn id(&self) -> String {
        "io.github.vietc.Tray".into()
    }

    fn title(&self) -> String {
        "Viet+".into()
    }

    fn icon_name(&self) -> String {
        "input-keyboard".into()
    }

    fn menu(&self) -> ksni::Menu {
        ksni::Menu {
            items: vec![
                ksni::MenuItem::label("Toggle Vietnamese/English").into(),
                ksni::MenuItem::separator().into(),
                ksni::MenuItem::label("Settings...").into(),
                ksni::MenuItem::separator().into(),
                ksni::MenuItem::label("Quit Viet+").into(),
            ],
        }
    }
}

fn main() {
    let service = ksni::TrayService::new(VietcTray);
    service.spawn();
    loop {
        std::thread::park();
    }
}
