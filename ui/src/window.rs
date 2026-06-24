use adw::prelude::*;
use adw::subclass::prelude::*;
use gtk::{gio, glib};

use crate::config::Config;

mod imp {
    use super::*;
    use std::cell::RefCell;

    #[derive(Default)]
    pub struct SettingsWindow {
        pub dirty: RefCell<bool>,
    }

    #[glib::object_subclass]
    impl ObjectSubclass for SettingsWindow {
        const NAME: &'static str = "SettingsWindow";
        type Type = super::SettingsWindow;
        type ParentType = adw::ApplicationWindow;
    }

    impl ObjectImpl for SettingsWindow {}
    impl WidgetImpl for SettingsWindow {}
    impl WindowImpl for SettingsWindow {}
    impl ApplicationWindowImpl for SettingsWindow {}
    impl AdwApplicationWindowImpl for SettingsWindow {}
}

glib::wrapper! {
    pub struct SettingsWindow(ObjectSubclass<imp::SettingsWindow>)
        @extends adw::ApplicationWindow, gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible, gtk::Buildable;
}

impl SettingsWindow {
    pub fn new(app: &adw::Application) -> Self {
        let win: Self = glib::Object::builder()
            .property("application", app)
            .property("default-width", 580)
            .property("default-height", 500)
            .property("title", "Viet+ Settings")
            .build();

        win.build_ui();
        win
    }

    fn mark_dirty(&self) {
        *self.imp().dirty.borrow_mut() = true;
    }

    fn build_ui(&self) {
        let config = Config::load();
        let trigger_keys = config.auto_restore.trigger_keys.clone();

        // Toast overlay for notifications
        let toast_overlay = adw::ToastOverlay::new();

        // Main box
        let main_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .build();

        // Header bar with view switcher
        let header = adw::HeaderBar::new();

        // View Stack
        let stack = adw::ViewStack::builder()
            .vexpand(true)
            .build();

        // View Switcher linked to stack
        let switcher = adw::ViewSwitcher::builder()
            .stack(&stack)
            .build();
        header.set_title_widget(Some(&switcher));

        // Save button (suggested action)
        let save_btn = gtk::Button::builder()
            .label("Save")
            .css_classes(["suggested-action"])
            .tooltip_text("Save settings (Ctrl+S)")
            .build();
        header.pack_end(&save_btn);

        // Keyboard shortcut for save
        let controller = gtk::EventControllerKey::new();
        let save_ref = save_btn.clone();
        controller.connect_key_pressed(move |_, key, _, modifiers| {
            if modifiers.contains(gtk::gdk::ModifierType::CONTROL_MASK)
                && key == gtk::gdk::Key::s
            {
                save_ref.emit_clicked();
                glib::Propagation::Stop
            } else {
                glib::Propagation::Proceed
            }
        });
        self.add_controller(controller);

        main_box.append(&header);

        // ==================== Page 1: Typing ====================
        let typing_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .margin_top(16)
            .margin_bottom(16)
            .margin_start(16)
            .margin_end(16)
            .build();

        // ========== Input Method Section ==========
        let method_group = adw::PreferencesGroup::builder()
            .title("Input Method")
            .description("Select your preferred Vietnamese typing method")
            .build();

        let method_row = adw::ComboRow::builder()
            .title("Keyboard Layout")
            .subtitle("Telex uses letters (aa=ă, ee=ê), VNI uses digits (a6=ă, e8=ê)")
            .model(&gtk::StringList::new(&["Telex (Recommended)", "VNI"]))
            .selected(if config.input_method == "vni" { 1 } else { 0 })
            .build();

        let toggle_row = adw::ComboRow::builder()
            .title("Toggle Key")
            .subtitle("Switch between Vietnamese and English input")
            .model(&gtk::StringList::new(&[
                "Ctrl + Space",
                "Ctrl + Shift",
                "Caps Lock",
            ]))
            .selected(match config.toggle_key.as_str() {
                "shift" => 1,
                "capslock" => 2,
                _ => 0,
            })
            .build();

        method_group.add(&method_row);
        method_group.add(&toggle_row);
        typing_box.append(&method_group);

        // ========== General Section ==========
        let general_group = adw::PreferencesGroup::builder()
            .title("General")
            .build();

        let start_enabled_row = adw::SwitchRow::builder()
            .title("Start Enabled")
            .subtitle("Enable Vietnamese input on startup")
            .active(config.start_enabled)
            .build();

        let app_memory_row = adw::SwitchRow::builder()
            .title("App Memory")
            .subtitle("Remember per-app Vietnamese/English state")
            .active(config.app_state.enabled)
            .build();

        let auto_restore_row = adw::SwitchRow::builder()
            .title("Auto Restore English")
            .subtitle("Automatically restore common English words")
            .active(config.auto_restore.enabled)
            .build();

        let autostart_row = adw::SwitchRow::builder()
            .title("Autostart on Boot")
            .subtitle("Start Viet+ automatically when your system starts")
            .active(crate::config::is_autostart_installed())
            .build();

        general_group.add(&start_enabled_row);
        general_group.add(&app_memory_row);
        general_group.add(&auto_restore_row);
        general_group.add(&autostart_row);
        typing_box.append(&general_group);

        let typing_clamp = adw::Clamp::builder().maximum_size(540).tightening_threshold(400).build();
        typing_clamp.set_child(Some(&typing_box));
        let typing_scrolled = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&typing_clamp)
            .build();
        stack.add_titled(&typing_scrolled, Some("typing"), "Typing");

