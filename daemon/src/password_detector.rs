use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use dbus::arg::{RefArg, Variant};
use dbus::blocking::{Connection, Proxy};
use dbus::Path;

const ROLE_PASSWORD_TEXT: i32 = 62;

pub struct PasswordDetector {
    cached: Option<bool>,
    atspi_ok: bool,
}

impl PasswordDetector {
    pub fn new() -> Self {
        Self { cached: None, atspi_ok: false }
    }

    pub fn check(&mut self) -> Option<bool> {
        let r = self.check_atspi2();
        self.atspi_ok = r.is_some();
        if let Some(v) = r {
            self.cached = Some(v);
        }
        r
    }

    pub fn is_available(&self) -> bool {
        self.atspi_ok
    }

    pub fn cached_result(&self) -> Option<bool> {
        self.cached
    }

    /// Get the AT-SPI2 accessibility bus address via session bus
    fn get_a11y_bus_address() -> Option<String> {
        let conn = Connection::new_session().ok()?;
        let proxy = conn.with_proxy(
            "org.a11y.Bus",
            "/org/a11y/bus",
            Duration::from_secs(2),
        );
        let (addr,): (String,) = proxy
            .method_call("org.a11y.Bus", "GetAddress", ())
            .ok()?;
        Some(addr)
    }

    fn check_atspi2(&self) -> Option<bool> {
        // AT-SPI2 runs on its own private D-Bus (accessibility bus),
        // NOT on the session bus. We must first get the a11y bus address.
        let addr = Self::get_a11y_bus_address()?;
        let conn = Connection::new_address(&addr).ok()?;
        let timeout = Duration::from_secs(2);

        let proxy = conn.with_proxy(
            "org.a11y.atspi.Registry",
            "/org/a11y/atspi/registry",
            timeout,
        );

        let (bus_name, props, _children): (String, HashMap<String, Variant<Box<dyn RefArg>>>, Vec<Variant<Box<dyn RefArg>>>) =
            proxy.method_call("org.a11y.atspi.Registry", "GetFocus", ()).ok()?;

        let path_variant = props.get("path")?;
        let path_str = path_variant.0.as_str()?;

        let acc_proxy = conn.with_proxy(&bus_name, path_str, timeout);

        let (role,): (i32,) = acc_proxy.method_call("org.a11y.atspi.Accessible", "GetRole", ()).ok()?;

        Some(role == ROLE_PASSWORD_TEXT)
    }
}

/// AT-SPI2 state bit for the focused node (AccessibleState.FOCUSED = 12).
const ATSPI_STATE_FOCUSED: u32 = 1 << 12;

/// Cache for the (expensive) focused-window walk, refreshed at most once per
/// ~900ms so the 250ms controller poll doesn't re-walk the whole a11y tree.
fn atspi_cache() -> &'static Mutex<Option<(String, Instant)>> {
    static C: OnceLock<Mutex<Option<(String, Instant)>>> = OnceLock::new();
    C.get_or_init(|| Mutex::new(None))
}

/// Identify the focused window on Wayland-native GNOME, where GNOME Shell
/// `Eval` is gated off and the X11/xprop fallbacks cannot see the window.
///
/// The classic `Registry.GetFocus` method is absent on current AT-SPI2, so we
/// descend the desktop accessibility tree looking for the node carrying the
/// FOCUSED state and return its owning application's name (lower-cased).
pub fn get_focused_window_class_atspi() -> Option<String> {
    let now = Instant::now();
    if let Ok(guard) = atspi_cache().lock() {
        if let Some((ref name, ref t)) = *guard {
            if now.duration_since(*t) < Duration::from_millis(900) {
                return Some(name.clone());
            }
        }
    }
    let name = atspi_compute();
    // On GNOME Wayland the shell's own window ("gnome-shell" / "Main stage")
    // carries the FOCUSED bit while client windows do not, so the walk can
    // only ever resolve to the shell here. Treat that as "unknown" rather than
    // misreporting the app as `gnome-shell`. On setups where AT-SPI reports
    // real client focus this returns the actual app name.
    let name = name.filter(|n| n != "gnome-shell" && n != "gnome-shell.desktop");
    if let Ok(mut guard) = atspi_cache().lock() {
        *guard = name.as_ref().map(|n| (n.clone(), now));
    }
    name
}

fn atspi_compute() -> Option<String> {
    let addr = PasswordDetector::get_a11y_bus_address()?;
    let conn = Connection::new_address(&addr).ok()?;
    let timeout = Duration::from_millis(150);

    let root = Proxy::new(
        "org.a11y.atspi.Registry",
        "/org/a11y/atspi/accessible/root",
        timeout,
        &conn,
    );

    let (children,): (Vec<(String, Path)>,) = root
        .method_call("org.a11y.atspi.Accessible", "GetChildren", ())
        .ok()?;

    let mut budget: u32 = 300;
    for (bus, path) in children {
        let bus_s: &str = bus.as_str();
        let path_s = format!("{}", path);
        let app_root = Proxy::new(bus_s, path_s.as_str(), timeout, &conn);
        if let Some(name) = atspi_dfs(&conn, &app_root, 0, timeout, &mut budget) {
            return Some(name);
        }
        if budget == 0 {
            break;
        }
    }
    None
}

fn atspi_dfs(
    conn: &Connection,
    proxy: &Proxy<'_, &Connection>,
    depth: u32,
    timeout: Duration,
    budget: &mut u32,
) -> Option<String> {
    if depth > 7 || *budget == 0 {
        return None;
    }
    *budget -= 1;

    let states: Vec<u32> = match proxy
        .method_call("org.a11y.atspi.Accessible", "GetState", ())
    {
        Ok((s,)) => s,
        Err(_) => Vec::new(),
    };
    if let Some(&first) = states.first() {
        if first & ATSPI_STATE_FOCUSED != 0 {
            if let Ok(((bus, app_path),)) = proxy
                .method_call::<((String, Path),), (), _, _>(
                    "org.a11y.atspi.Accessible",
                    "GetApplication",
                    (),
                )
            {
                let bus_s: &str = bus.as_str();
                let path_s = format!("{}", app_path);
                let app_proxy = Proxy::new(bus_s, path_s.as_str(), timeout, conn);
                if let Some(name) = atspi_name(&app_proxy) {
                    return Some(name.to_lowercase());
                }
            }
        }
    }

    let children: Vec<(String, Path)> = match proxy
        .method_call("org.a11y.atspi.Accessible", "GetChildren", ())
    {
        Ok((c,)) => c,
        Err(_) => Vec::new(),
    };
    for (bus, path) in children {
        let bus_s: &str = bus.as_str();
        let path_s = format!("{}", path);
        let child = Proxy::new(bus_s, path_s.as_str(), timeout, conn);
        if let Some(name) = atspi_dfs(conn, &child, depth + 1, timeout, budget) {
            return Some(name);
        }
        if *budget == 0 {
            break;
        }
    }
    None
}

fn atspi_name(proxy: &Proxy<'_, &Connection>) -> Option<String> {
    let (variant,): (Variant<Box<dyn RefArg>>,) = proxy
        .method_call(
            "org.freedesktop.DBus.Properties",
            "Get",
            ("org.a11y.atspi.Accessible", "Name"),
        )
        .ok()?;
    let name: &str = variant.0.as_str()?;
    Some(name.to_string())
}
