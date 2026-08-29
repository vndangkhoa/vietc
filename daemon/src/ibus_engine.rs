// SPDX-License-Identifier: MIT
//
// vietc as a native IBus engine.

// IBus routes keystrokes from any focused client (X11/XWayland *and* native
// Wayland GNOME apps) to an engine over D-Bus. An engine serves a
// `org.freedesktop.IBus.Factory` object whose `CreateEngine(s) -> (o)` returns
// the path of a `org.freedesktop.IBus.Engine` object. We register ourselves
// with the running ibus-daemon via `RegisterComponent(v)`, make the engine the
// global engine via `SetGlobalEngine(s)`, and then handle `ProcessKeyEvent`.
//
// We reuse the existing vietc `Engine` (VNI/Telex) 1:1: each forwarded key
// goes through `Engine::process_key`, and the returned `EngineEvent` is turned
// into IBus `CommitText` / `DeleteSurroundingText` signals. This mirrors the
// daemon's existing "type then correct in place" behaviour but works for every
// app because IBus is the compositor-approved input method.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use dbus::arg::messageitem::{MessageItem, MessageItemArray, MessageItemDict};
use dbus::ffidisp::{Connection, MsgHandler, MsgHandlerResult, MsgHandlerType};
use dbus::strings::{BusName, ErrorName, Interface, Member};
use dbus::{Message, MessageType, Path, Signature};

use vietc_engine::{Engine, EngineEvent, InputMethod};

use crate::im_plan::is_flush_char;

const IBUS_BUS_NAME: &str = "org.freedesktop.IBus";
const IBUS_BUS_PATH: &str = "/org/freedesktop/IBus";
const IBUS_INTERFACE: &str = "org.freedesktop.IBus";
const FACTORY_PATH: &str = "/org/freedesktop/IBus/Factory";
const FACTORY_INTERFACE: &str = "org.freedesktop.IBus.Factory";
const ENGINE_INTERFACE: &str = "org.freedesktop.IBus.Engine";
const ENGINE_NAME: &str = "vietc";

const FACTORY_INTROSPECTION: &str = r#"<node>
  <interface name='org.freedesktop.IBus.Factory'>
    <method name='CreateEngine'>
      <arg direction='in' type='s' name='name' />
      <arg direction='out' type='o' />
    </method>
  </interface>
  <interface name='org.freedesktop.DBus.Introspectable'>
    <method name='Introspect'>
      <arg direction='out' type='s' />
    </method>
  </interface>
  <interface name='org.freedesktop.DBus.Properties'>
    <method name='Get'>
      <arg direction='in' type='s' name='interface_name' />
      <arg direction='in' type='s' name='property_name' />
      <arg direction='out' type='v' />
    </method>
    <method name='GetAll'>
      <arg direction='in' type='s' name='interface_name' />
      <arg direction='out' type='a{sv}' />
    </method>
    <method name='Set'>
      <arg direction='in' type='s' name='interface_name' />
      <arg direction='in' type='s' name='property_name' />
      <arg direction='in' type='v' name='value' />
    </method>
  </interface>
</node>"#;

