use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use vietc_engine::{Engine, EngineEvent, EventStore, InputEvent, InputMethod};

use crate::app_state::AppStateManager;
use crate::commands::OutputCommand;
use crate::config::Config;
use crate::event::is_flush_char;
use crate::log::log_info;

pub struct Daemon {
    pub engine: Engine,
    pub config: Config,
    pub config_path: PathBuf,
    pub config_modified: std::time::SystemTime,
    pub app_state: AppStateManager,
    pub engine_enabled: Arc<AtomicBool>,
    pub grab_enabled: bool,
    pub event_store: EventStore,
    pub screen_output: String,
}

impl Daemon {
    pub fn new(config: Config, config_path: PathBuf, engine_enabled: Arc<AtomicBool>) -> Self {
        let method = match config.input_method.as_str() {
            "vni" => InputMethod::Vni,
            _ => InputMethod::Telex,
        };
        let mut engine = Engine::new(method);
        engine.set_enabled(config.start_enabled);
        engine.set_auto_restore(config.auto_restore.enabled);
        engine_enabled.store(config.start_enabled, Ordering::SeqCst);

        for (shortcut, expansion) in &config.macros {
            engine.add_macro(shortcut.clone(), expansion.clone());
        }

        let mut app_state = AppStateManager::new(
            config.app_state.english_apps.clone(),
            config.app_state.vietnamese_apps.clone(),
            config.app_state.bypass_apps.clone(),
            config.app_state.terminal_apps.clone(),
            config.app_state.terminal_input_method.clone(),
            config.input_method.clone(),
            config.start_enabled,
        );
        app_state.load_overrides();
        app_state.set_password_config(
            config.password_detection.enabled,
            config.password_detection.check_atspi2,
            config.password_detection.check_window_title,
            config.password_detection.title_keywords.clone(),
            config.password_detection.password_apps.clone(),
        );

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
            event_store: EventStore::new(),
            screen_output: String::new(),
        }
    }

    pub fn write_status(&self) {
        if let Some(parent) = self.config_path.parent() {
            let status_path = parent.join("status");
            let enabled = self.engine.is_enabled();
            self.engine_enabled.store(enabled, Ordering::SeqCst);
            let status_str = if enabled { "vn" } else { "en" };
            let _ = std::fs::write(status_path, status_str);
        }
    }

    pub fn write_method_status(&self) {
        if let Some(parent) = self.config_path.parent() {
            let method_path = parent.join("method");
            let method = &self.config.input_method;
            let _ = std::fs::write(method_path, method);
        }
    }

    pub fn toggle_method(&mut self) {
        let new_global = match self.config.input_method.as_str() {
            "vni" => "telex",
            _ => "vni",
        };
        self.config.input_method = new_global.into();
        self.app_state.set_global_method(new_global);
        let effective = self.app_state.effective_method();
        let engine_method = match effective {
            "vni" => InputMethod::Vni,
            _ => InputMethod::Telex,
        };
        self.engine.set_method(engine_method);
        self.write_method_status();
        log_info(&format!(
            "[vietc] Input method toggled: global={}, effective={}",
            self.config.input_method, effective
        ));
    }

    pub fn sync_status_file(&mut self) {
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

    pub fn reload_config(&mut self) -> bool {
        let modified = fs::metadata(&self.config_path)
            .and_then(|m| m.modified())
            .unwrap_or(std::time::SystemTime::now());

        if modified <= self.config_modified {
            return false;
        }

        match Config::load_from(&self.config_path) {
            Ok(new_config) => {
                self.engine
                    .set_auto_restore(new_config.auto_restore.enabled);

                self.engine.clear_macros();
                for (shortcut, expansion) in &new_config.macros {
                    self.engine.add_macro(shortcut.clone(), expansion.clone());
                }

                self.app_state.set_global_method(&new_config.input_method);
                self.app_state.update_lists(
                    new_config.app_state.english_apps.clone(),
                    new_config.app_state.vietnamese_apps.clone(),
                    new_config.app_state.bypass_apps.clone(),
                    new_config.app_state.terminal_apps.clone(),
                    new_config.app_state.terminal_input_method.clone(),
                );

                let effective = self.app_state.effective_method();
                let engine_method = match effective {
                    "vni" => InputMethod::Vni,
                    _ => InputMethod::Telex,
                };
                self.engine.set_method(engine_method);

                self.app_state.set_password_config(
                    new_config.password_detection.enabled,
                    new_config.password_detection.check_atspi2,
                    new_config.password_detection.check_window_title,
                    new_config.password_detection.title_keywords.clone(),
                    new_config.password_detection.password_apps.clone(),
                );

                self.grab_enabled = new_config.grab;
                self.config = new_config;
                self.config_modified = modified;
                true
            }
            Err(_) => false,
        }
    }

    pub fn process_key(&mut self, ch: char) -> Vec<OutputCommand> {
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
        }

        commands
    }

    pub fn toggle(&mut self) {
        let new_state = self.app_state.toggle_current_app();

        self.engine.set_enabled(new_state);
        self.write_status();

        if new_state {
            self.engine.reset();
        }
    }

    pub fn is_current_app_bypassed(&self) -> bool {
        if !self.config.app_state.enabled {
            return false;
        }
        self.app_state.is_current_app_bypassed()
    }

    pub fn replay_and_inject(&mut self, ch: char) -> Vec<OutputCommand> {
        let mut commands = Vec::new();
        let method = match self.config.input_method.as_str() {
            "vni" => InputMethod::Vni,
            _ => InputMethod::Telex,
        };

        if is_flush_char(ch) {
            self.event_store.push(InputEvent::Flush(ch));
            let to_commit = self.word_to_commit();
            if !self.screen_output.is_empty() && to_commit != self.screen_output {
                let backspaces = self.screen_output.chars().count();
                commands.push(OutputCommand::Backspace(backspaces));
                commands.push(OutputCommand::Type(to_commit));
            }
            commands.push(OutputCommand::Type(ch.to_string()));
            self.event_store.clear();
            self.screen_output.clear();
            return commands;
        }

        self.event_store.push(InputEvent::KeyTyped(ch));

        let (new_output, did_flush) = Engine::replay_events(
            method,
            &self.config.macros,
            &self.event_store,
        );

        if did_flush {
            let to_commit = self.word_to_commit();
            if !self.screen_output.is_empty() && to_commit != self.screen_output {
                let backspaces = self.screen_output.chars().count();
                commands.push(OutputCommand::Backspace(backspaces));
                commands.push(OutputCommand::Type(to_commit));
            }
            self.event_store.clear();
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

    pub fn replay_backspace(&mut self) -> Vec<OutputCommand> {
        let mut commands = Vec::new();
        let method = match self.config.input_method.as_str() {
            "vni" => InputMethod::Vni,
            _ => InputMethod::Telex,
        };

        if self.event_store.is_empty() {
            commands.push(OutputCommand::Backspace(1));
            return commands;
        }

        self.event_store.push(InputEvent::Backspace);

        match self.event_store.pop() {
            Some(InputEvent::Backspace) => {
                self.event_store.pop();
            }
            Some(_) => {}
            None => {}
        }

        let (new_output, _) = if self.event_store.is_empty() {
            (String::new(), false)
        } else {
            Engine::replay_events(
                method,
                &self.config.macros,
                &self.event_store,
            )
        };

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

    pub fn word_to_commit(&self) -> String {
        if self.config.auto_restore.enabled {
            let raw = self.event_store.raw_keystrokes();
            if Engine::should_restore_word(&self.screen_output, &raw) {
                return raw;
            }
        }
        self.screen_output.clone()
    }

    pub fn replay_reset(&mut self) {
        self.event_store.clear();
        self.screen_output.clear();
    }

    pub fn check_app_change_with(&mut self, new_class: String) {
        if let Some(should_enable) = self.app_state.update_with_app(new_class) {
            self.engine.set_enabled(should_enable);
            self.write_status();
        }
        let effective = self.app_state.effective_method();
        let engine_method = match effective {
            "vni" => InputMethod::Vni,
            _ => InputMethod::Telex,
        };
        self.engine.set_method(engine_method);
    }
}
