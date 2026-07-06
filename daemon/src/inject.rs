use vietc_protocol::KeyInjector;

use crate::commands::OutputCommand;
use crate::display;
use crate::log::log_info;

pub fn execute_commands(
    injector: &dyn KeyInjector,
    commands: &[OutputCommand],
    grabbed: bool,
) {
    let mut pending_backspaces: usize = 0;
    let mut pending_text = String::new();

    for cmd in commands {
        match cmd {
            OutputCommand::Backspace(count) => {
                let adjusted = if grabbed {
                    count.saturating_sub(1)
                } else {
                    *count
                };
                pending_backspaces += adjusted;
            }
            OutputCommand::Type(text) => {
                pending_text.push_str(text);
            }
        }
    }

    if pending_backspaces > 0 || !pending_text.is_empty() {
        let _ = injector.inject_replacement(pending_backspaces, &pending_text);
    } else if !commands.is_empty() {
        let _ = injector.inject_replacement(pending_backspaces, &pending_text);
    }

    injector.flush();

    if !commands.is_empty() {
        std::thread::sleep(std::time::Duration::from_millis(20));
    }
}

pub fn create_injector(
    display: display::DisplayServer,
) -> Result<Box<dyn KeyInjector>, Box<dyn std::error::Error>> {
    match vietc_protocol::uinput_monitor::UinputInjector::new("vietc") {
        Ok(injector) => {
            log_info("[vietc] Using uinput injection");
            return Ok(Box::new(injector));
        }
        Err(e) => {
            log_info(&format!("[vietc] uinput not available: {}", e));
        }
    }

    if vietc_protocol::uinput_client::UinputClient::is_available() {
        log_info("[vietc] Using uinputd socket injection");
        return Ok(Box::new(vietc_protocol::uinput_client::UinputClient));
    }

    #[cfg(feature = "x11")]
    if display != display::DisplayServer::Wayland {
        match vietc_protocol::x11_inject::X11Injector::new() {
            Ok(injector) => {
                log_info("[vietc] Using X11 injection (fallback)");
                return Ok(Box::new(injector));
            }
            Err(e) => {
                log_info(&format!("[vietc] X11 not available: {}", e));
            }
        }
    }

    Err("No injection backend available".into())
}
