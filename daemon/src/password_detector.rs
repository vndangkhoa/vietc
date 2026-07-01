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

    fn check_atspi2(&self) -> Option<bool> {
        let conn = Connection::new_session().ok()?;
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
