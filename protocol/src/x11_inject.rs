// SPDX-License-Identifier: MIT
use super::inject::{InjectResult, KeyInjector};
use std::cell::RefCell;
use std::ffi::{c_char, c_int, c_void};

type Display = c_void;
type Window = u64;
type Atom = u64;
type Time = u64;

extern "C" {
    fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
}

const CURRENT_TIME: Time = 0;
const PROP_MODE_REPLACE: c_int = 0;
const NO_EVENT_MASK: i64 = 0;
const COPY_FROM_PARENT: Window = 0;

const SELECTION_REQUEST: c_int = 30;
const SELECTION_NOTIFY: c_int = 31;

struct X11Lib {
    x11_handle: *mut c_void,
    xtst_handle: *mut c_void,

    x_open_display: unsafe extern "C" fn(*const c_char) -> *mut Display,
    x_close_display: unsafe extern "C" fn(*mut Display) -> c_int,
    x_default_root_window: unsafe extern "C" fn(*mut Display) -> Window,
    x_flush: unsafe extern "C" fn(*mut Display) -> c_int,
    x_test_fake_key_event: unsafe extern "C" fn(*mut Display, u32, c_int, u64) -> c_int,
    x_intern_atom: unsafe extern "C" fn(*mut Display, *const c_char, c_int) -> Atom,
    x_set_selection_owner: unsafe extern "C" fn(*mut Display, Atom, Window, Time) -> c_int,
    x_change_property: unsafe extern "C" fn(*mut Display, Window, Atom, Atom, c_int, c_int, *const c_void, c_int) -> c_int,
    x_send_event: unsafe extern "C" fn(*mut Display, Window, c_int, i64, *const c_void) -> c_int,
    x_create_simple_window: unsafe extern "C" fn(*mut Display, Window, c_int, c_int, c_int, c_int, c_int, Atom, Atom) -> Window,
    x_map_window: unsafe extern "C" fn(*mut Display, Window) -> c_int,
    x_destroy_window: unsafe extern "C" fn(*mut Display, Window) -> c_int,
    x_pending: unsafe extern "C" fn(*mut Display) -> c_int,
    x_next_event: unsafe extern "C" fn(*mut Display, *mut XEvent),
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
                x11_handle = dlopen(path, 1);
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

            macro_rules! sym {
                ($handle:expr, $name:expr) => {
                    std::mem::transmute(dlsym($handle, concat!($name, "\0").as_ptr() as *const c_char))
                };
            }

            let x_open_display = sym!(x11_handle, "XOpenDisplay");
            let x_close_display = sym!(x11_handle, "XCloseDisplay");
            let x_default_root_window = sym!(x11_handle, "XDefaultRootWindow");
            let x_flush = sym!(x11_handle, "XFlush");
            let x_intern_atom = sym!(x11_handle, "XInternAtom");
            let x_set_selection_owner = sym!(x11_handle, "XSetSelectionOwner");
            let x_change_property = sym!(x11_handle, "XChangeProperty");
            let x_send_event = sym!(x11_handle, "XSendEvent");
            let x_create_simple_window = sym!(x11_handle, "XCreateSimpleWindow");
            let x_map_window = sym!(x11_handle, "XMapWindow");
            let x_destroy_window = sym!(x11_handle, "XDestroyWindow");
            let x_pending = sym!(x11_handle, "XPending");
            let x_next_event = sym!(x11_handle, "XNextEvent");
            let x_test_fake_key_event = sym!(xtst_handle, "XTestFakeKeyEvent");

