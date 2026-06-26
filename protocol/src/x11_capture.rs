use std::ffi::{c_char, c_int, c_void};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};

type Display = c_void;
type Window = u64;
type Time = u64;

// X11 event types
const KEY_PRESS: c_int = 2;
const KEY_RELEASE: c_int = 3;

// X11 modifier masks
const CONTROL_MASK: c_int = 4;
const MOD1_MASK: c_int = 8;
const MOD4_MASK: c_int = 64;

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
    x_lookup_string: unsafe extern "C" fn(*mut XKeyEvent, *mut c_char, c_int, *mut KeySym, *mut c_int) -> c_int,
    x_utf8_lookup_string: Option<unsafe extern "C" fn(*mut XKeyEvent, *mut c_char, c_int, *mut KeySym, *mut c_int) -> c_int>,
    x_flush: unsafe extern "C" fn(*mut Display) -> c_int,
    x_connection_number: unsafe extern "C" fn(*mut Display) -> c_int,
    // XRecord
    x_record_query_version: unsafe extern "C" fn(*mut Display, *mut c_int, *mut c_int) -> i32,
    x_record_alloc_range: unsafe extern "C" fn() -> *mut XRecordRange,
    x_record_create_context: unsafe extern "C" fn(*mut Display, c_int, *mut c_int, c_int, *mut *mut XRecordRange, c_int) -> u64,
    x_record_enable_context_async: unsafe extern "C" fn(*mut Display, u64, Option<XRecordCallback>, *mut c_void) -> i32,
    x_record_process_replies: unsafe extern "C" fn(*mut Display),
    x_record_disable_context: unsafe extern "C" fn(*mut Display, u64) -> i32,
    x_record_free_context: unsafe extern "C" fn(*mut Display, u64) -> i32,
    x_free: unsafe extern "C" fn(*mut c_void) -> c_int,
}

// XRecordRange: 32 bytes total
// device_events is at offset 18 (XRecordRange8: first=offset 18, last=offset 19)
#[repr(C)]
struct XRecordRange {
    _bytes: [u8; 32],
}

type XRecordCallback = unsafe extern "C" fn(*mut c_void, *mut XRecordInterceptData);

#[repr(C)]
struct XRecordInterceptData {
    id: u64,
    server_time: u64,
    client_swapped: c_int,
    _pad: c_int,
    data_len: c_int,
    data: *mut u8,
}

#[repr(C)]
struct Timeval {
    tv_sec: i64,
    tv_usec: i64,
}

#[repr(C)]
struct FdSet {
    fds_bits: [u64; 16],
}

extern "C" {
    fn select(nfds: c_int, readfds: *mut FdSet, writefds: *mut FdSet, exceptfds: *mut FdSet, timeout: *mut Timeval) -> c_int;
}

fn fd_zero(set: &mut FdSet) {
    set.fds_bits = [0u64; 16];
}

fn fd_set_bit(fd: c_int, set: &mut FdSet) {
    let idx = fd as usize / 64;
    let bit = fd as usize % 64;
    if idx < set.fds_bits.len() {
        set.fds_bits[idx] |= 1u64 << bit;
    }
}

