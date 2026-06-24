use adw::prelude::*;
use gtk::{gio, glib};

mod config;
mod window;

use window::SettingsWindow;

fn main() -> glib::ExitCode {
    let app = adw::Application::builder()
        .application_id("io.github.vietc.Settings")
        .flags(gio::ApplicationFlags::FLAGS_NONE)
        .build();

    app.connect_activate(|app| {
        let window = SettingsWindow::new(app);
        window.present();
    });

    app.run()
}