            Ok(Self {
                x11_handle,
                xtst_handle,
                x_open_display,
                x_close_display,
                x_default_root_window,
                x_flush,
                x_test_fake_key_event,
                x_intern_atom,
                x_set_selection_owner,
                x_change_property,
                x_send_event,
                x_create_simple_window,
                x_map_window,
                x_destroy_window,
                x_pending,
                x_next_event,
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

fn char_to_keycode(ch: char) -> Option<(u32, bool)> {
    match ch {
        'a' => Some((30, false)),
        'b' => Some((48, false)),
        'c' => Some((46, false)),
        'd' => Some((32, false)),
        'e' => Some((18, false)),
        'f' => Some((33, false)),
        'g' => Some((34, false)),
        'h' => Some((35, false)),
        'i' => Some((23, false)),
        'j' => Some((36, false)),
        'k' => Some((37, false)),
        'l' => Some((38, false)),
        'm' => Some((50, false)),
        'n' => Some((49, false)),
        'o' => Some((24, false)),
        'p' => Some((25, false)),
        'q' => Some((16, false)),
        'r' => Some((19, false)),
        's' => Some((31, false)),
        't' => Some((20, false)),
        'u' => Some((22, false)),
        'v' => Some((47, false)),
        'w' => Some((17, false)),
        'x' => Some((45, false)),
        'y' => Some((21, false)),
        'z' => Some((44, false)),
        'A' => Some((30, true)),
        'B' => Some((48, true)),
        'C' => Some((46, true)),
        'D' => Some((32, true)),
        'E' => Some((18, true)),
        'F' => Some((33, true)),
        'G' => Some((34, true)),
        'H' => Some((35, true)),
        'I' => Some((23, true)),
        'J' => Some((36, true)),
        'K' => Some((37, true)),
        'L' => Some((38, true)),
        'M' => Some((50, true)),
        'N' => Some((49, true)),
        'O' => Some((24, true)),
        'P' => Some((25, true)),
        'Q' => Some((16, true)),
        'R' => Some((19, true)),
        'S' => Some((31, true)),
        'T' => Some((20, true)),
        'U' => Some((22, true)),
        'V' => Some((47, true)),
        'W' => Some((17, true)),
        'X' => Some((45, true)),
        'Y' => Some((21, true)),
        'Z' => Some((44, true)),
        _ => None,
    }
}

#[repr(C)]
struct XSelectionRequestEvent {
    _type: c_int,
    _serial: u64,
    _send_event: c_int,
    _display: *mut Display,
    owner: Window,
    requestor: Window,
    selection: Atom,
    target: Atom,
    property: Atom,
    time: Time,
}

#[repr(C)]
struct XEvent {
    _bytes: [u8; 192],
}

impl XEvent {
    fn event_type(&self) -> c_int {
        unsafe { std::ptr::read_unaligned(self._bytes.as_ptr() as *const c_int) }
    }
}

pub struct X11Injector {
    lib: X11Lib,
    display: *mut Display,
    clipboard_window: Window,
    atom_clipboard: Atom,
    atom_utf8: Atom,
    atom_targets: Atom,
    atom_string: Atom,
    clipboard_text: RefCell<String>,
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
            let root = (lib.x_default_root_window)(display);

            let atom_clipboard = (lib.x_intern_atom)(display, b"CLIPBOARD\0".as_ptr() as *const c_char, 0);
            let atom_utf8 = (lib.x_intern_atom)(display, b"UTF8_STRING\0".as_ptr() as *const c_char, 0);
            let atom_targets = (lib.x_intern_atom)(display, b"TARGETS\0".as_ptr() as *const c_char, 0);
            let atom_string = (lib.x_intern_atom)(display, b"STRING\0".as_ptr() as *const c_char, 0);

            // Create a small hidden window for clipboard ownership
            let clipboard_window = (lib.x_create_simple_window)(
                display, root, 0, 0, 1, 1, 0, COPY_FROM_PARENT, COPY_FROM_PARENT,
            );
            (lib.x_map_window)(display, clipboard_window);

            Ok(Self {
                lib,
                display,
                clipboard_window,
                atom_clipboard,
                atom_utf8,
                atom_targets,
                atom_string,
                clipboard_text: RefCell::new(String::new()),
            })
        }
    }

    fn set_clipboard_text(&self, text: &str) {
        *self.clipboard_text.borrow_mut() = text.to_string();
        unsafe {
            // Set the text as a property on our clipboard window
            (self.lib.x_change_property)(
                self.display,
                self.clipboard_window,
                self.atom_clipboard,
                self.atom_utf8,
                8, // 8-bit format
                PROP_MODE_REPLACE,
                text.as_ptr() as *const c_void,
                text.len() as c_int,
            );

            // Also set as STRING (for apps that don't understand UTF8_STRING)
            (self.lib.x_change_property)(
                self.display,
                self.clipboard_window,
                self.atom_clipboard,
                self.atom_string,
                8,
                PROP_MODE_REPLACE,
                text.as_ptr() as *const c_void,
                text.len() as c_int,
            );

            // Claim the CLIPBOARD selection
            (self.lib.x_set_selection_owner)(
                self.display,
                self.atom_clipboard,
                self.clipboard_window,
                CURRENT_TIME,
            );

            (self.lib.x_flush)(self.display);
        }
    }

    fn handle_pending_events(&self) {
        unsafe {
            while (self.lib.x_pending)(self.display) > 0 {
                let mut event: XEvent = std::mem::zeroed();
                (self.lib.x_next_event)(self.display, &mut event);
                if event.event_type() == SELECTION_REQUEST {
                    let req = &*(&event as *const XEvent as *const XSelectionRequestEvent);
                    self.handle_selection_request(req);
                }
            }
        }
    }

    fn handle_selection_request(&self, req: &XSelectionRequestEvent) {
        eprintln!(
            "[vietc] SelectionRequest: target={} requestor={}",
            req.target, req.requestor
        );

        // Determine what property to use for the response
        let property = if req.property == 0 {
            req.target // Use the target atom as property if property is None
        } else {
            req.property
        };

        unsafe {
            if req.target == self.atom_targets {
                // Respond with supported targets: TARGETS, UTF8_STRING, STRING
                let targets: [Atom; 3] = [self.atom_targets, self.atom_utf8, self.atom_string];
                (self.lib.x_change_property)(
                    self.display,
                    req.requestor,
                    property,
                    self.atom_targets,
                    32, // 32-bit format
                    PROP_MODE_REPLACE,
                    targets.as_ptr() as *const c_void,
                    targets.len() as c_int,
                );
            } else if req.target == self.atom_utf8 || req.target == self.atom_string {
                // Respond with the actual clipboard text
                (self.lib.x_change_property)(
                    self.display,
                    req.requestor,
                    property,
                    req.target,
                    8, // 8-bit format
                    PROP_MODE_REPLACE,
                    self.clipboard_text.borrow().as_ptr() as *const c_void,
                    self.clipboard_text.borrow().len() as c_int,
                );
            }

            // Send SelectionNotify to inform the requestor
            let mut notify = std::mem::zeroed::<XSelectionNotifyEvent>();
            notify._type = SELECTION_NOTIFY as c_int;
            notify._display = self.display;
            notify.requestor = req.requestor;
            notify.selection = req.selection;
            notify.target = req.target;
            notify.property = if req.target == self.atom_targets
                || req.target == self.atom_utf8
                || req.target == self.atom_string
            {
                property
            } else {
                0 // PropertyNone = unsupported target
            };
            notify.time = req.time;

            (self.lib.x_send_event)(
                self.display,
                req.requestor,
                0, // propagate = False
                NO_EVENT_MASK,
                &notify as *const XSelectionNotifyEvent as *const c_void,
            );
            (self.lib.x_flush)(self.display);
        }
    }

    fn paste_via_clipboard(&self, backspaces: usize, text: &str) -> bool {
        // Set clipboard text directly via X11
        self.set_clipboard_text(text);

        // Handle any pending SelectionRequest events that may have queued
        // (unlikely at this point, but be safe)
        self.handle_pending_events();

        // Send backspaces via XTest (X11 keycode 22 = backspace)
        if backspaces > 0 {
            for _ in 0..backspaces {
                self.send_keycode(22, false);
            }
        }

        // Send Ctrl+V via XTest to paste (evdev codes + 8 = X11)
        unsafe {
            (self.lib.x_test_fake_key_event)(self.display, 29 + 8, 1, 0); // Ctrl_L press
            (self.lib.x_test_fake_key_event)(self.display, 47 + 8, 1, 0); // V press
            (self.lib.x_test_fake_key_event)(self.display, 47 + 8, 0, 0); // V release
            (self.lib.x_test_fake_key_event)(self.display, 29 + 8, 0, 0); // Ctrl_L release
            (self.lib.x_flush)(self.display);
        }

        // Handle SelectionRequest events that come from the paste target
        // Process events with a short spin loop (up to ~50ms)
        for _ in 0..4 {
            // Brief sleep to let X11 events propagate
            std::thread::sleep(std::time::Duration::from_millis(5));
            self.handle_pending_events();
        }

        true
    }

    fn send_keycode(&self, evdev_keycode: u32, shift: bool) {
        let x11 = evdev_keycode + 8;
        unsafe {
            if shift {
                (self.lib.x_test_fake_key_event)(self.display, 42 + 8, 1, 0); // Shift_L
            }
            (self.lib.x_test_fake_key_event)(self.display, x11, 1, 0);
            (self.lib.x_test_fake_key_event)(self.display, x11, 0, 0);
            if shift {
                (self.lib.x_test_fake_key_event)(self.display, 42 + 8, 0, 0);
            }
            (self.lib.x_flush)(self.display);
        }
    }

    fn send_unicode_via_xdotool(&self, ch: char) {
        let s = ch.to_string();

        // Try ydotool first (uinput-based, works as root)
        let ydotool_ok = std::process::Command::new("ydotool")
            .args(["type", &s])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if ydotool_ok {
            return;
        }

        // Try xdotool
        let xdotool_ok = std::process::Command::new("xdotool")
            .args(["type", "--clearmodifiers", &s])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if xdotool_ok {
            return;
        }

        // Fallback: direct X11 clipboard + Ctrl+V
        self.paste_via_clipboard(0, &s);
    }
}

impl Drop for X11Injector {
    fn drop(&mut self) {
        unsafe {
            if self.clipboard_window != 0 && !self.display.is_null() {
                (self.lib.x_destroy_window)(self.display, self.clipboard_window);
            }
            (self.lib.x_close_display)(self.display);
        }
    }
}

#[repr(C)]
struct XSelectionNotifyEvent {
    _type: c_int,
    _serial: u64,
    _send_event: c_int,
    _display: *mut Display,
    requestor: Window,
    selection: Atom,
    target: Atom,
    property: Atom,
    time: Time,
}

impl KeyInjector for X11Injector {
    fn send_key_event(&self, keycode: u16, value: i32) -> InjectResult {
        // X11 keycodes = Linux evdev keycodes + 8
        let x11_keycode = keycode as u32 + 8;
        unsafe {
            (self.lib.x_test_fake_key_event)(self.display, x11_keycode, value, 0);
            (self.lib.x_flush)(self.display);
        }
        InjectResult::Success
    }

