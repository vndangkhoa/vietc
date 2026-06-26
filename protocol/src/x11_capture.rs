use std::collections::VecDeque;
use std::ffi::{c_char, c_int, c_void};
use std::io::{Read, BufRead};
use std::os::unix::io::AsRawFd;
use std::process::{Command, Child, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};

type Display = c_void;

const KEY_PRESS: c_int = 2;

const CONTROL_MASK: c_int = 4;
const MOD1_MASK: c_int = 8;
const MOD4_MASK: c_int = 64;

extern "C" {
    fn dlopen(filename: *const c_char, flag: c_int) -> *mut c_void;
    fn dlsym(handle: *mut c_void, symbol: *const c_char) -> *mut c_void;
    fn dlclose(handle: *mut c_void) -> c_int;
    fn poll(fds: *mut PollFd, nfds: u64, timeout: i32) -> i32;
}

#[repr(C)]
struct PollFd {
    fd: i32,
    events: i16,
    revents: i16,
}

const POLLIN: i16 = 1;

#[repr(C)]
struct XKeyEvent {
    _type: c_int,
    _serial: u64,
    _send_event: c_int,
    _display: *mut Display,
    window: u64,
    _root: u64,
    _subwindow: u64,
    _time: u64,
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

pub static SKIP_RECORD_EVENTS: AtomicBool = AtomicBool::new(false);

struct LookupLib {
    handle: *mut c_void,
    display: *mut Display,
    x_close_display: unsafe extern "C" fn(*mut Display) -> c_int,
    x_lookup_string: unsafe extern "C" fn(*mut XKeyEvent, *mut c_char, c_int, *mut KeySym, *mut c_int) -> c_int,
    x_utf8_lookup_string: Option<unsafe extern "C" fn(*mut c_void, *mut XKeyEvent, *mut c_char, c_int, *mut KeySym, *mut c_int) -> c_int>,
}

unsafe impl Send for LookupLib {}

impl Drop for LookupLib {
    fn drop(&mut self) {
        unsafe {
            (self.x_close_display)(self.display);
            dlclose(self.handle);
        }
    }
}

impl LookupLib {
    fn new() -> Option<Self> {
        unsafe {
            let paths = [
                b"libX11.so.6\0".as_ptr() as *const c_char,
                b"libX11.so\0".as_ptr() as *const c_char,
            ];
            let mut handle = std::ptr::null_mut();
            for path in paths {
                handle = dlopen(path, 1);
                if !handle.is_null() { break; }
            }
            if handle.is_null() { return None; }

            macro_rules! sym {
                ($name:expr) => {
                    std::mem::transmute(dlsym(handle, concat!($name, "\0").as_ptr() as *const c_char))
                };
            }

            let x_open_display: unsafe extern "C" fn(*const c_char) -> *mut Display = sym!("XOpenDisplay");
            let display = x_open_display(std::ptr::null());
            if display.is_null() {
                dlclose(handle);
                return None;
            }

            Some(Self {
                handle,
                display,
                x_close_display: sym!("XCloseDisplay"),
                x_lookup_string: sym!("XLookupString"),
                x_utf8_lookup_string: {
                    let p = dlsym(handle, b"Xutf8LookupString\0".as_ptr() as *const c_char);
                    if p.is_null() { None } else { Some(std::mem::transmute(p)) }
                },
            })
        }
    }