const ENGINE_INTROSPECTION: &str = r#"<node>
  <interface name='org.freedesktop.IBus.Engine'>
    <method name='ProcessKeyEvent'>
      <arg direction='in' type='u' name='keyval' />
      <arg direction='in' type='u' name='keycode' />
      <arg direction='in' type='u' name='state' />
      <arg direction='out' type='b' />
    </method>
    <method name='SetCursorLocation'>
      <arg direction='in' type='i' name='x' />
      <arg direction='in' type='i' name='y' />
      <arg direction='in' type='i' name='w' />
      <arg direction='in' type='i' name='h' />
    </method>
    <method name='SetCapabilities'>
      <arg direction='in' type='u' name='caps' />
    </method>
    <method name='PropertyActivate'>
      <arg direction='in' type='s' name='name' />
      <arg direction='in' type='u' name='state' />
    </method>
    <method name='PropertyShow'>
      <arg direction='in' type='s' name='name' />
    </method>
    <method name='PropertyHide'>
      <arg direction='in' type='s' name='name' />
    </method>
    <method name='CandidateClicked'>
      <arg direction='in' type='u' name='index' />
      <arg direction='in' type='u' name='button' />
      <arg direction='in' type='u' name='state' />
    </method>
    <method name='FocusIn' />
    <method name='FocusInId'>
      <arg direction='in' type='s' name='object_path' />
      <arg direction='in' type='s' name='client' />
    </method>
    <method name='FocusOut' />
    <method name='FocusOutId'>
      <arg direction='in' type='s' name='object_path' />
    </method>
    <method name='Reset' />
    <method name='Enable' />
    <method name='Disable' />
    <method name='PageUp' />
    <method name='PageDown' />
    <method name='CursorUp' />
    <method name='CursorDown' />
    <method name='SetSurroundingText'>
      <arg direction='in' type='v' name='text' />
      <arg direction='in' type='u' name='cursor_pos' />
      <arg direction='in' type='u' name='anchor_pos' />
    </method>
    <signal name='CommitText'>
      <arg type='v' name='text' />
    </signal>
    <signal name='UpdatePreeditText'>
      <arg type='v' name='text' />
      <arg type='u' name='cursor_pos' />
      <arg type='b' name='visible' />
      <arg type='u' name='mode' />
    </signal>
    <signal name='UpdateAuxiliaryText'>
      <arg type='v' name='text' />
      <arg type='b' name='visible' />
    </signal>
    <signal name='UpdateLookupTable'>
      <arg type='v' name='table' />
      <arg type='b' name='visible' />
    </signal>
    <signal name='RegisterProperties'>
      <arg type='v' name='props' />
    </signal>
    <signal name='UpdateProperty'>
      <arg type='v' name='prop' />
    </signal>
    <signal name='ForwardKeyEvent'>
      <arg type='u' name='keyval' />
      <arg type='u' name='keycode' />
      <arg type='u' name='state' />
    </signal>
    <signal name='DeleteSurroundingText'>
      <arg type='i' name='offset' />
      <arg type='u' name='nchars' />
    </signal>
    <property name='ContentType' type='(uu)' access='write' />
    <property name='FocusId' type='(b)' access='read' />
    <property name='ActiveSurroundingText' type='(b)' access='read' />
  </interface>
  <interface name='org.freedesktop.DBus.Introspectable'>
    <method name='Introspect'>
      <arg direction='out' type='s' />
    </method>
  </interface>
  <interface name='org.freedesktop.DBus.Properties'>
    <method name='Get'>
      <arg direction='in' type='s' name='interface_name' />
      <arg direction='in' type='s' name='property_name' />
      <arg direction='out' type='v' />
    </method>
    <method name='GetAll'>
      <arg direction='in' type='s' name='interface_name' />
      <arg direction='out' type='a{sv}' />
    </method>
    <method name='Set'>
      <arg direction='in' type='s' name='interface_name' />
      <arg direction='in' type='s' name='property_name' />
      <arg direction='in' type='v' name='value' />
    </method>
  </interface>
</node>"#;

struct EngineContext {
    engine: Engine,
    enabled: bool,
    focus_id: bool,
    preedit: String,
    dedup: crate::key_dedup::DedupState,
}

struct IBusState {
    contexts: Mutex<HashMap<String, EngineContext>>,
    counter: Mutex<u32>,
    method: Mutex<InputMethod>,
    engine_enabled: Arc<AtomicBool>,
    auto_restore: bool,
    deduplicate: bool,
    dedup_two_back: bool,
    dedup_window_ms: u64,
    aux_popup: Mutex<Option<(String, std::time::Instant)>>,
}

/// Maps an IBus keyval to a Unicode char we can feed the vietc engine.
/// Returns `None` for function keys (BackSpace is handled specially by the
/// caller). IBus keyvals for printable characters equal their Unicode
/// codepoint.
fn keyval_to_char(keyval: u32) -> Option<char> {
    if keyval == 0xFF08 {
        // BackSpace
        return Some('\x08');
    }
    if keyval >= 0x20 && keyval <= 0x10FFFF {
        if keyval >= 0xFF00 && keyval <= 0xFFFF {
            return None; // XKB / function keys
        }
        char::from_u32(keyval)
    } else {
        None
    }
}

fn sig(s: &str) -> Signature<'static> {
    Signature::new(s).unwrap()
}

fn empty_dict() -> MessageItem {
    MessageItem::Dict(MessageItemDict::new(vec![], sig("s"), sig("v")).unwrap())
}

fn ibus_attr_no_underline(start: u32, end: u32) -> MessageItem {
    // IBusAttribute serializes as (s a{sv} uuuu)
    MessageItem::Struct(vec![
        MessageItem::Str("IBusAttribute".into()),
        empty_dict(),
        MessageItem::UInt32(1), // IBUS_ATTR_TYPE_UNDERLINE
        MessageItem::UInt32(0), // IBUS_ATTR_UNDERLINE_NONE
        MessageItem::UInt32(start),
        MessageItem::UInt32(end),
    ])
}

fn ibus_attr_list(char_count: usize) -> MessageItem {
    // IBusAttrList serializes as (s a{sv} av)
    let attrs = if char_count > 0 {
        vec![MessageItem::Variant(Box::new(ibus_attr_no_underline(
            0,
            char_count as u32,
        )))]
    } else {
        vec![]
    };
    MessageItem::Struct(vec![
        MessageItem::Str("IBusAttrList".into()),
        empty_dict(),
        MessageItem::Array(MessageItemArray::new(attrs, sig("av")).unwrap()),
    ])
}

fn ibus_text_struct(text: &str) -> MessageItem {
    // IBusText serializes as (s a{sv} s v)
    let char_count = text.chars().count();
    MessageItem::Struct(vec![
        MessageItem::Str("IBusText".into()),
        empty_dict(),
        MessageItem::Str(text.into()),
        MessageItem::Variant(Box::new(ibus_attr_list(char_count))),
    ])
}

