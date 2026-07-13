// SPDX-License-Identifier: MIT
//
// Aux controller mode. Bamboo (or another external IBus engine) performs the
// Vietnamese composition; vietc only watches the focused window and the
// password-field state, then switches the active IBus engine accordingly:
//
//   * app in `english_apps` / manually toggled off / password field focused
//     -> EN_ENGINE (BambooUs, no Vietnamese transformation)
//   * otherwise -> VN_ENGINE (Bamboo)
//
// vietc registers no IBus component in this mode, so the doubling bug from
// re-registration can never recur.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::app_state;
use crate::daemon::Daemon;
use crate::ibus_control;
use crate::signal;

const POLL_MS: u64 = 250;

pub fn run_controller(mut daemon: Daemon) -> Result<(), Box<dyn std::error::Error>> {
    let status_path: Option<PathBuf> = daemon
        .config_path
        .parent()
        .map(|p| p.join("status"));
    let engine_enabled = daemon.engine_enabled.clone();
    let mut last_class = String::new();
    let mut last_target = String::new();

    eprintln!(
        "[vietc] Aux controller running (VN='{}', EN='{}')",
        ibus_control::VN_ENGINE,
        ibus_control::EN_ENGINE
    );

    loop {
        if signal::SIGNAL_EXIT.load(Ordering::SeqCst) {
            eprintln!("[vietc] received stop signal, exiting controller");
            break Ok(());
        }

        // Track focused app; only re-evaluate app state on change.
        //
        // When the window class can't be identified (GNOME Shell `Eval` is
        // unavailable on this session and the focused app is Wayland-native,
        // so the X11 fallbacks can't see it), we deliberately LEAVE the IBus
        // engine untouched (`None` target). This lets the per-app engine the
        // user assigned through the IBus/language indicator stick — e.g.
        // ptyxis -> BambooUs (English, no garbling) while firefox/gedit keep
        // Bamboo (Vietnamese). vietc only drives the engine for the apps it
        // *can* see (XWayland: VS Code, X11 terminals).
        let class = app_state::get_focused_window_class();
        let target: Option<&str> = match &class {
            Some(c) => {
                if *c != last_class {
                    last_class = c.clone();
                    daemon.app_state.update_with_app(c.clone());
                }
                let mut vn = daemon.app_state.get_default_state();
                // Terminals: force English. Bamboo's no-underline (Surrounding
                // Text) mode can't do in-place editing inside a terminal, so it
                // garbles Vietnamese there; terminals are also predominantly
                // used for commands.
                if vn && daemon.app_state.is_terminal_app() {
                    vn = false;
                }
                // Only run (potentially expensive) password detection when
                // Vietnamese would otherwise be on.
                let vn = if vn {
                    !daemon.app_state.check_password_field()
                } else {
                    false
                };
                Some(if vn {
                    ibus_control::VN_ENGINE
                } else {
                    ibus_control::EN_ENGINE
                })
            }
            None => None,
        };

        if let Some(target) = target {
            if target != last_target {
                last_target = target.to_string();
                ibus_control::set_ibus_engine(target);
                if let Some(ref sp) = status_path {
                    let _ = std::fs::write(
                        sp,
                        if target == ibus_control::VN_ENGINE {
                            "vn"
                        } else {
                            "en"
                        },
                    );
                }
                engine_enabled.store(target == ibus_control::VN_ENGINE, Ordering::SeqCst);
            }
        }

        thread::sleep(Duration::from_millis(POLL_MS));
    }
}