    fn lookup_keycode(&self, keycode: u32, state: c_int) -> Option<char> {
        unsafe {
            let mut xke: XKeyEvent = std::mem::zeroed();
            xke._type = KEY_PRESS;
            xke.keycode = keycode;
            xke.state = state;

            let mut buf = [0u8; 32];
            let mut keysym: KeySym = 0;
            let len = if let Some(xutf8) = self.x_utf8_lookup_string {
                xutf8(
                    std::ptr::null_mut(),
                    &mut xke as *mut XKeyEvent,
                    buf.as_mut_ptr() as *mut c_char,
                    buf.len() as c_int,
                    &mut keysym,
                    std::ptr::null_mut(),
                )
            } else {
                (self.x_lookup_string)(
                    &mut xke as *mut XKeyEvent,
                    buf.as_mut_ptr() as *mut c_char,
                    buf.len() as c_int,
                    &mut keysym,
                    std::ptr::null_mut(),
                )
            };

            if len > 0 {
                let s = std::str::from_utf8(&buf[..len as usize]).ok()?;
                s.chars().next()
            } else {
                None
            }
        }
    }
}

/// Pipe event from C helper: 8 bytes, packed.
#[repr(C, packed)]
#[derive(Clone, Copy)]
struct PipeEvent {
    keycode: u8,
    pressed: u8,
    state: u16,
    _padding: [u8; 4],
}

pub struct X11Capture {
    child: Child,
    pipe_fd: i32,
    pipe_stdout: Option<std::process::ChildStdout>,
    lookup: LookupLib,
    event_queue: VecDeque<X11KeyEvent>,
    pub focus_lost: bool,
}

unsafe impl Send for X11Capture {}

impl X11Capture {
    pub fn new() -> Option<Self> {
        let lookup = LookupLib::new();
        let xrecord_path = find_xrecord_binary();
        eprintln!("[vietc] X11Capture: spawning vietc-xrecord from {}", xrecord_path);

        let mut child = Command::new(&xrecord_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .ok()?;

        // Wait for "ready" on stderr
        {
            let stderr = child.stderr.take()?;
            let mut reader = std::io::BufReader::new(stderr);
            let mut line = String::new();
            reader.read_line(&mut line).ok()?;
            eprintln!("[vietc] vietc-xrecord: {}", line.trim());
        }

        let stdout = child.stdout.take()?;
        let pipe_fd = stdout.as_raw_fd();

        // Set pipe to non-blocking
        unsafe {
            let flags = libc::fcntl(pipe_fd, libc::F_GETFL);
            libc::fcntl(pipe_fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }

        let lookup = lookup?;

        Some(Self {
            child,
            pipe_fd,
            pipe_stdout: Some(stdout),
            lookup,
            event_queue: VecDeque::new(),
            focus_lost: false,
        })
    }

    pub fn grab_keyboard(&mut self) -> bool { false }
    pub fn ungrab_keyboard(&mut self) {}
    pub fn is_grabbed(&self) -> bool { false }

    /// Wait for events from the C helper pipe with timeout.
    pub fn wait_for_event(&mut self, timeout_ms: u64) -> bool {
        // If SKIP_RECORD_EVENTS is true, aggressively drain all pending events
        if SKIP_RECORD_EVENTS.load(Ordering::Relaxed) {
            let deadline = std::time::Instant::now() + std::time::Duration::from_millis(50);
            loop {
                self.drain_pipe();
                if std::time::Instant::now() >= deadline {
                    break;
                }
                let mut pfd = PollFd {
                    fd: self.pipe_fd,
                    events: POLLIN,
                    revents: 0,
                };
                unsafe {
                    poll(&mut pfd, 1, 5);
                }
                if pfd.revents & POLLIN == 0 {
                    std::thread::sleep(std::time::Duration::from_micros(500));
                    self.drain_pipe();
                    break;
                }
            }
        }

        self.drain_pipe();

        if !self.event_queue.is_empty() {
            return true;
        }

        let mut pfd = PollFd {
            fd: self.pipe_fd,
            events: POLLIN,
            revents: 0,
        };
        unsafe {
            poll(&mut pfd, 1, timeout_ms as i32);
        }

        if pfd.revents & POLLIN != 0 {
            self.drain_pipe();
        }

        if let Ok(None) = self.child.try_wait() {
        } else {
            self.restart_xrecord();
        }

        !self.event_queue.is_empty()
    }

    fn drain_pipe(&mut self) {
        if let Some(ref mut stdout) = self.pipe_stdout {
            let mut buf = [0u8; 8];
            let mut filled = 0usize;
            loop {
                match stdout.read(&mut buf[filled..]) {
                    Ok(0) => break,
                    Ok(n) => {
                        filled += n;
                        while filled >= 8 {
                            let ev: PipeEvent = unsafe { std::mem::transmute(buf) };
                            if SKIP_RECORD_EVENTS.load(Ordering::Relaxed) {
                                if ev.keycode == 0 && ev.state == 2 {
                                    self.focus_lost = true;
                                }
                            } else if ev.keycode == 0 && ev.pressed == 0 {
                                if ev.state == 2 {
                                    self.focus_lost = true;
                                }
                            } else {
                                let event = X11KeyEvent {
                                    keycode: ev.keycode as u32,
                                    ch: None,
                                    pressed: ev.pressed == 1,
                                    state: ev.state as c_int,
                                };
                                self.event_queue.push_back(event);
                            }
                            filled -= 8;
                            if filled > 0 {
                                buf.copy_within(8..8 + filled, 0);
                            }
                        }
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                    Err(_) => break,
                }
            }
        }
    }

    fn restart_xrecord(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.pipe_stdout = None;

        let xrecord_path = find_xrecord_binary();
        if let Ok(mut child) = Command::new(&xrecord_path)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
        {
            if let Some(stderr) = child.stderr.take() {
                let mut reader = std::io::BufReader::new(stderr);
                let mut line = String::new();
                let _ = reader.read_line(&mut line);
                eprintln!("[vietc] vietc-xrecord restarted: {}", line.trim());
            }
            if let Some(stdout) = child.stdout.take() {
                let fd = stdout.as_raw_fd();
                unsafe {
                    let flags = libc::fcntl(fd, libc::F_GETFL);
                    libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
                }
                self.pipe_fd = fd;
                self.pipe_stdout = Some(stdout);
            }
            self.child = child;
        }
    }

    pub fn next_event(&mut self) -> Option<X11KeyEvent> {
        if let Some(mut event) = self.event_queue.pop_front() {
            event.ch = self.lookup.lookup_keycode(event.keycode, event.state);
            Some(event)
        } else {
            None
        }
    }

    pub fn is_modifier_pressed(&self, state: c_int) -> bool {
        (state & (CONTROL_MASK | MOD1_MASK | MOD4_MASK)) != 0
    }

    /// Drain any pending events from the pipe without adding them to the queue.
    /// Used after injection to clear feedback events while SKIP_RECORD_EVENTS is set.
    pub fn drain_injected(&mut self) {
        self.drain_pipe();
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
        f()
    }
}

impl Drop for X11Capture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn find_xrecord_binary() -> String {
    if let Ok(output) = std::process::Command::new("which").arg("vietc-xrecord").output() {
        if output.status.success() {
            return String::from_utf8_lossy(&output.stdout).trim().to_string();
        }
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            let path = dir.join("vietc-xrecord");
            if path.exists() {
                return path.to_string_lossy().to_string();
            }
        }
    }

    for p in &["/usr/bin/vietc-xrecord", "/usr/local/bin/vietc-xrecord"] {
        if std::path::Path::new(p).exists() {
            return p.to_string();
        }
    }

    "vietc-xrecord".to_string()
}