fn ibus_text_variant(text: &str) -> MessageItem {
    MessageItem::Variant(Box::new(ibus_text_struct(text)))
}

fn engine_desc_struct() -> MessageItem {
    // IBusEngineDesc serializes as (s a{sv} sssssssussssssss)
    MessageItem::Struct(vec![
        MessageItem::Str("IBusEngineDesc".into()),
        empty_dict(),
        MessageItem::Str(ENGINE_NAME.into()),                  // name
        MessageItem::Str("Viet+ (vietc)".into()),             // longname
        MessageItem::Str("Vietnamese input method (vietc)".into()), // description
        MessageItem::Str("vi".into()),                        // language
        MessageItem::Str("MIT".into()),                       // license
        MessageItem::Str("vietc".into()),                     // author
        MessageItem::Str("".into()),                          // icon
        MessageItem::Str("us".into()),                        // layout
        MessageItem::UInt32(99),                              // rank
        MessageItem::Str("".into()),                          // hotkeys
        MessageItem::Str("V".into()),                         // symbol
        MessageItem::Str("".into()),                          // setup
        MessageItem::Str("".into()),                          // layout_variant
        MessageItem::Str("".into()),                          // layout_option
        MessageItem::Str(env!("CARGO_PKG_VERSION").into()),   // version
        MessageItem::Str("vietc".into()),                     // textdomain
        MessageItem::Str("".into()),                          // icon_prop_key
    ])
}

fn component_variant() -> MessageItem {
    // IBusComponent serializes as (s a{sv} ssssssss av av)
    let engines = MessageItem::Array(
        MessageItemArray::new(
            vec![MessageItem::Variant(Box::new(engine_desc_struct()))],
            sig("av"),
        )
        .unwrap(),
    );
    let observed = MessageItem::Array(MessageItemArray::new(vec![], sig("av")).unwrap());
    let component = MessageItem::Struct(vec![
        MessageItem::Str("IBusComponent".into()),
        empty_dict(),
        MessageItem::Str("vietc".into()),                    // name
        MessageItem::Str("Viet+ input method".into()),       // description
        MessageItem::Str(env!("CARGO_PKG_VERSION").into()),  // version
        MessageItem::Str("MIT".into()),                      // license
        MessageItem::Str("vietc".into()),                    // author
        MessageItem::Str("".into()),                         // homepage
        MessageItem::Str("".into()),                         // exec
        MessageItem::Str("vietc".into()),                    // textdomain
        observed,
        engines,
    ]);
    MessageItem::Variant(Box::new(component))
}

fn focus_id_variant(focused: bool) -> MessageItem {
    MessageItem::Variant(Box::new(MessageItem::Struct(vec![MessageItem::Bool(
        focused,
    )])))
}

struct IBusHandler {
    state: Arc<IBusState>,
    conn: Arc<Connection>,
}

impl MsgHandler for IBusHandler {
    fn handler_type(&self) -> MsgHandlerType {
        MsgHandlerType::MsgType(MessageType::MethodCall)
    }

    fn handle_msg(&mut self, msg: &Message) -> Option<MsgHandlerResult> {
        handle_message(&self.state, &self.conn, msg);
        Some(MsgHandlerResult {
            handled: true,
            done: false,
            reply: vec![],
        })
    }
}

fn ibus_bus_dir() -> Option<PathBuf> {
    let base = if let Ok(x) = std::env::var("XDG_CONFIG_HOME") {
        if x.is_empty() {
            return None;
        }
        PathBuf::from(x)
    } else {
        let home = std::env::var("HOME").ok()?;
        PathBuf::from(home).join(".config")
    };
    Some(base.join("ibus").join("bus"))
}

/// IBus serves clients over its own private D-Bus bus whose address is written
/// to `~/.config/ibus/bus/*.conf`. The session-bus name `org.freedesktop.IBus`
/// is only a placeholder owned by the daemon and exports no objects, so we must
/// connect via this private address instead.
fn ibus_private_address() -> Option<String> {
    if let Ok(a) = std::env::var("IBUS_ADDRESS") {
        let a = a.trim();
        if !a.is_empty() {
            return Some(a.to_string());
        }
    }
    let dir = ibus_bus_dir()?;
    let entries = std::fs::read_dir(&dir).ok()?;
    for e in entries.flatten() {
        if let Ok(content) = std::fs::read_to_string(e.path()) {
            for line in content.lines() {
                let line = line.trim();
                if let Some(addr) = line.strip_prefix("IBUS_ADDRESS=") {
                    let addr = addr.trim();
                    if !addr.is_empty() {
                        return Some(addr.to_string());
                    }
                }
            }
        }
    }
    None
}

