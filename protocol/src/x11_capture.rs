use std::ffi::{c_char, c_int, c_void};

type Display = c_void;
type Window = u64;
type Time = u64;

// X11 event types
const KEY_PRESS: c_int = 2;
const KEY_RELEASE: c_int = 3;
const FOCUS_IN: c_int = 9;
const FOCUS_OUT: c_int = 10;

// X11 modifier masks
const CONTROL_MASK: c_int = 4;
const MOD1_MASK: c_int = 8;  // Alt
const MOD4_MASK: c_int = 64; // Super/Win

// Grab modes
const GRAB_MODE_ASYNC: c_int = 1;

extern "C" {
    fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
}

struct X11Lib {
    handle: *mut c_void,
    x_open_display: unsafe extern "C" fn(*const c_char) -> *mut Display,
    x_close_display: unsafe extern "C" fn(*mut Display) -> c_int,
    x_default_root_window: unsafe extern "C" fn(*mut Display) -> Window,
    x_grab_keyboard: unsafe extern "C" fn(*mut Display, Window, c_int, c_int, c_int, Time) -> c_int,
    x_ungrab_keyboard: unsafe extern "C" fn(*mut Display, Time) -> c_int,
    x_pending: unsafe extern "C" fn(*mut Display) -> c_int,
    x_next_event: unsafe extern "C" fn(*mut Display, *mut XEvent),
    x_lookup_string: unsafe extern "C" fn(*mut XKeyEvent, *mut c_char, c_int, *mut KeySym, *mut c_int) -> c_int,
    x_utf8_lookup_string: Option<unsafe extern "C" fn(*mut XKeyEvent, *mut c_char, c_int, *mut KeySym, *mut c_int) -> c_int>,
    x_flush: unsafe extern "C" fn(*mut Display) -> c_int,
}

impl X11Lib {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        unsafe {
            let paths = [
                b"libX11.so.6\0".as_ptr() as *const c_char,
                b"libX11.so\0".as_ptr() as *const c_char,
            ];
            let mut handle = std::ptr::null_mut();
            for path in paths {
                handle = dlopen(path, 1);
                if !handle.is_null() {
                    break;
                }
            }
            if handle.is_null() {
                return Err("Failed to load libX11.so.6".into());
            }

            macro_rules! sym {
                ($name:expr) => {
                    std::mem::transmute(dlsym(handle, concat!($name, "\0").as_ptr() as *const c_char))
                };
            }

            let x_open_display = sym!("XOpenDisplay");
            let x_close_display = sym!("XCloseDisplay");
            let x_default_root_window = sym!("XDefaultRootWindow");
            let x_grab_keyboard = sym!("XGrabKeyboard");
            let x_ungrab_keyboard = sym!("XUngrabKeyboard");
            let x_pending = sym!("XPending");
            let x_next_event = sym!("XNextEvent");
            let x_lookup_string = sym!("XLookupString");
            let x_utf8_lookup_string = dlsym(handle, b"Xutf8LookupString\0".as_ptr() as *const c_char);
            let x_utf8_lookup_string = if x_utf8_lookup_string.is_null() {
                None
            } else {
                Some(std::mem::transmute(x_utf8_lookup_string))
            };
            let x_flush = sym!("XFlush");

            Ok(Self {
                handle,
                x_open_display,
                x_close_display,
                x_default_root_window,
                x_grab_keyboard,
                x_ungrab_keyboard,
                x_pending,
                x_next_event,
                x_lookup_string,
                x_utf8_lookup_string,
                x_flush,
            })
        }
    }
}