        // ==================== Page 2: Apps ====================
        let apps_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .margin_top(16)
            .margin_bottom(16)
            .margin_start(16)
            .margin_end(16)
            .build();

        let apps_group = adw::PreferencesGroup::builder()
            .title("Application Lists")
            .description("Override input method for specific applications")
            .build();

        // English apps
        let english_list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();

        for app in &config.app_state.english_apps {
            english_list.append(&Self::make_app_row_static(app, &english_list));
        }

        let english_entry = gtk::SearchEntry::builder()
            .placeholder_text("Add application name...")
            .hexpand(true)
            .build();

        let english_add = gtk::Button::builder()
            .icon_name("list-add-symbolic")
            .css_classes(["flat", "accent"])
            .tooltip_text("Add application")
            .build();

        let english_input = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(4)
            .build();
        english_input.append(&english_entry);
        english_input.append(&english_add);

        let english_header = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .build();
        let english_label = gtk::Label::builder()
            .label("English Mode (Telex disabled)")
            .halign(gtk::Align::Start)
            .css_classes(["heading", "dim-label"])
            .build();
        english_header.append(&english_label);
        english_header.append(&english_list);
        english_header.append(&english_input);

        let english_row = adw::ActionRow::builder()
            .title("English Applications")
            .activatable(false)
            .build();
        english_row.add_suffix(&english_header);
        apps_group.add(&english_row);

        // Vietnamese apps
        let viet_list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();

        for app in &config.app_state.vietnamese_apps {
            viet_list.append(&Self::make_app_row_static(app, &viet_list));
        }

        let viet_entry = gtk::SearchEntry::builder()
            .placeholder_text("Add application name...")
            .hexpand(true)
            .build();

        let viet_add = gtk::Button::builder()
            .icon_name("list-add-symbolic")
            .css_classes(["flat", "accent"])
            .tooltip_text("Add application")
            .build();

        let viet_input = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(4)
            .build();
        viet_input.append(&viet_entry);
        viet_input.append(&viet_add);

        let viet_header = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .build();
        let viet_label = gtk::Label::builder()
            .label("Vietnamese Mode (Telex enabled)")
            .halign(gtk::Align::Start)
            .css_classes(["heading", "dim-label"])
            .build();
        viet_header.append(&viet_label);
        viet_header.append(&viet_list);
        viet_header.append(&viet_input);

        let viet_row = adw::ActionRow::builder()
            .title("Vietnamese Applications")
            .activatable(false)
            .build();
        viet_row.add_suffix(&viet_header);
        apps_group.add(&viet_row);

        apps_box.append(&apps_group);