fn fd_isset(fd: c_int, set: &FdSet) -> bool {
    let idx = fd as usize / 64;
    let bit = fd as usize % 64;
    if idx < set.fds_bits.len() {
        (set.fds_bits[idx] & (1u64 << bit)) != 0
    } else {
        false
    }
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

            // libXtst.so.6 for XRecord
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

            let x_open_display = sym!("XOpenDisplay");
            let x_close_display = sym!("XCloseDisplay");
            let x_default_root_window = sym!("XDefaultRootWindow");
            let x_grab_keyboard = sym!("XGrabKeyboard");
            let x_ungrab_keyboard = sym!("XUngrabKeyboard");
            let x_lookup_string = sym!("XLookupString");
            let x_utf8_lookup_string = dlsym(handle, b"Xutf8LookupString\0".as_ptr() as *const c_char);
            let x_utf8_lookup_string = if x_utf8_lookup_string.is_null() {
                None
            } else {
                Some(std::mem::transmute(x_utf8_lookup_string))
            };
            let x_flush = sym!("XFlush");
            let x_connection_number = sym!("XConnectionNumber");

            if xtst_handle.is_null() {
                return Err("Failed to load libXtst.so.6 — install libxtst6".into());
            }

            macro_rules! xtst_sym {
                ($name:expr) => {
                    std::mem::transmute(dlsym(xtst_handle, concat!($name, "\0").as_ptr() as *const c_char))
                };
            }

            let x_record_query_version = xtst_sym!("XRecordQueryVersion");
            let x_record_alloc_range = xtst_sym!("XRecordAllocRange");
            let x_record_create_context = xtst_sym!("XRecordCreateContext");
            let x_record_enable_context_async = xtst_sym!("XRecordEnableContextAsync");
            let x_record_process_replies = xtst_sym!("XRecordProcessReplies");
            let x_record_disable_context = xtst_sym!("XRecordDisableContext");
            let x_record_free_context = xtst_sym!("XRecordFreeContext");
            let x_free = sym!("XFree");

            Ok(Self {
                handle,
                x_open_display,
                x_close_display,
                x_default_root_window,
                x_grab_keyboard,
                x_ungrab_keyboard,
                x_lookup_string,
                x_utf8_lookup_string,
                x_flush,
                x_connection_number,
                x_record_query_version,
                x_record_alloc_range,
                x_record_create_context,
                x_record_enable_context_async,
                x_record_process_replies,
                x_record_disable_context,
                x_record_free_context,
                x_free,
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

type KeySym = u64;

pub struct X11KeyEvent {
    pub keycode: u32,
    pub ch: Option<char>,
    pub pressed: bool,
    pub state: c_int,
}

// Shared event queue between XRecord callback and capture reader
struct EventQueue {
    queue: VecDeque<X11KeyEvent>,
}

static mut EVENT_QUEUE: Option<Arc<Mutex<EventQueue>>> = None;

unsafe extern "C" fn record_callback(_closure: *mut c_void, data: *mut XRecordInterceptData) {
    if data.is_null() {
        return;
    }
    let data_len = (*data).data_len;
    if data_len < 2 {
        return;
    }
    let data_bytes = (*data).data;
    if data_bytes.is_null() {
        return;
    }
    let event_type: c_int = *data_bytes as c_int;
    let keycode: u8 = *data_bytes.add(1);

    if event_type != KEY_PRESS && event_type != KEY_RELEASE {
        return;
    }

    // XRecord data layout for keyboard events: type(1) + keycode(1) + state(2)
    let state: c_int = if data_len >= 4 {
        *(data_bytes.add(2) as *const u16) as c_int
    } else {
        0
    };

    let event = X11KeyEvent {
        keycode: keycode as u32,
        ch: None, // Will be resolved later via XLookupString or keysym mapping
        pressed: event_type == KEY_PRESS,
        state,
    };

    if let Some(ref q) = EVENT_QUEUE {
        if let Ok(mut queue) = q.lock() {
            queue.queue.push_back(event);
        }
    }
}

pub struct X11Capture {
    lib: X11Lib,
    display: *mut Display,
    root: Window,
    grabbed: bool,
    record_context: u64,
    record_display: *mut Display,
    pub focus_lost: bool,
}

unsafe impl Send for X11Capture {}

impl X11Capture {
    pub fn new() -> Option<Self> {
        let lib = match X11Lib::new() {
            Ok(lib) => lib,
            Err(e) => {
                eprintln!("[vietc] X11Capture: failed to load: {}", e);
                return None;
            }
        };

        unsafe {
            let display = (lib.x_open_display)(std::ptr::null());
            if display.is_null() {
                eprintln!("[vietc] X11Capture: cannot open display");
                return None;
            }

            let root = (lib.x_default_root_window)(display);

            // Check XRecord version
            let mut major = 0i32;
            let mut minor = 0i32;
            if (lib.x_record_query_version)(display, &mut major, &mut minor) == 0 {
                eprintln!("[vietc] X11Capture: XRecord extension not available");
                (lib.x_close_display)(display);
                return None;
            }
            eprintln!("[vietc] X11Capture: XRecord version {}.{}", major, minor);

            // Allocate range for keyboard events
            let range = (lib.x_record_alloc_range)();
            if range.is_null() {
                eprintln!("[vietc] X11Capture: XRecordAllocRange failed");
                (lib.x_close_display)(display);
                return None;
            }
            // Set range: device_events at offset 18
            // XRecordRange8: first byte, last byte
            (*range)._bytes[18] = KEY_PRESS as u8;  // device_events.first
            (*range)._bytes[19] = KEY_RELEASE as u8; // device_events.last
            eprintln!("[vietc] X11Capture: range set (KeyPress={}, KeyRelease={})", KEY_PRESS, KEY_RELEASE);

            // Create XRecord context
            // XRecordClientSpec is XID = unsigned long (8 bytes on x86_64)
            let mut spec: u64 = 3; // XRecordAllClients = 3
            let mut range_ptr: *mut XRecordRange = range;
            let ctx = (lib.x_record_create_context)(
                display,
                0,              // own_client
                &mut spec as *mut u64 as *mut c_int, // client_spec (pointer to unsigned long)
                1,              // nclients
                &mut range_ptr, // ranges (pointer to array of range pointers)
                1,              // nranges
            );
            (lib.x_free)(range as *mut c_void);

            if ctx == 0 {
                eprintln!("[vietc] X11Capture: XRecordCreateContext failed");
                (lib.x_close_display)(display);
                return None;
            }
            eprintln!("[vietc] X11Capture: XRecord context created (ctx={})", ctx);

            // Initialize event queue
            EVENT_QUEUE = Some(Arc::new(Mutex::new(EventQueue {
                queue: VecDeque::new(),
            })));

            // Enable XRecord with async callback
            let closure: *mut c_void = std::ptr::null_mut();
            (lib.x_record_enable_context_async)(display, ctx, Some(record_callback), closure);
            (lib.x_flush)(display);

            eprintln!("[vietc] X11Capture: XRecord context enabled — capturing keyboard events");

            Some(Self {
                lib,
                display,
                root,
                grabbed: false,
                record_context: ctx,
                record_display: display,
                focus_lost: false,
            })
        }
    }

    pub fn grab_keyboard(&mut self) -> bool {
        unsafe {
            let status = (self.lib.x_grab_keyboard)(
                self.display,
                self.root,
                0, // owner_events = False — block events from reaching apps
                GRAB_MODE_ASYNC,
                GRAB_MODE_ASYNC,
                0,
            ) as i32;
            if status == 0 {
                self.grabbed = true;
                (self.lib.x_flush)(self.display);
                eprintln!("[vietc] X11Capture: keyboard grabbed (blocking apps)");
                true
            } else {
                eprintln!("[vietc] X11Capture: grab failed status={}", status);
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

    pub fn is_grabbed(&self) -> bool {
        self.grabbed
    }

    /// Wait for XRecord data to arrive on the X11 connection fd, with timeout.
    pub fn wait_for_event(&mut self, timeout_ms: u64) -> bool {
        unsafe {
            (self.lib.x_flush)(self.display);

            let fd = (self.lib.x_connection_number)(self.display);
            let mut readfds: FdSet = std::mem::zeroed();
            fd_zero(&mut readfds);
            fd_set_bit(fd, &mut readfds);
            let mut timeout = Timeval {
                tv_sec: (timeout_ms / 1000) as i64,
                tv_usec: ((timeout_ms % 1000) * 1000) as i64,
            };
            let n = select(fd + 1, &mut readfds, std::ptr::null_mut(), std::ptr::null_mut(), &mut timeout);
            if n > 0 && fd_isset(fd, &readfds) {
                // Process XRecord replies — this fires the callback
                (self.lib.x_record_process_replies)(self.display);
                true
            } else {
                false
            }
        }
    }

    pub fn next_event(&mut self) -> Option<X11KeyEvent> {
        unsafe {
            if let Some(ref q) = EVENT_QUEUE {
                if let Ok(mut queue) = q.lock() {
                    if let Some(mut event) = queue.queue.pop_front() {
                        // Resolve the character from the keycode + modifier state
                        event.ch = self.lookup_keycode(event.keycode, event.state);
                        return Some(event);
                    }
                }
            }
        }
        None
    }

    pub fn is_modifier_pressed(&self, state: c_int) -> bool {
        (state & (CONTROL_MASK | MOD1_MASK | MOD4_MASK)) != 0
    }

    pub fn with_grab<F, T>(&mut self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
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

    pub fn lookup_keycode(&self, keycode: u32, state: c_int) -> Option<char> {
        // Construct a fake XKeyEvent for XLookupString
        let mut xke: XKeyEvent = unsafe { std::mem::zeroed() };
        xke._type = KEY_PRESS;
        xke.keycode = keycode;
        xke.state = state;

        let mut buf = [0u8; 32];
        let mut keysym: KeySym = 0;
        let len = unsafe {
            if let Some(xutf8) = self.lib.x_utf8_lookup_string {
                xutf8(
                    &mut xke as *mut XKeyEvent,
                    buf.as_mut_ptr() as *mut c_char,
                    buf.len() as c_int,
                    &mut keysym as *mut KeySym,
                    std::ptr::null_mut(),
                )
            } else {
                (self.lib.x_lookup_string)(
                    &mut xke as *mut XKeyEvent,
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
        unsafe {
            if self.grabbed {
                self.ungrab_keyboard();
            }
            (self.lib.x_record_disable_context)(self.record_display, self.record_context);
            (self.lib.x_record_free_context)(self.record_display, self.record_context);
            (self.lib.x_close_display)(self.display);
        }
    }
}