impl Drop for X11Lib {
    fn drop(&mut self) {
        unsafe {
            dlclose(self.handle);
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
struct XKeyEvent {
    _type: c_int,
    _serial: u64,
    _send_event: c_int,
    _display: *mut Display,
    window: Window,
    _root: Window,
    _subwindow: Window,
    _time: Time,
    _x: c_int,
    _y: c_int,
    _x_root: c_int,
    _y_root: c_int,
    state: c_int,
    keycode: u32,
    _same_screen: c_int,
}

#[repr(C)]
union XEventData {
    key: XKeyEvent,
}

#[repr(C)]
struct XEvent {
    _type: c_int,
    _pad: [u8; 24],
    data: XEventData,
}

type KeySym = u64;

pub struct X11KeyEvent {
    pub keycode: u32,
    pub ch: Option<char>,
    pub pressed: bool,
    pub state: c_int,
}

pub struct X11Capture {
    lib: X11Lib,
    display: *mut Display,
    root: Window,
    grabbed: bool,
    /// Set to true when FocusOut is received — caller should reset engine state
    pub focus_lost: bool,
}

unsafe impl Send for X11Capture {}

impl X11Capture {
    pub fn new() -> Option<Self> {
        let lib = match X11Lib::new() {
            Ok(lib) => lib,
            Err(e) => {
                eprintln!("[vietc] X11Capture: failed to load X11: {}", e);
                return None;
            }
        };

        unsafe {
            let display = (lib.x_open_display)(std::ptr::null());
            if display.is_null() {
                eprintln!("[vietc] X11Capture: cannot open display. Is DISPLAY set?");
                return None;
            }

            let root = (lib.x_default_root_window)(display);
            eprintln!("[vietc] X11Capture: initialized successfully");
            Some(Self {
                lib,
                display,
                root,
                grabbed: false,
                focus_lost: false,
            })
        }
    }

    pub fn grab_keyboard(&mut self) -> bool {
        unsafe {
            let status = (self.lib.x_grab_keyboard)(
                self.display,
                self.root,
                0, // owner_events = False
                GRAB_MODE_ASYNC,
                GRAB_MODE_ASYNC,
                0, // CurrentTime
            ) as i32;
            if status == 0 {
                self.grabbed = true;
                eprintln!("[vietc] X11Capture: grabbed keyboard successfully");
                true
            } else {
                eprintln!("[vietc] X11Capture: grab failed with status {}", status);
                false
            }
        }
    }

    pub fn ungrab_keyboard(&mut self) {
        if self.grabbed {
            unsafe {
                (self.lib.x_ungrab_keyboard)(self.display, 0);
                (self.lib.x_flush)(self.display);
            }
            self.grabbed = false;
        }
    }

    pub fn has_pending_events(&self) -> bool {
        if !self.grabbed {
            return false;
        }
        unsafe { (self.lib.x_pending)(self.display) > 0 }
    }

    pub fn is_grabbed(&self) -> bool {
        self.grabbed
    }

    pub fn next_event(&mut self) -> Option<X11KeyEvent> {
        if !self.grabbed {
            return None;
        }

        // Non-blocking: only read if events are pending
        if !self.has_pending_events() {
            return None;
        }

        let mut event: XEvent = unsafe { std::mem::zeroed() };
        unsafe {
            (self.lib.x_next_event)(self.display, &mut event);
        }

        let _type = event._type;

        // Handle FocusIn/FocusOut — reset engine state when focus changes
        if _type == FOCUS_OUT {
            self.focus_lost = true;
            return self.next_event();
        }
        if _type == FOCUS_IN {
            self.focus_lost = false;
            return self.next_event();
        }

        if _type != KEY_PRESS && _type != KEY_RELEASE {
            return self.next_event();
        }

        let key_event = unsafe { &event.data.key };
        let ch = self.lookup_key(key_event);
        Some(X11KeyEvent {
            keycode: key_event.keycode,
            ch,
            pressed: _type == KEY_PRESS,
            state: key_event.state,
        })
    }

    pub fn is_modifier_pressed(&self, state: c_int) -> bool {
        (state & (CONTROL_MASK | MOD1_MASK | MOD4_MASK)) != 0
    }

    pub fn with_grab<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        // Grab should already be held; just execute
        f()
    }

    pub fn without_grab<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.ungrab_keyboard();
        let result = f();
        self.grab_keyboard();
        result
    }

    fn lookup_key(&self, event: &XKeyEvent) -> Option<char> {
        let mut buf = [0u8; 32];
        let mut keysym: KeySym = 0;
        let len = unsafe {
            if let Some(xutf8) = self.lib.x_utf8_lookup_string {
                xutf8(
                    event as *const XKeyEvent as *mut XKeyEvent,
                    buf.as_mut_ptr() as *mut c_char,
                    buf.len() as c_int,
                    &mut keysym as *mut KeySym,
                    std::ptr::null_mut(),
                )
            } else {
                (self.lib.x_lookup_string)(
                    event as *const XKeyEvent as *mut XKeyEvent,
                    buf.as_mut_ptr() as *mut c_char,
                    buf.len() as c_int,
                    &mut keysym as *mut KeySym,
                    std::ptr::null_mut(),
                )
            }
        };

        if len > 0 {
            let s = std::str::from_utf8(&buf[..len as usize]).ok()?;
            s.chars().next()
        } else {
            None
        }
    }
}

impl Drop for X11Capture {
    fn drop(&mut self) {
        if self.grabbed {
            self.ungrab_keyboard();
        }
        unsafe {
            (self.lib.x_close_display)(self.display);
        }
    }
}