    fn send_backspace(&self) -> InjectResult {
        self.send_keycode(22, false); // X11 keycode 22 = backspace
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
        // ASCII: type individual characters via XTest (fast, no side effects)
        let is_ascii = s.chars().all(|c| char_to_keycode(c).is_some());
        if is_ascii {
            for ch in s.chars() {
                self.send_char(ch);
            }
            return InjectResult::Success;
        }

        // Non-ASCII (Vietnamese Unicode): use clipboard paste via X11 API + XTest
        // This avoids xdotool/ydotool subprocesses that silently drop Vietnamese.
        self.paste_via_clipboard(0, s);
        InjectResult::Success
    }

    fn inject_replacement(&self, backspaces: usize, text: &str) -> InjectResult {
        let is_ascii = text.chars().all(|c| char_to_keycode(c).is_some());
        if is_ascii {
            if backspaces > 0 {
                for _ in 0..backspaces {
                    self.send_keycode(14, false);
                }
            }
            for ch in text.chars() {
                if let Some((keycode, shift)) = char_to_keycode(ch) {
                    self.send_keycode(keycode, shift);
                }
            }
            return InjectResult::Success;
        }

        // Contains Unicode: use direct X11 clipboard + XTest Ctrl+V
        self.paste_via_clipboard(backspaces, text);
        InjectResult::Success
    }

    fn flush(&self) -> InjectResult {
        unsafe {
            (self.lib.x_flush)(self.display);
        }
        InjectResult::Success
    }

    fn update_pasted_text(&self, _text: &str) -> InjectResult {
        InjectResult::Success
    }
}
