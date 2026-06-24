use super::inject::{InjectResult, KeyInjector};
use std::ffi::{c_char, c_int, c_void};

type Display = c_void;
type Window = u64;

// Dynamic linker FFI
extern "C" {
    fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
}

struct X11Lib {
    x11_handle: *mut c_void,
    xtst_handle: *mut c_void,
    
    // Symbols
    x_open_display: unsafe extern "C" fn(*const c_char) -> *mut Display,
    x_close_display: unsafe extern "C" fn(*mut Display) -> c_int,
    x_default_root_window: unsafe extern "C" fn(*mut Display) -> Window,
    x_flush: unsafe extern "C" fn(*mut Display) -> c_int,
    x_test_fake_key_event: unsafe extern "C" fn(*mut Display, u32, c_int, u64) -> c_int,
}

impl X11Lib {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let x11_paths = [
                b"libX11.so.6\0".as_ptr() as *const c_char,
                b"libX11.so\0".as_ptr() as *const c_char,
            ];
            let mut x11_handle = std::ptr::null_mut();
            for path in x11_paths {
                x11_handle = dlopen(path, 1); // RTLD_LAZY
                if !x11_handle.is_null() {
                    break;
                }
            }
            if x11_handle.is_null() {
                return Err("Failed to load libX11.so.6".into());
            }

            let xtst_paths = [
                b"libXtst.so.6\0".as_ptr() as *const c_char,
                b"libXtst.so\0".as_ptr() as *const c_char,
            ];
            let mut xtst_handle = std::ptr::null_mut();
            for path in xtst_paths {
                xtst_handle = dlopen(path, 1);
                if !xtst_handle.is_null() {
                    break;
                }
            }
            if xtst_handle.is_null() {
                dlclose(x11_handle);
                return Err("Failed to load libXtst.so.6".into());
            }

            let x_open_display = std::mem::transmute(dlsym(x11_handle, b"XOpenDisplay\0".as_ptr() as *const c_char));
            let x_close_display = std::mem::transmute(dlsym(x11_handle, b"XCloseDisplay\0".as_ptr() as *const c_char));
            let x_default_root_window = std::mem::transmute(dlsym(x11_handle, b"XDefaultRootWindow\0".as_ptr() as *const c_char));
            let x_flush = std::mem::transmute(dlsym(x11_handle, b"XFlush\0".as_ptr() as *const c_char));
            let x_test_fake_key_event = std::mem::transmute(dlsym(xtst_handle, b"XTestFakeKeyEvent\0".as_ptr() as *const c_char));

            Ok(Self {
                x11_handle,
                xtst_handle,
                x_open_display,
                x_close_display,
                x_default_root_window,
                x_flush,
                x_test_fake_key_event,
            })
        }
    }
}

impl Drop for X11Lib {
    fn drop(&mut self) {
        unsafe {
            dlclose(self.x11_handle);
            dlclose(self.xtst_handle);
        }
    }
}

// X11 keycodes for common ASCII characters
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

pub struct X11Injector {
    lib: X11Lib,
    display: *mut Display,
    #[allow(dead_code)]
    window: Window,
}

unsafe impl Send for X11Injector {}
unsafe impl Sync for X11Injector {}

