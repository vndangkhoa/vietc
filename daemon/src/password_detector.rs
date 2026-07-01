use std::collections::HashMap;
use std::time::Duration;

use dbus::arg::{RefArg, Variant};
use dbus::blocking::Connection;

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
