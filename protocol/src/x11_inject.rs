use super::inject::{InjectResult, KeyInjector};

// X11 keycodes for common ASCII characters
// These are Linux evdev keycodes (same as X11 for most keys)
fn char_to_keycode(ch: char) -> Option<(u32, bool)> {
    match ch {
        'a' => Some((30, false)), 'b' => Some((48, false)), 'c' => Some((46, false)),
        'd' => Some((32, false)), 'e' => Some((18, false)), 'f' => Some((33, false)),
        'g' => Some((34, false)), 'h' => Some((35, false)), 'i' => Some((23, false)),
        'j' => Some((36, false)), 'k' => Some((37, false)), 'l' => Some((38, false)),
        'm' => Some((50, false)), 'n' => Some((49, false)), 'o' => Some((24, false)),
        'p' => Some((25, false)), 'q' => Some((16, false)), 'r' => Some((19, false)),
        's' => Some((31, false)), 't' => Some((20, false)), 'u' => Some((22, false)),
        'v' => Some((47, false)), 'w' => Some((17, false)), 'x' => Some((45, false)),
        'y' => Some((21, false)), 'z' => Some((44, false)),
        'A' => Some((30, true)), 'B' => Some((48, true)), 'C' => Some((46, true)),
        'D' => Some((32, true)), 'E' => Some((18, true)), 'F' => Some((33, true)),
        'G' => Some((34, true)), 'H' => Some((35, true)), 'I' => Some((23, true)),
        'J' => Some((36, true)), 'K' => Some((37, true)), 'L' => Some((38, true)),
        'M' => Some((50, true)), 'N' => Some((49, true)), 'O' => Some((24, true)),
        'P' => Some((25, true)), 'Q' => Some((16, true)), 'R' => Some((19, true)),
        'S' => Some((31, true)), 'T' => Some((20, true)), 'U' => Some((22, true)),
        'V' => Some((47, true)), 'W' => Some((17, true)), 'X' => Some((45, true)),
        'Y' => Some((21, true)), 'Z' => Some((44, true)),
        '0' => Some((11, false)), '1' => Some((2, false)), '2' => Some((3, false)),
        '3' => Some((4, false)), '4' => Some((5, false)), '5' => Some((6, false)),
        '6' => Some((7, false)), '7' => Some((8, false)), '8' => Some((9, false)),
        '9' => Some((10, false)),
        ' ' => Some((57, false)), '.' => Some((52, false)), ',' => Some((51, false)),
        '-' => Some((12, false)), '=' => Some((13, false)), ';' => Some((39, false)),
        '\'' => Some((40, false)), '/' => Some((53, false)), '\\' => Some((43, false)),
        '`' => Some((41, false)), '[' => Some((26, false)), ']' => Some((27, false)),
        _ => None,
    }
}

/// X11 injection backend using XTEST extension
///
/// Sends fake key events via XSendEvent/XTestFakeKeyEvent.
/// Works on X11 sessions. Falls back to uinput on Wayland.
pub struct X11Injector {
    display: *mut xlib::Display,
    #[allow(dead_code)]
    window: xlib::Window,
}

unsafe impl Send for X11Injector {}
unsafe impl Sync for X11Injector {}

impl X11Injector {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let display = xlib::XOpenDisplay(std::ptr::null());
            if display.is_null() {
                return Err("Cannot open X11 display. Is DISPLAY set?".into());
            }
            let window = xlib::XDefaultRootWindow(display);
            Ok(Self { display, window })
        }
    }

    fn send_keycode(&self, keycode: u32, shift: bool) {
        unsafe {
            if shift {
                xlib::XTestFakeKeyEvent(self.display, 50, 1, 0); // Shift press
            }
            xlib::XTestFakeKeyEvent(self.display, keycode, 1, 0); // Key press
            xlib::XTestFakeKeyEvent(self.display, keycode, 0, 0); // Key release
            if shift {
                xlib::XTestFakeKeyEvent(self.display, 50, 0, 0); // Shift release
            }
            xlib::XFlush(self.display);
        }
    }

    fn send_unicode_via_xdotool(&self, ch: char) {
        // For Unicode chars, use xdotool type as fallback
        let s = ch.to_string();
        let _ = std::process::Command::new("xdotool")
            .args(["type", "--clearmodifiers", &s])
            .output();
    }
}

impl KeyInjector for X11Injector {
    fn send_backspace(&self) -> InjectResult {
        self.send_keycode(14, false); // KEY_BACKSPACE
        InjectResult::Success
    }

    fn send_char(&self, ch: char) -> InjectResult {
        if let Some((keycode, shift)) = char_to_keycode(ch) {
            self.send_keycode(keycode, shift);
            InjectResult::Success
        } else {
            // Unicode char - use xdotool
            self.send_unicode_via_xdotool(ch);
            InjectResult::Success
        }
    }

    fn send_string(&self, s: &str) -> InjectResult {
        for ch in s.chars() {
            self.send_char(ch);
        }
        InjectResult::Success
    }

    fn flush(&self) -> InjectResult {
        unsafe { xlib::XFlush(self.display); }
        InjectResult::Success
    }
}

impl Drop for X11Injector {
    fn drop(&mut self) {
        unsafe { xlib::XCloseDisplay(self.display); }
    }
}

// Minimal Xlib/XTEST FFI
mod xlib {
    use std::ffi::c_void;

    pub type Display = c_void;
    pub type Window = u64;

    extern "C" {
        pub fn XOpenDisplay(name: *const std::ffi::c_char) -> *mut Display;
        pub fn XCloseDisplay(display: *mut Display) -> std::ffi::c_int;
        pub fn XDefaultRootWindow(display: *mut Display) -> Window;
        pub fn XFlush(display: *mut Display) -> std::ffi::c_int;
        pub fn XTestFakeKeyEvent(
            display: *mut Display,
            keycode: u32,
            state: std::ffi::c_int,
            time: u64,
        ) -> std::ffi::c_int;
    }
}