/// True if the IBus daemon described by the bus-dir address file is alive.
fn ibus_daemon_alive() -> bool {
    let dir = match ibus_bus_dir() {
        Some(d) => d,
        None => return false,
    };
    let entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(_) => return false,
    };
    for e in entries.flatten() {
        if let Ok(content) = std::fs::read_to_string(e.path()) {
            for line in content.lines() {
                if let Some(pid_s) = line.trim().strip_prefix("IBUS_DAEMON_PID=") {
                    let pid_s = pid_s.trim();
                    if std::process::Command::new("kill")
                        .args(["-0", pid_s])
                        .status()
                        .map(|s| s.success())
                        .unwrap_or(false)
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Ensure an IBus daemon is running and return its private D-Bus address.
fn ensure_ibus_running() -> Option<String> {
    if let Some(addr) = ibus_private_address() {
        if ibus_daemon_alive() {
            return Some(addr);
        }
    }
    // Start the SYSTEM ibus-daemon. Use the absolute path: a VS Code snap ships
    // its own (broken) ibus-daemon that shadows the bare "ibus-daemon" name in
    // some $PATH layouts, so never rely on PATH resolution here.
    let _ = std::process::Command::new("/usr/bin/ibus-daemon")
        .args(["-d", "--desktop=gnome"])
        // Clear snap/GTK env that makes ibus' GTK helper processes
        // (ibus-ui-gtk3, ibus-extension-gtk3) load snap's incompatible
        // libpthread and crash before the daemon exports its objects.
        .env_remove("GTK_PATH")
        .env_remove("GTK_EXE_PREFIX")
        .env_remove("GDK_PIXBUF_MODULEDIR")
        .env_remove("GDK_PIXBUF_MODULE_FILE")
        .env_remove("GTK_IM_MODULE_FILE")
        .env_remove("GI_TYPELIB_PATH")
        .env_remove("LD_LIBRARY_PATH")
        .env_remove("GTK_MODULES")
        .status();
    for _ in 0..50 {
        if let Some(addr) = ibus_private_address() {
            if ibus_daemon_alive() {
                return Some(addr);
            }
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    None
}

fn register_with_ibus(conn: &Connection) {
    let bus = BusName::new(IBUS_BUS_NAME).unwrap();
    let path = Path::new(IBUS_BUS_PATH).unwrap();
    let iface = Interface::new(IBUS_INTERFACE).unwrap();

    let reg = Message::method_call(&bus, &path, &iface, &Member::new("RegisterComponent").unwrap());
    let reg = reg.append1(component_variant());
    match conn.send(reg) {
        Ok(_) => crate::log::log_info("[vietc-ibus] sent RegisterComponent to ibus-daemon"),
        Err(_) => crate::log::log_info("[vietc-ibus] failed to send RegisterComponent"),
    }

    send_set_global(conn);
}

/// Ask the daemon to make vietc the global engine. This is non-blocking: the
/// daemon will synchronously call back our Factory's `CreateEngine` (served by
/// the running `iter` loop) to instantiate the engine, so we must not block
/// waiting for the reply on this same thread.
fn send_set_global(conn: &Connection) {
    let bus = BusName::new(IBUS_BUS_NAME).unwrap();
    let path = Path::new(IBUS_BUS_PATH).unwrap();
    let iface = Interface::new(IBUS_INTERFACE).unwrap();
    let m = Message::method_call(&bus, &path, &iface, &Member::new("SetGlobalEngine").unwrap());
    let m = m.append1(ENGINE_NAME);
    match conn.send(m) {
        Ok(_) => crate::log::log_info("[vietc-ibus] sent SetGlobalEngine(vietc)"),
        Err(_) => crate::log::log_info("[vietc-ibus] failed to send SetGlobalEngine"),
    }
}

/// Run the vietc IBus engine. Blocks while the connection is alive.
pub fn run_ibus_engine(
    method: InputMethod,
    engine_enabled: Arc<AtomicBool>,
    auto_restore: bool,
    deduplicate: bool,
    dedup_two_back: bool,
    dedup_window_ms: u64,
) -> Result<(), Box<dyn std::error::Error>> {
    let address = match ensure_ibus_running() {
        Some(a) => a,
        None => {
            crate::log::log_info(
                "[vietc-ibus] could not start/find ibus-daemon; IBus engine disabled",
            );
            return Ok(());
        }
    };

    let conn = Arc::new(Connection::open_private(&address)?);
    // Register (Hello) so the daemon learns our unique connection name and can
    // address our Factory object when a client requests the vietc engine.
    if let Err(e) = conn.register() {
        crate::log::log_info(&format!("[vietc-ibus] failed to register on ibus bus: {}", e));
        return Ok(());
    }

    let state = Arc::new(IBusState {
        contexts: Mutex::new(HashMap::new()),
        counter: Mutex::new(0),
        method: Mutex::new(method),
        engine_enabled,
        auto_restore,
        deduplicate,
        dedup_two_back,
        dedup_window_ms,
        aux_popup: Mutex::new(None),
    });

    // The Factory/Engine handler MUST be installed before we register, because
    // SetGlobalEngine makes the daemon synchronously call our Factory's
    // CreateEngine. With no handler running yet, that call would go unanswered.
    conn.add_handler(IBusHandler {
        state: state.clone(),
        conn: conn.clone(),
    });
    // Mark the Factory path as existing so external/gdbus proxies can
    // introspect it (without this, calls like CreateEngine fail with
    // "object does not exist" because the daemon's proxy cannot cache the
    // interface). Actual replies are still sent by the handler above.
    if let Err(e) = conn.register_object_path(FACTORY_PATH) {
        crate::log::log_info(&format!(
            "[vietc-ibus] register_object_path factory failed: {}",
            e
        ));
    }

    // Register non-blocking (the iter loop below answers the daemon's
    // CreateEngine callback). The daemon may not be ready to instantiate the
    // engine on the very first attempt, so we re-send SetGlobalEngine a few
    // times from the dispatch loop until it sticks.
    register_with_ibus(&conn);

    crate::log::log_info("[vietc-ibus] engine running; dispatching D-Bus messages");
    let mut setglobal_tries = 0u32;
    loop {
        if !conn.is_connected() {
            crate::log::log_info("[vietc-ibus] disconnected from ibus-daemon");
            break Ok(());
        }
        for _ in conn.iter(50) {}

        // Auto-hide auxiliary text badge after timeout (500ms)
        let to_hide = {
            let mut guard = state.aux_popup.lock().unwrap();
            if let Some((ref p, expiry)) = *guard {
                if std::time::Instant::now() >= expiry {
                    let path = p.clone();
                    *guard = None;
                    Some(path)
                } else {
                    None
                }
            } else {
                None
            }
        };
        if let Some(path) = to_hide {
            emit_update_auxiliary_text(&conn, &path, "", false);
        }

        setglobal_tries = setglobal_tries.wrapping_add(1);
        // Keep re-asserting the global engine every ~5s indefinitely.
        if setglobal_tries % 100 == 0 {
            send_set_global(&conn);
        }
    }
}

fn handle_message(state: &Arc<IBusState>, conn: &Connection, msg: &Message) {
    if msg.msg_type() != MessageType::MethodCall {
        return;
    }
    let path_s = msg.path().map(|p| p.to_string()).unwrap_or_default();
    let path = path_s.as_str();
    let iface_s = msg.interface().map(|p| p.to_string()).unwrap_or_default();
    let iface = iface_s.as_str();
    let member_s = msg.member().map(|p| p.to_string()).unwrap_or_default();
    let member = member_s.as_str();

    // Introspection: the IBus daemon (and GTK clients) use GDBus proxies that
    // introspect our objects before invoking methods, so we must answer it or
    // they report "method does not exist".
    if iface == "org.freedesktop.DBus.Introspectable" && member == "Introspect" {
        let xml = if path == FACTORY_PATH {
            FACTORY_INTROSPECTION
        } else {
            ENGINE_INTROSPECTION
        };
        let reply = msg.method_return().append1(xml);
        let _ = conn.send(reply);
        return;
    }

    // Properties are handled further below (see all_properties/focus_id_variant).

    // Factory: create a new engine instance on demand.
    if path == FACTORY_PATH && iface == FACTORY_INTERFACE && member == "CreateEngine" {
        let name: String = msg.read1().unwrap_or_default();
        let engine_path = create_engine(state, conn, &name);
        let reply = msg.method_return().append1(Path::new(engine_path.as_str()).unwrap());
        let _ = conn.send(reply);
        return;
    }

    // Engine objects.
    if path.starts_with("/org/freedesktop/IBus/Engine/") && iface == ENGINE_INTERFACE {
        match member {
            "ProcessKeyEvent" => {
                let (keyval, keycode, keystate): (u32, u32, u32) =
                    msg.read3().unwrap_or((0, 0, 0));
                let consumed =
                    handle_process_key_event(state, path, conn, keyval, keycode, keystate);
                let mut reply = msg.method_return();
                let reply = reply.append1(consumed);
                let _ = conn.send(reply);
            }
            "FocusIn" => {
                set_focus(state, path, true);
                reply_empty(msg, conn);
            }
            "FocusOut" => {
                set_focus(state, path, false);
                reply_empty(msg, conn);
            }
            "Reset" => {
                reset_context(state, path);
                reply_empty(msg, conn);
            }
            "Enable" => {
                set_enabled(state, path, true);
                reply_empty(msg, conn);
            }
            "Disable" => {
                set_enabled(state, path, false);
                reply_empty(msg, conn);
            }
            _ => reply_empty(msg, conn),
        }
        return;
    }

    // Introspectable.
    if iface == "org.freedesktop.DBus.Introspectable" && member == "Introspect" {
        let xml = if path == FACTORY_PATH {
            FACTORY_INTROSPECTION
        } else {
            ENGINE_INTROSPECTION
        };
        let reply = msg.method_return().append1(xml);
        let _ = conn.send(reply);
        return;
    }

    // Properties.
    if iface == "org.freedesktop.DBus.Properties" {
        match member {
            "GetAll" => {
                let reply = msg.method_return().append1(all_properties(false));
                let _ = conn.send(reply);
            }
            "Get" => {
                let (_iface, prop): (String, String) = msg.read2().unwrap_or_default();
                let value = match prop.as_str() {
                    "FocusId" => focus_id_variant(false),
                    "ActiveSurroundingText" => focus_id_variant(false),
                    "ContentType" => MessageItem::Variant(Box::new(MessageItem::Struct(vec![
                        MessageItem::UInt32(0),
                        MessageItem::UInt32(0),
                    ]))),
                    _ => {
                        error_reply(conn, msg, "org.freedesktop.DBus.Error.InvalidArgs", "Unknown property");
                        return;
                    }
                };
                let reply = msg.method_return().append1(value);
                let _ = conn.send(reply);
            }
            "Set" => {
                reply_empty(msg, conn);
            }
            _ => reply_empty(msg, conn),
        }
        return;
    }

    // Anything else: answer so the caller doesn't hang.
    error_reply(
        conn,
        msg,
        "org.freedesktop.DBus.Error.UnknownMethod",
        "No such method",
    );
}

fn reply_empty(msg: &Message, conn: &Connection) {
    let reply = msg.method_return();
    let _ = conn.send(reply);
}

fn error_reply(conn: &Connection, msg: &Message, name: &str, text: &str) {
    match ErrorName::new(name) {
        Ok(en) => {
            let cmsg = std::ffi::CString::new(text).unwrap_or_default();
            let e = msg.error(&en, &cmsg);
            let _ = conn.send(e);
        }
        Err(_) => reply_empty(msg, conn),
    }
}

fn create_engine(state: &IBusState, conn: &Connection, _name: &str) -> String {
    let mut counter = state.counter.lock().unwrap();
    *counter += 1;
    let path = format!("/org/freedesktop/IBus/Engine/{}", *counter);
    let method = *state.method.lock().unwrap();
    let mut engine = Engine::new(method);
    engine.set_auto_restore(state.auto_restore);
    let ctx = EngineContext {
        engine,
        enabled: true,
        focus_id: false,
        preedit: String::new(),
        dedup: crate::key_dedup::DedupState::new(),
    };
    state.contexts.lock().unwrap().insert(path.clone(), ctx);
    // Register the path so it genuinely exists on the bus (enables introspection
    // for the daemon's engine proxy). The actual handling is done by the
    // connection-level handler installed via `add_handler`.
    if let Err(e) = conn.register_object_path(&path) {
        crate::log::log_info(&format!(
            "[vietc-ibus] register_object_path {} failed: {}",
            path, e
        ));
    }
    crate::log::log_info(&format!("[vietc-ibus] created engine instance {}", path));
    path
}

fn set_focus(state: &IBusState, path: &str, focused: bool) {
    if let Some(ctx) = state.contexts.lock().unwrap().get_mut(path) {
        ctx.focus_id = focused;
        ctx.engine.reset();
        ctx.preedit.clear();
    }
}

fn set_enabled(state: &IBusState, path: &str, enabled: bool) {
    if let Some(ctx) = state.contexts.lock().unwrap().get_mut(path) {
        ctx.enabled = enabled;
        ctx.engine.reset();
        ctx.preedit.clear();
    }
}

fn reset_context(state: &IBusState, path: &str) {
    if let Some(ctx) = state.contexts.lock().unwrap().get_mut(path) {
        ctx.engine.reset();
        ctx.preedit.clear();
    }
}

fn handle_process_key_event(
    state: &Arc<IBusState>,
    path: &str,
    conn: &Connection,
    keyval: u32,
    _keycode: u32,
    state_flags: u32,
) -> bool {
    // Ignore key release events (bit 30 in IBus modifier mask: 1 << 30 = 0x40000000).
    // IBus invokes ProcessKeyEvent for both key-press and key-release.
    // Without this check, every key is processed twice (once on press, once on release).
    const IBUS_RELEASE_MASK: u32 = 1 << 30;
    if (state_flags & IBUS_RELEASE_MASK) != 0 {
        return false;
    }

    // IBus GDK mask: Shift=1, Ctrl=4, Alt=8, Super=64.
    const CTRL: u32 = 4;
    const ALT: u32 = 8;
    const SUPER: u32 = 64;

    // Ctrl+Shift cycles ENG -> VNI -> TELEX -> ENG.
    // IBus delivers Shift_L (0xFFE1) / Shift_R (0xFFE2) with Ctrl held.
    let is_shift_key = keyval == 0xFFE1 || keyval == 0xFFE2 || keyval == 65505 || keyval == 65506;
    if is_shift_key && (state_flags & CTRL) != 0 {
        let is_enabled = state.engine_enabled.load(Ordering::SeqCst);
        let mut method_guard = state.method.lock().unwrap();

        let (new_enabled, new_method, label) = if !is_enabled {
            (true, InputMethod::Vni, "VNI")
        } else if *method_guard == InputMethod::Vni {
            (true, InputMethod::Telex, "TELEX")
        } else {
            (false, *method_guard, "ENG")
        };

        state.engine_enabled.store(new_enabled, Ordering::SeqCst);
        *method_guard = new_method;

        // Update all existing engine instances
        let mut contexts = state.contexts.lock().unwrap();
        for (ctx_path, ctx) in contexts.iter_mut() {
            ctx.engine.set_method(new_method);
            ctx.engine.reset();
            if !ctx.preedit.is_empty() {
                ctx.preedit.clear();
                emit_update_preedit(conn, ctx_path, "");
            }
        }
        emit_update_preedit(conn, path, "");

        if let Some(dir) = dirs::config_dir() {
            let _ = std::fs::write(
                dir.join("vietc").join("status"),
                if new_enabled { "vn" } else { "en" },
            );
            if new_enabled {
                let method_str = match new_method {
                    InputMethod::Telex => "telex",
                    InputMethod::Vni => "vni",
                };
                let _ = std::fs::write(dir.join("vietc").join("method"), method_str);
            }
        }

        crate::log::log_info(&format!("[vietc-ibus] Ctrl+Shift cycle mode -> {}", label));
        return true;
    }

    // Master toggle off -> let every key pass through untouched.
    if !state.engine_enabled.load(Ordering::SeqCst) {
        return false;
    }
    // Don't compose while Alt/Super is held, or Ctrl with a non-toggle key.
    // Shift alone is NOT blocked (needed for uppercase and Telex `w`). The
    // Ctrl+Space / Ctrl+Shift toggles above already returned, so they are not
    // affected. Any other Ctrl combo (Ctrl+C, Ctrl+V, etc.) is forwarded.
    if state_flags & (ALT | SUPER) != 0 {
        return false;
    }
    if (state_flags & CTRL) != 0 && !is_shift_key {
        return false;
    }
    let mut guard = state.contexts.lock().unwrap();
    let ctx = match guard.get_mut(path) {
        Some(c) => c,
        None => return false,
    };
    if !ctx.enabled {
        return false;
    }

    // Workaround for a stuck/auto-repeating keyboard that emits every keystroke
    // twice. Drop a keyval that repeats the previous one (vv -> v, oo -> o, 44
    // -> 4, or a run of spaces from a stuck spacebar). Only printable
    // letters/digits and spaces are considered; Backspace/arrows/modifiers break
    // the chain so it can't span word or edit boundaries. Safe for Vietnamese.
        if state.deduplicate {
            let now = std::time::Instant::now();
            let ch = keyval_to_char(keyval);
            let dedupable = ch.map_or(false, |c| c.is_alphanumeric() || c == ' ');
            let is_space = ch == Some(' ');
            if ctx
                .dedup
                .observe(keyval, dedupable, is_space, state.dedup_two_back, state.dedup_window_ms, now)
            {
                return true;
            }
        }

    // Enter / Return: reset engine buffer and let key produce a newline.
    if keyval == 0xFF0D || keyval == 0xFF8D {
        ctx.engine.reset();
        return false;
    }

    // Escape: reset engine buffer and pass Escape through
    if keyval == 0xFF1B {
        ctx.engine.reset();
        return false;
    }

    // Navigation keys (Arrows, Home, End, PageUp/Down, Delete, Tab):
    // reset engine buffer so cursor moves freely in the document.
    let is_nav = (keyval >= 0xFF50 && keyval <= 0xFF58) || keyval == 0xFFFF || keyval == 0xFF09;
    if is_nav {
        ctx.engine.reset();
        return false;
    }

    // Backspace: update engine internal buffer and let application delete character directly.
    if keyval == 0xFF08 {
        ctx.engine.process_key('\x08');
        return false;
    }

    let ch = match keyval_to_char(keyval) {
        Some(c) => c,
        None => return false,
    };

    // Direct Commit model: text is committed directly to the focused client as real
    // characters without preedit underline. When tone/mark transformations occur,
    // DeleteSurroundingText replaces previous characters seamlessly.
    if is_flush_char(ch) {
        if let Some(EngineEvent::Replace { backspaces, insert }) = ctx.engine.process_key(ch) {
            emit_delete_surrounding_text(conn, path, -(backspaces as i32), backspaces as u32);
            emit_commit_text(conn, path, &format!("{insert}{ch}"));
        } else {
            emit_commit_text(conn, path, &ch.to_string());
        }
        ctx.engine.reset();
        return true;
    }

    match ctx.engine.process_key(ch) {
        Some(EngineEvent::Replace { backspaces, insert }) => {
            emit_delete_surrounding_text(conn, path, -(backspaces as i32), backspaces as u32);
            emit_commit_text(conn, path, &insert);
            true
        }
        Some(EngineEvent::AutoRestore(s))
        | Some(EngineEvent::UndoTones { restored: s, .. })
        | Some(EngineEvent::Flush(s))
        | Some(EngineEvent::Paste(s)) => {
            emit_commit_text(conn, path, &s);
            true
        }
        Some(EngineEvent::Insert(s)) => {
            emit_commit_text(conn, path, &s);
            true
        }
        None => {
            emit_commit_text(conn, path, &ch.to_string());
            true
        }
    }
}

fn emit_delete_surrounding_text(conn: &Connection, path: &str, offset: i32, nchars: u32) {
    let m = engine_signal(path, "DeleteSurroundingText")
        .append1(MessageItem::Int32(offset))
        .append1(MessageItem::UInt32(nchars));
    let _ = conn.send(m);
}

fn engine_signal(path: &str, member: &str) -> Message {
    Message::signal(
        &Path::new(path).unwrap(),
        &Interface::new(ENGINE_INTERFACE).unwrap(),
        &Member::new(member).unwrap(),
    )
}

fn emit_commit_text(conn: &Connection, path: &str, text: &str) {
    let m = engine_signal(path, "CommitText").append1(ibus_text_variant(text));
    let _ = conn.send(m);
}

fn emit_update_preedit(conn: &Connection, path: &str, text: &str) {
    let m = engine_signal(path, "UpdatePreeditText")
        .append1(ibus_text_variant(text))
        .append1(text.chars().count() as u32)
        .append1(!text.is_empty())
        .append1(0u32);
    let _ = conn.send(m);
}

fn emit_update_auxiliary_text(conn: &Connection, path: &str, text: &str, visible: bool) {
    let m = engine_signal(path, "UpdateAuxiliaryText")
        .append1(ibus_text_variant(text))
        .append1(visible);
    let _ = conn.send(m);
    if visible {
        let _ = conn.send(engine_signal(path, "ShowAuxiliaryText"));
    } else {
        let _ = conn.send(engine_signal(path, "HideAuxiliaryText"));
    }
}

fn all_properties(focused: bool) -> MessageItem {
    let mut entries: Vec<(MessageItem, MessageItem)> = vec![
        (
            MessageItem::Str("FocusId".into()),
            focus_id_variant(focused),
        ),
        (
            MessageItem::Str("ActiveSurroundingText".into()),
            focus_id_variant(false),
        ),
        (
            MessageItem::Str("ContentType".into()),
            MessageItem::Variant(Box::new(MessageItem::Struct(vec![
                MessageItem::UInt32(0),
                MessageItem::UInt32(0),
            ]))),
        ),
    ];
    MessageItem::Dict(MessageItemDict::new(entries, sig("s"), sig("v")).unwrap())
}

fn show_osd(summary: &str, body: &str) {
    let summary = summary.to_string();
    let body = body.to_string();
    std::thread::spawn(move || {
        let _ = std::process::Command::new("notify-send")
            .args([
                "-t",
                "1200",
                "-h",
                "string:x-canonical-private-synchronous:vietc-osd",
                "-a",
                "Viet+",
                &summary,
                &body,
            ])
            .output();
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn struct_of(item: MessageItem) -> Vec<MessageItem> {
        let inner = match item {
            MessageItem::Variant(inner) => *inner,
            other => other,
        };
        match inner {
            MessageItem::Struct(s) => s,
            other => panic!("expected struct, got {:?}", other),
        }
    }

    #[test]
    fn component_variant_shape() {
        let children = struct_of(component_variant());
        // (s a{sv} ssssssss av av) => 12 children
        assert_eq!(children.len(), 12);
        assert_eq!(children[0], MessageItem::Str("IBusComponent".into()));
        // engines array is the last child and must be an array
        assert!(matches!(children[11], MessageItem::Array(_)));
    }

    #[test]
    fn engine_desc_variant_shape() {
        let children = struct_of(engine_desc_struct());
        // (s a{sv} sssssssussssssss) => 19 children
        assert_eq!(children.len(), 19);
        assert_eq!(children[0], MessageItem::Str("IBusEngineDesc".into()));
        assert_eq!(children[2], MessageItem::Str("vietc".into()));
    }

    #[test]
    fn text_variant_shape() {
        let children = struct_of(ibus_text_variant("võ"));
        assert_eq!(children.len(), 4);
        assert_eq!(children[0], MessageItem::Str("IBusText".into()));
        assert_eq!(children[2], MessageItem::Str("võ".into()));
        // attr list nested inside a variant
        match &children[3] {
            MessageItem::Variant(inner) => match **inner {
                MessageItem::Struct(ref s) => {
                    assert_eq!(s[0], MessageItem::Str("IBusAttrList".into()))
                }
                ref o => panic!("attr not struct: {:?}", o),
            },
            other => panic!("text attr not variant: {:?}", other),
        }
    }

    #[test]
    fn keyval_mapping() {
        assert_eq!(keyval_to_char(0x6F), Some('o'));
        assert_eq!(keyval_to_char(0x34), Some('4'));
        assert_eq!(keyval_to_char(0xFF08), Some('\x08'));
        assert_eq!(keyval_to_char(0xFF0D), None); // Return
    }

    #[test]
    fn release_mask_constants() {
        const IBUS_RELEASE_MASK: u32 = 1 << 30;
        let release_event_state: u32 = IBUS_RELEASE_MASK;
        assert_eq!(release_event_state & IBUS_RELEASE_MASK, IBUS_RELEASE_MASK);
        let press_event_state: u32 = 0;
        assert_eq!(press_event_state & IBUS_RELEASE_MASK, 0);
    }
}