        let apps_clamp = adw::Clamp::builder().maximum_size(540).tightening_threshold(400).build();
        apps_clamp.set_child(Some(&apps_box));
        let apps_scrolled = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&apps_clamp)
            .build();
        stack.add_titled(&apps_scrolled, Some("apps"), "Apps");

        // ==================== Page 3: Shortcuts ====================
        let shortcuts_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Vertical)
            .spacing(8)
            .margin_top(16)
            .margin_bottom(16)
            .margin_start(16)
            .margin_end(16)
            .build();

        // ========== Macros Section ==========
        let macros_group = adw::PreferencesGroup::builder()
            .title("Macros")
            .description("Type shortcuts that expand to Vietnamese phrases")
            .build();

        let macros_list = gtk::ListBox::builder()
            .selection_mode(gtk::SelectionMode::None)
            .css_classes(["boxed-list"])
            .build();

        for (shortcut, expansion) in &config.macros {
            macros_list.append(&Self::make_macro_row_static(shortcut, expansion, &macros_list));
        }

        let macro_shortcut = gtk::SearchEntry::builder()
            .placeholder_text("ko")
            .width_chars(8)
            .build();

        let macro_expansion = gtk::SearchEntry::builder()
            .placeholder_text("không")
            .hexpand(true)
            .build();

        let macro_add = gtk::Button::builder()
            .icon_name("list-add-symbolic")
            .css_classes(["flat", "accent"])
            .tooltip_text("Add macro")
            .build();

        let macro_input = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(4)
            .build();
        macro_input.append(&macro_shortcut);
        macro_input.append(&gtk::Label::builder().label("→").css_classes(["dim-label"]).build());
        macro_input.append(&macro_expansion);
        macro_input.append(&macro_add);

        macros_group.add(&macros_list);
        macros_group.add(&macro_input);
        shortcuts_box.append(&macros_group);

        // ========== Reference Card ==========
        let ref_group = adw::PreferencesGroup::builder()
            .title("Quick Reference")
            .build();

        let ref_row = adw::ActionRow::builder()
            .title("Common Shortcuts")
            .subtitle("ko→không, dc→được, vs→với, lm→làm")
            .activatable(false)
            .build();

        let ref_icon = gtk::Image::builder()
            .icon_name("dialog-information-symbolic")
            .tooltip_text("Type these shortcuts followed by space")
            .build();
        ref_row.add_suffix(&ref_icon);

        ref_group.add(&ref_row);
        shortcuts_box.append(&ref_group);

        let shortcuts_clamp = adw::Clamp::builder().maximum_size(540).tightening_threshold(400).build();
        shortcuts_clamp.set_child(Some(&shortcuts_box));
        let shortcuts_scrolled = gtk::ScrolledWindow::builder()
            .vexpand(true)
            .hscrollbar_policy(gtk::PolicyType::Never)
            .child(&shortcuts_clamp)
            .build();
        stack.add_titled(&shortcuts_scrolled, Some("shortcuts"), "Shortcuts");

        // ========== Status Bar ==========
        let status_box = gtk::Box::builder()
            .orientation(gtk::Orientation::Horizontal)
            .spacing(8)
            .margin_top(8)
            .build();

        let status_icon = gtk::Image::builder()
            .icon_name("emblem-ok-symbolic")
            .build();

        let status_label = gtk::Label::builder()
            .label("Ready")
            .hexpand(true)
            .halign(gtk::Align::Start)
            .css_classes(["dim-label"])
            .build();

        status_box.append(&status_icon);
        status_box.append(&status_label);

        main_box.append(&stack);
        main_box.append(&status_box);

        toast_overlay.set_child(Some(&main_box));
        adw::prelude::AdwApplicationWindowExt::set_content(self, Some(&toast_overlay));

        // ========== Callbacks ==========

        // Mark dirty on any change
        {
            let win = self.clone();
            method_row.connect_selected_notify(move |_| { win.mark_dirty(); });
        }
        {
            let win = self.clone();
            toggle_row.connect_selected_notify(move |_| { win.mark_dirty(); });
        }
        {
            let win = self.clone();
            start_enabled_row.connect_active_notify(move |_| { win.mark_dirty(); });
        }
        {
            let win = self.clone();
            app_memory_row.connect_active_notify(move |_| { win.mark_dirty(); });
        }
        {
            let win = self.clone();
            auto_restore_row.connect_active_notify(move |_| { win.mark_dirty(); });
        }
        {
            let win = self.clone();
            autostart_row.connect_active_notify(move |_| { win.mark_dirty(); });
        }

        // Add English app
        self.setup_add_app(&english_entry, &english_add, &english_list, &status_label, &status_icon);

        // Add Vietnamese app
        self.setup_add_app(&viet_entry, &viet_add, &viet_list, &status_label, &status_icon);

        // Add macro
        self.setup_add_macro(&macro_shortcut, &macro_expansion, &macro_add, &macros_list, &status_label, &status_icon);

        // Save button
        {
            let method_row = method_row.clone();
            let toggle_row = toggle_row.clone();
            let start_switch = start_enabled_row.clone();
            let app_switch = app_memory_row.clone();
            let auto_switch = auto_restore_row.clone();
            let autostart_switch = autostart_row.clone();
            let english = english_list.clone();
            let viet = viet_list.clone();
            let macros = macros_list.clone();
            let status_label = status_label.clone();
            let status_icon = status_icon.clone();
            let toast_overlay = toast_overlay.clone();
            let win = self.clone();
            let trigger_keys = trigger_keys.clone();

            save_btn.connect_clicked(move |_| {
                let method = match method_row.selected() {
                    1 => "vni",
                    _ => "telex",
                };
                let toggle = match toggle_row.selected() {
                    1 => "shift",
                    2 => "capslock",
                    _ => "space",
                };

                let english_apps = Self::collect_app_names(&english);
                let vietnamese_apps = Self::collect_app_names(&viet);
                let macro_map = Self::collect_macros(&macros);

                let config = Config {
                    input_method: method.into(),
                    toggle_key: toggle.into(),
                    start_enabled: start_switch.is_active(),
                    auto_restore: crate::config::AutoRestoreConfig {
                        enabled: auto_switch.is_active(),
                        trigger_keys: trigger_keys.clone(),
                    },
                    app_state: crate::config::AppStateConfig {
                        enabled: app_switch.is_active(),
                        english_apps,
                        vietnamese_apps,
                    },
                    macros: macro_map,
                };

                // Save autostart state
                if autostart_switch.is_active() {
                    crate::config::install_autostart_force();
                } else {
                    crate::config::uninstall_autostart();
                }

                match config.save() {
                    Ok(()) => {
                        status_label.set_text(&format!("Saved to {}", Config::path().display()));
                        status_icon.set_icon_name(Some("emblem-ok-symbolic"));
                        status_label.remove_css_class("error");
                        status_label.add_css_class("dim-label");

                        *win.imp().dirty.borrow_mut() = false;

                        let toast = adw::Toast::new("Settings saved");
                        toast.set_timeout(2);
                        toast_overlay.add_toast(toast);
                    }
                    Err(e) => {
                        status_label.set_text(&format!("Error: {}", e));
                        status_icon.set_icon_name(Some("dialog-error-symbolic"));
                        status_label.remove_css_class("dim-label");
                        status_label.add_css_class("error");

                        let toast = adw::Toast::new(&format!("Save failed: {}", e));
                        toast.set_timeout(3);
                        toast_overlay.add_toast(toast);
                    }
                }
            });
        }
    }

    fn setup_add_app(
        &self,
        entry: &gtk::SearchEntry,
        add_btn: &gtk::Button,
        list: &gtk::ListBox,
        status_label: &gtk::Label,
        status_icon: &gtk::Image,
    ) {
        let add_fn = {
            let list = list.clone();
            let entry = entry.clone();
            let status_label = status_label.clone();
            let status_icon = status_icon.clone();
            let win = self.clone();
            move || {
                let text = entry.text().to_string();
                if !text.is_empty() {
                    let row = Self::make_app_row_static(&text, &list);
                    list.append(&row);
                    entry.set_text("");
                    status_label.set_text("Unsaved changes");
                    status_icon.set_icon_name(Some("dialog-information-symbolic"));
                    win.mark_dirty();
                }
            }
        };

        let add_fn2 = add_fn.clone();
        add_btn.connect_clicked(move |_| add_fn2());

        let add_fn3 = add_fn.clone();
        entry.connect_activate(move |_| add_fn3());
    }

    fn setup_add_macro(
        &self,
        shortcut: &gtk::SearchEntry,
        expansion: &gtk::SearchEntry,
        add_btn: &gtk::Button,
        list: &gtk::ListBox,
        status_label: &gtk::Label,
        status_icon: &gtk::Image,
    ) {
        let add_fn = {
            let list = list.clone();
            let shortcut = shortcut.clone();
            let expansion = expansion.clone();
            let status_label = status_label.clone();
            let status_icon = status_icon.clone();
            let win = self.clone();
            move || {
                let s = shortcut.text().to_string();
                let e = expansion.text().to_string();
                if !s.is_empty() && !e.is_empty() {
                    let row = Self::make_macro_row_static(&s, &e, &list);
                    list.append(&row);
                    shortcut.set_text("");
                    expansion.set_text("");
                    status_label.set_text("Unsaved changes");
                    status_icon.set_icon_name(Some("dialog-information-symbolic"));
                    win.mark_dirty();
                }
            }
        };

        let add_fn2 = add_fn.clone();
        add_btn.connect_clicked(move |_| add_fn2());

        let add_fn3 = add_fn.clone();
        expansion.connect_activate(move |_| add_fn3());
    }

    fn make_app_row_static(app: &str, list: &gtk::ListBox) -> adw::ActionRow {
        let row = adw::ActionRow::builder()
            .title(app)
            .activatable(false)
            .build();

        let remove_btn = gtk::Button::builder()
            .icon_name("user-trash-symbolic")
            .css_classes(["flat", "destructive-action"])
            .tooltip_text("Remove")
            .build();

        let list_ref = list.clone();
        let app_name = app.to_string();
        remove_btn.connect_clicked(move |_| {
            let mut i = 0;
            while let Some(child) = list_ref.row_at_index(i) {
                if let Some(row) = child.downcast_ref::<adw::ActionRow>() {
                    if row.title() == app_name {
                        list_ref.remove(&child);
                        return;
                    }
                }
                i += 1;
            }
        });

        row.add_suffix(&remove_btn);
        row
    }

    fn make_macro_row_static(shortcut: &str, expansion: &str, list: &gtk::ListBox) -> adw::ActionRow {
        let row = adw::ActionRow::builder()
            .title(shortcut)
            .subtitle(expansion)
            .activatable(false)
            .build();

        let arrow = gtk::Label::builder()
            .label("→")
            .css_classes(["dim-label"])
            .build();
        row.add_prefix(&arrow);

        let remove_btn = gtk::Button::builder()
            .icon_name("user-trash-symbolic")
            .css_classes(["flat", "destructive-action"])
            .tooltip_text("Remove")
            .build();

        let list_ref = list.clone();
        let shortcut_name = shortcut.to_string();
        remove_btn.connect_clicked(move |_| {
            let mut i = 0;
            while let Some(child) = list_ref.row_at_index(i) {
                if let Some(row) = child.downcast_ref::<adw::ActionRow>() {
                    if row.title() == shortcut_name {
                        list_ref.remove(&child);
                        return;
                    }
                }
                i += 1;
            }
        });

        row.add_suffix(&remove_btn);
        row
    }

    fn collect_app_names(list: &gtk::ListBox) -> Vec<String> {
        let mut names = Vec::new();
        let mut i = 0;
        while let Some(child) = list.row_at_index(i) {
            if let Some(row) = child.downcast_ref::<adw::ActionRow>() {
                names.push(row.title().to_string());
            }
            i += 1;
        }
        names
    }

    fn collect_macros(list: &gtk::ListBox) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        let mut i = 0;
        while let Some(child) = list.row_at_index(i) {
            if let Some(row) = child.downcast_ref::<adw::ActionRow>() {
                let shortcut = row.title().to_string();
                let expansion = row.subtitle().unwrap_or_default().to_string();
                if !shortcut.is_empty() {
                    map.insert(shortcut, expansion);
                }
            }
            i += 1;
        }
        map
    }
}