impl X11Injector {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let lib = X11Lib::new()?;
        unsafe {
            let display = (lib.x_open_display)(std::ptr::null());
            if display.is_null() {
                return Err("Cannot open X11 display. Is DISPLAY set?".into());
            }
            let window = (lib.x_default_root_window)(display);
            Ok(Self { lib, display, window })
        }
    }

    fn send_keycode(&self, keycode: u32, shift: bool) {
        unsafe {
            if shift {
                (self.lib.x_test_fake_key_event)(self.display, 50, 1, 0); // Shift press
            }
            (self.lib.x_test_fake_key_event)(self.display, keycode, 1, 0); // Key press
            (self.lib.x_test_fake_key_event)(self.display, keycode, 0, 0); // Key release
            if shift {
                (self.lib.x_test_fake_key_event)(self.display, 50, 0, 0); // Shift release
            }
            (self.lib.x_flush)(self.display);
        }
    }

    fn send_unicode_via_xdotool(&self, ch: char) {
        // For Unicode chars, try ydotool first (uinput-based, works as root),
        // then xdotool (X11 XTest) as fallback.
        let s = ch.to_string();
        let ydotool_ok = std::process::Command::new("ydotool")
            .args(["type", &s])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ydotool_ok {
            return;
        }
        let xdotool_ok = std::process::Command::new("xdotool")
            .args(["type", "--clearmodifiers", &s])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if xdotool_ok {
            return;
        }
        // Clipboard fallback: xclip + Ctrl+V via XTEST
        let copied = std::process::Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child.stdin.take().unwrap().write_all(s.as_bytes())?;
                child.wait()
            })
            .map(|status| status.success())
            .unwrap_or(false);
        if copied {
            unsafe {
                (self.lib.x_test_fake_key_event)(self.display, 29, 1, 0); // Ctrl press
                (self.lib.x_test_fake_key_event)(self.display, 47, 1, 0); // V press
                (self.lib.x_test_fake_key_event)(self.display, 47, 0, 0); // V release
                (self.lib.x_test_fake_key_event)(self.display, 29, 0, 0); // Ctrl release
                (self.lib.x_flush)(self.display);
            }
        }
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

    fn inject_replacement(&self, backspaces: usize, text: &str) -> InjectResult {
        let is_ascii = text.chars().all(|c| char_to_keycode(c).is_some());

        if is_ascii {
            if backspaces > 0 {
                for _ in 0..backspaces {
                    self.send_keycode(14, false); // KEY_BACKSPACE
                }
            }
            for ch in text.chars() {
                if let Some((keycode, shift)) = char_to_keycode(ch) {
                    self.send_keycode(keycode, shift);
                }
            }
            return InjectResult::Success;
        }

        // Contains Unicode: try xdotool with both backspaces and text in a single command
        let has_xdotool = std::process::Command::new("which")
            .arg("xdotool")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        if has_xdotool {
            let mut args = Vec::new();
            if backspaces > 0 {
                args.push("key".to_string());
                for _ in 0..backspaces {
                    args.push("BackSpace".to_string());
                }
            }
            if !text.is_empty() {
                args.push("type".to_string());
                args.push("--clearmodifiers".to_string());
                args.push(text.to_string());
            }

            let ok = std::process::Command::new("xdotool")
                .args(&args)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            if ok {
                return InjectResult::Success;
            }
        }

        // Fallback: Clipboard copy + paste.
        // Send backspaces via XTEST, then copy to clipboard, then paste (Ctrl+V) via XTEST.
        // Since all XTEST key events go through the same display connection, their ordering is guaranteed.
        let mut clipboard_cmd = std::process::Command::new("xclip");
        clipboard_cmd.args(["-selection", "clipboard"]);
        clipboard_cmd.stdin(std::process::Stdio::piped());
        let copied = clipboard_cmd.spawn()
            .and_then(|mut child| {
                use std::io::Write;
                child.stdin.take().unwrap().write_all(text.as_bytes())?;
                child.wait()
            })
            .map(|status| status.success())
            .unwrap_or(false);

        if copied {
            if backspaces > 0 {
                for _ in 0..backspaces {
                    self.send_keycode(14, false); // KEY_BACKSPACE
                }
            }
            unsafe {
                (self.lib.x_test_fake_key_event)(self.display, 29, 1, 0); // Ctrl press
                (self.lib.x_test_fake_key_event)(self.display, 47, 1, 0); // V press
                (self.lib.x_test_fake_key_event)(self.display, 47, 0, 0); // V release
                (self.lib.x_test_fake_key_event)(self.display, 29, 0, 0); // Ctrl release
                (self.lib.x_flush)(self.display);
            }
            InjectResult::Success
        } else {
            // Absolute last resort: backspaces via XTEST followed by individual unicode send_unicode_via_xdotool
            if backspaces > 0 {
                for _ in 0..backspaces {
                    self.send_keycode(14, false); // KEY_BACKSPACE
                }
            }
            for ch in text.chars() {
                self.send_char(ch);
            }
            InjectResult::Success
        }
    }

    fn flush(&self) -> InjectResult {
        unsafe { (self.lib.x_flush)(self.display); }
        InjectResult::Success
    }
}

impl Drop for X11Injector {
    fn drop(&mut self) {
        unsafe { (self.lib.x_close_display)(self.display); }
    }
}
