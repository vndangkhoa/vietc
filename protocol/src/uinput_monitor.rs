use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;

use super::inject::{InjectResult, KeyInjector};

const UINPUT_MAX_NAME_SIZE: usize = 80;
const UI_SET_EVBIT: u64 = 0x40045564;
const UI_SET_KEYBIT: u64 = 0x40045565;
#[allow(dead_code)]
const UI_SET_ABSBIT: u64 = 0x40045566;
const UI_DEV_CREATE: u64 = 0x5501;
const UI_DEV_DESTROY: u64 = 0x5502;
const UI_DEV_SETUP: u64 = 0x405c5503;
const EV_KEY: u16 = 0x01;
#[allow(dead_code)]
const EV_ABS: u16 = 0x03;
const KEY_MAX: u32 = 0x1ff;

pub struct UinputInjector {
    file: File,
}

unsafe impl Send for UinputInjector {}
unsafe impl Sync for UinputInjector {}

impl UinputInjector {
    fn send_enter(&self) {
        self.send_uinput_event(EV_KEY, 28, 1);
        self.send_uinput_event(0, 0, 0);
        std::thread::sleep(std::time::Duration::from_millis(2));
        self.send_uinput_event(EV_KEY, 28, 0);
        self.send_uinput_event(0, 0, 0);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

    pub fn new(name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/uinput")?;

        let fd = file.as_raw_fd();

        // Enable EV_KEY
        ioctl(fd, UI_SET_EVBIT, EV_KEY as u64)
            .map_err(|e| format!("UI_SET_EVBIT failed: {}", e))?;

        // Enable all key codes we'll need
        for code in 0..=KEY_MAX {
            ioctl(fd, UI_SET_KEYBIT, code as u64)
                .map_err(|e| format!("UI_SET_KEYBIT {} failed: {}", code, e))?;
        }

        // Create uinput device
        let mut usetup: uinput_setup = unsafe { std::mem::zeroed() };
        let name_bytes = name.as_bytes();
        let copy_len = name_bytes.len().min(UINPUT_MAX_NAME_SIZE - 1);
        for (i, &byte) in name_bytes.iter().enumerate().take(copy_len) {
            usetup.name[i] = byte as i8;
        }
        usetup.name[copy_len] = 0;
        usetup.id.bustype = 0x03; // BUS_USB
        usetup.id.vendor = 0x1234;
        usetup.id.product = 0x5678;
        usetup.id.version = 1;

        ioctl(fd, UI_DEV_SETUP, &usetup as *const uinput_setup as u64)
            .map_err(|e| format!("UI_DEV_SETUP failed: {}", e))?;

        ioctl(fd, UI_DEV_CREATE, 0).map_err(|e| format!("UI_DEV_CREATE failed: {}", e))?;

        // Small delay for device to be ready
        std::thread::sleep(std::time::Duration::from_millis(10));

        Ok(Self { file })
    }

    fn send_uinput_event(&self, type_: u16, code: u16, value: i32) {
        let event = input_event {
            time: timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
            type_,
            code,
            value,
        };

        unsafe {
            let ptr = &event as *const input_event as *const u8;
            let len = std::mem::size_of::<input_event>();
            let _ = libc::write(
                self.file.as_raw_fd() as libc::c_int,
                ptr as *const libc::c_void,
                len,
            );
        }
    }

    fn send_key_stroke(&self, keycode: u16, shift: bool) {
        if shift {
            self.send_uinput_event(EV_KEY, 42, 1); // Shift press
            self.send_uinput_event(0, 0, 0); // SYN
            std::thread::sleep(std::time::Duration::from_millis(2));
        }

        self.send_uinput_event(EV_KEY, keycode, 1); // Key press
        self.send_uinput_event(0, 0, 0); // SYN
        std::thread::sleep(std::time::Duration::from_millis(2));

        self.send_uinput_event(EV_KEY, keycode, 0); // Key release
        self.send_uinput_event(0, 0, 0); // SYN
        std::thread::sleep(std::time::Duration::from_millis(2));

        if shift {
            self.send_uinput_event(EV_KEY, 42, 0); // Shift release
            self.send_uinput_event(0, 0, 0); // SYN
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }
}

impl KeyInjector for UinputInjector {
    fn send_backspace(&self) -> InjectResult {
        self.send_uinput_event(EV_KEY, 14, 1); // KEY_BACKSPACE press
        self.send_uinput_event(0, 0, 0); // SYN
        std::thread::sleep(std::time::Duration::from_millis(2));

        self.send_uinput_event(EV_KEY, 14, 0); // KEY_BACKSPACE release
        self.send_uinput_event(0, 0, 0); // SYN
        std::thread::sleep(std::time::Duration::from_millis(2));

        InjectResult::Success
    }

    fn send_key_event(&self, keycode: u16, value: i32) -> InjectResult {
        self.send_uinput_event(EV_KEY, keycode, value);
        self.send_uinput_event(0, 0, 0);
        std::thread::sleep(std::time::Duration::from_millis(2));
        InjectResult::Success
    }

    fn send_char(&self, ch: char) -> InjectResult {
        if let Some(keycode) = char_to_linux_keycode(ch) {
            let needs_shift = ch.is_uppercase() || "!@#$%^&*()_+{}|:\"<>?".contains(ch);
            self.send_key_stroke(keycode, needs_shift);
            return InjectResult::Success;
        }
        // Vietnamese Unicode char: map to base ASCII and send via uinput
        let ascii = strip_vn_diacritic(ch);
        if let Some(keycode) = char_to_linux_keycode(ascii) {
            let needs_shift = ascii.is_uppercase();
            self.send_key_stroke(keycode, needs_shift);
        }
        InjectResult::Success
    }

    fn send_string(&self, s: &str) -> InjectResult {
        // ASCII characters: inject directly via uinput keycodes
        let is_ascii = s.chars().all(|c| char_to_linux_keycode(c).is_some());
        eprintln!(
            "[vietc] send_string: len={}, is_ascii={}",
            s.len(),
            is_ascii
        );

        if is_ascii {
            eprintln!(
                "[vietc] send_string: ASCII '{}' via uinput",
                s.escape_default()
            );
            for ch in s.chars() {
                self.send_char(ch);
            }
            return InjectResult::Success;
        }

        // Unicode text: single clipboard copy + paste (reliable method)
        eprintln!(
            "[vietc] send_string: Unicode '{}' - using clipboard",
            s.escape_default()
        );
        let copied = self.copy_to_clipboard(s);
        if copied {
            eprintln!("[vietc] send_string: clipboard OK, sending Ctrl+V");
            self.send_ctrl_v();
            eprintln!("[vietc] send_string complete (clipboard)");
            return InjectResult::Success;
        } else {
            eprintln!(
                "[vietc] send_string failed for '{}' (clipboard unavailable)",
                s.escape_default()
            );
            // Last resort: try paste_string (will try clipboard internally)
            self.paste_string(s);
        }
        InjectResult::Success
    }

    fn inject_replacement(&self, backspaces: usize, text: &str) -> InjectResult {
        self.inject_replacement_atomic(backspaces, text)
    }
    fn flush(&self) -> InjectResult {
        InjectResult::Success
    }

    /// Record that Unicode text was pasted via clipboard (for future delete/backspace support)
    fn update_pasted_text(&self, text: &str) -> InjectResult {
        // Text tracking happens through OutputCommand pipeline in daemon
        // This is called after clipboard paste to inform engine of pasted content
        eprintln!(
            "[vietc] update_pasted_text: recorded '{}' (len={})",
            text.escape_default(),
            text.len()
        );
        InjectResult::Success
    }
}

impl UinputInjector {
    /// Get the original non-root username when running as root.
    /// Checks SUDO_USER (sudo), PKEXEC_UID (pkexec), /proc/self/loginuid,
    /// and falls back to `logname`.
    fn get_original_username() -> Option<String> {
        let is_root = unsafe { libc::getuid() == 0 };
        if !is_root {
            return None;
        }

        if let Ok(user) = std::env::var("SUDO_USER") {
            if !user.is_empty() {
                return Some(user);
            }
        }

        if let Ok(uid_str) = std::env::var("PKEXEC_UID") {
            if let Ok(uid) = uid_str.parse::<u32>() {
                unsafe {
                    let pw = libc::getpwuid(uid);
                    if !pw.is_null() {
                        let name = std::ffi::CStr::from_ptr((*pw).pw_name)
                            .to_string_lossy()
                            .into_owned();
                        if !name.is_empty() {
                            return Some(name);
                        }
                    }
                }
            }
        }

        if let Ok(content) = std::fs::read_to_string("/proc/self/loginuid") {
            if let Ok(uid) = content.trim().parse::<u32>() {
                if uid != 4294967295 {
                    unsafe {
                        let pw = libc::getpwuid(uid);
                        if !pw.is_null() {
                            let name = std::ffi::CStr::from_ptr((*pw).pw_name)
                                .to_string_lossy()
                                .into_owned();
                            if !name.is_empty() {
                                return Some(name);
                            }
                        }
                    }
                }
            }
        }

        if let Ok(output) = std::process::Command::new("logname").output() {
            if output.status.success() {
                let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !name.is_empty() {
                    return Some(name);
                }
            }
        }

        None
    }

    /// Get original non-root UID and GID when running as root.
    fn get_original_uid_gid() -> Option<(u32, u32)> {
        let is_root = unsafe { libc::getuid() == 0 };
        if !is_root {
            return None;
        }

        let mut target_uid = None;

        if let Ok(uid_str) = std::env::var("SUDO_UID") {
            if let Ok(uid) = uid_str.parse::<u32>() {
                target_uid = Some(uid);
            }
        }

        if target_uid.is_none() {
            if let Ok(uid_str) = std::env::var("PKEXEC_UID") {
                if let Ok(uid) = uid_str.parse::<u32>() {
                    target_uid = Some(uid);
                }
            }
        }

        if target_uid.is_none() {
            if let Ok(content) = std::fs::read_to_string("/proc/self/loginuid") {
                if let Ok(uid) = content.trim().parse::<u32>() {
                    if uid != 4294967295 {
                        target_uid = Some(uid);
                    }
                }
            }
        }

        if let Some(uid) = target_uid {
            unsafe {
                let pw = libc::getpwuid(uid);
                if !pw.is_null() {
                    let gid = (*pw).pw_gid;
                    return Some((uid, gid));
                }
            }
        }

        None
    }

    /// Run an external command as the original user if we're root.
    /// Uses native OS setuid/setgid to avoid slow PAM/logging/sudo startup overhead.
    fn run_as_user(program: &str, args: &[&str]) -> std::process::Output {
        let mut cmd = Self::user_cmd(program);
        cmd.args(args);
        match cmd.output() {
            Ok(output) => output,
            Err(e) => {
                eprintln!("[vietc] Failed to run {}: {}", program, e);
                std::process::Output {
                    status: std::process::ExitStatus::default(),
                    stdout: vec![],
                    stderr: format!("{}\n", e).into_bytes(),
                }
            }
        }
    }

    /// Send backspaces and text through a single injection channel to avoid
    /// reordering between input methods. Backspaces always go through uinput
    /// (kernel device, no display server dependency). Text is typed via the
    /// best available method: ydotool (uinput) for ASCII, xdotool (X11) or
    /// clipboard for Unicode.
    fn inject_replacement_atomic(&self, backspaces: usize, text: &str) -> InjectResult {
        // If all ASCII, send keycodes directly
        if text.chars().all(|c| char_to_linux_keycode(c).is_some() || c == '\n') {
            if backspaces > 0 {
                for _ in 0..backspaces { let _ = self.send_backspace(); }
            }
            for ch in text.chars() {
                if ch == '\n' { self.send_enter(); }
                else { let _ = self.send_char(ch); }
            }
            return InjectResult::Success;
        }

        // Unicode: clipboard paste. Backspaces FIRST, then paste everything at once.
        if backspaces > 0 {
            for _ in 0..backspaces { let _ = self.send_backspace(); }
        }

        if self.copy_to_clipboard(text) {
            self.send_ctrl_v_x11();
        }

        InjectResult::Success
    }

    /// Copy text to clipboard and paste via Ctrl+V through our uinput device.
    /// Only used as a last resort if Wayland/X11 direct typing tools are unavailable.
    /// Tries xdotool first (X11/XWayland), then clipboard fallback.
    fn paste_string(&self, s: &str) {
        let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        if is_wayland {
            eprintln!("[vietc] paste_string: trying wtype...");
            let output = Self::run_as_user("wtype", &["--", s]);
            if output.status.success() {
                eprintln!("[vietc] paste_string: wtype success");
                return;
            }
            eprintln!("[vietc] paste_string: wtype failed, trying clipboard...");
        } else {
            // Try xdotool first (works on X11 and XWayland for UTF-8)
            eprintln!("[vietc] paste_string: trying xdotool...");
            let output = Self::run_as_user("xdotool", &["type", s]);
            if output.status.success() {
                eprintln!("[vietc] paste_string: xdotool success");
                // Record pasted text for future delete/backspace operations
                let _ = Self::run_as_user("vietc", &["update-pasted", "-text", s]);
                return;
            }
            eprintln!("[vietc] paste_string: xdotool failed, trying clipboard...");
        }

        // Clipboard fallback: copy + paste via our uinput device
        let copied = self.copy_to_clipboard(s);
        if copied {
            eprintln!("[vietc] paste_string: clipboard OK, sending Ctrl+V");
            self.send_ctrl_v();
            return;
        }

        eprintln!(
            "[vietc] WARNING: No injection method works for '{}'!",
            s.escape_default()
        );
    }

    /// Build a command to run as the original user with display environment.
    fn user_cmd(program: &str) -> std::process::Command {
        let is_root = unsafe { libc::getuid() == 0 };
        if is_root {
            if let Some((uid, gid)) = Self::get_original_uid_gid() {
                let wayland_display = std::env::var("WAYLAND_DISPLAY").unwrap_or_default();
                let xdg_runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_default();
                let display = std::env::var("DISPLAY").unwrap_or_default();
                let xauthority = std::env::var("XAUTHORITY").unwrap_or_default();

                use std::os::unix::process::CommandExt;
                let mut cmd = std::process::Command::new(program);
                cmd.uid(uid).gid(gid);

                if !wayland_display.is_empty() {
                    cmd.env("WAYLAND_DISPLAY", wayland_display);
                }
                if !xdg_runtime_dir.is_empty() {
                    cmd.env("XDG_RUNTIME_DIR", xdg_runtime_dir);
                }
                if !display.is_empty() {
                    cmd.env("DISPLAY", display);
                }
                if !xauthority.is_empty() {
                    cmd.env("XAUTHORITY", xauthority);
                }
                if let Some(username) = Self::get_original_username() {
                    cmd.env("HOME", format!("/home/{}", username));
                }
                return cmd;
            }
        }
        std::process::Command::new(program)
    }

    /// Copy text to clipboard using wl-copy (Wayland) or xclip (X11).
    fn copy_to_clipboard(&self, s: &str) -> bool {
        // Try wl-copy (Wayland) via user_cmd
        {
            let mut cmd = Self::user_cmd("wl-copy");
            let result = cmd
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    child.stdin.take().unwrap().write_all(s.as_bytes())?;
                    child.wait()
                });
            if let Ok(status) = result {
                if status.success() {
                    return true;
                }
            }
        }

        // Try xclip (X11) via user_cmd
        {
            let mut cmd = Self::user_cmd("xclip");
            cmd.args(["-selection", "clipboard"]);
            let result = cmd
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    child.stdin.take().unwrap().write_all(s.as_bytes())?;
                    child.wait()
                })
                .map(|status| status.success())
                .unwrap_or(false);
            if result {
                return true;
            }
        }

        false
    }

    /// Send Ctrl+V through our uinput device.
    fn send_ctrl_v(&self) {
        self.send_uinput_event(EV_KEY, 29, 1); // KEY_LEFTCTRL press
        self.send_uinput_event(0, 0, 0); // SYN
        std::thread::sleep(std::time::Duration::from_millis(5));

        self.send_uinput_event(EV_KEY, 47, 1); // KEY_V press
        self.send_uinput_event(0, 0, 0); // SYN
        std::thread::sleep(std::time::Duration::from_millis(5));

        self.send_uinput_event(EV_KEY, 47, 0); // KEY_V release
        self.send_uinput_event(0, 0, 0); // SYN
        std::thread::sleep(std::time::Duration::from_millis(5));

        self.send_uinput_event(EV_KEY, 29, 0); // KEY_LEFTCTRL release
        self.send_uinput_event(0, 0, 0); // SYN
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    /// Send Ctrl+V via X11 XTest (avoids uinput kernel feedback loop).
    /// Uses a lazily-opened persistent X11 connection.
    fn send_ctrl_v_x11(&self) {
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            self.send_ctrl_v();
            return;
        }
        // Persistent X11 state (raw pointers, only used from injection thread)
        static mut X11_DPY: *mut libc::c_void = std::ptr::null_mut();
        static mut X11_KEY: Option<unsafe extern "C" fn(*mut libc::c_void, u32, libc::c_int, u64) -> libc::c_int> = None;
        static mut X11_FLUSH: Option<unsafe extern "C" fn(*mut libc::c_void) -> libc::c_int> = None;
        static mut X11_KEYCODE: Option<unsafe extern "C" fn(*mut libc::c_void, u64) -> u32> = None;
        static X11_INIT: std::sync::Once = std::sync::Once::new();

        X11_INIT.call_once(|| {
            unsafe {
                let lib = libc::dlopen(b"libX11.so.6\0".as_ptr() as *const libc::c_char, 1);
                if lib.is_null() { return; }
                let xtst = libc::dlopen(b"libXtst.so.6\0".as_ptr() as *const libc::c_char, 1);
                if xtst.is_null() { libc::dlclose(lib); return; }

                type FnOpen = unsafe extern "C" fn(*const libc::c_char) -> *mut libc::c_void;
                let xopen: FnOpen = std::mem::transmute(libc::dlsym(lib, b"XOpenDisplay\0".as_ptr() as *const libc::c_char));
                let dpy = xopen(std::ptr::null());
                if dpy.is_null() { libc::dlclose(xtst); libc::dlclose(lib); return; }

                X11_DPY = dpy;
                X11_KEY = Some(std::mem::transmute(libc::dlsym(xtst, b"XTestFakeKeyEvent\0".as_ptr() as *const libc::c_char)));
                X11_FLUSH = Some(std::mem::transmute(libc::dlsym(lib, b"XFlush\0".as_ptr() as *const libc::c_char)));
                X11_KEYCODE = Some(std::mem::transmute(libc::dlsym(lib, b"XKeysymToKeycode\0".as_ptr() as *const libc::c_char)));
            }
        });

        unsafe {
            if X11_DPY.is_null() || X11_KEY.is_none() { self.send_ctrl_v(); return; }
            let dpy = X11_DPY;
            let xkey = X11_KEY.unwrap();
            let xflush = X11_FLUSH.unwrap();
            let xkeycode = X11_KEYCODE.unwrap();
            let ctrl_kc = xkeycode(dpy, 0xFFE3);
            let v_kc = xkeycode(dpy, 0x0076);
            xkey(dpy, ctrl_kc, 1, 0);
            xkey(dpy, v_kc, 1, 0);
            xkey(dpy, v_kc, 0, 0);
            xkey(dpy, ctrl_kc, 0, 0);
            xflush(dpy);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
}

impl Drop for UinputInjector {
    fn drop(&mut self) {
        let _ = ioctl(self.file.as_raw_fd(), UI_DEV_DESTROY, 0);
    }
}

fn strip_vn_diacritic(ch: char) -> char {
    match ch {
        'à' | 'á' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ' | 'â' | 'ầ' | 'ấ' | 'ẩ' | 'ẫ' | 'ậ' => 'a',
        'À' | 'Á' | 'Ả' | 'Ã' | 'Ạ' | 'Ă' | 'Ằ' | 'Ắ' | 'Ẳ' | 'Ẵ' | 'Ặ' | 'Â' | 'Ầ' | 'Ấ' | 'Ẩ' | 'Ẫ' | 'Ậ' => 'A',
        'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ' => 'e',
        'È' | 'É' | 'Ẻ' | 'Ẽ' | 'Ẹ' | 'Ê' | 'Ề' | 'Ế' | 'Ể' | 'Ễ' | 'Ệ' => 'E',
        'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị' => 'i',
        'Ì' | 'Í' | 'Ỉ' | 'Ĩ' | 'Ị' => 'I',
        'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ờ' | 'ớ' | 'ở' | 'ỡ' | 'ợ' => 'o',
        'Ò' | 'Ó' | 'Ỏ' | 'Õ' | 'Ọ' | 'Ô' | 'Ồ' | 'Ố' | 'Ổ' | 'Ỗ' | 'Ộ' | 'Ơ' | 'Ờ' | 'Ớ' | 'Ở' | 'Ỡ' | 'Ợ' => 'O',
        'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự' => 'u',
        'Ù' | 'Ú' | 'Ủ' | 'Ũ' | 'Ụ' | 'Ư' | 'Ừ' | 'Ứ' | 'Ử' | 'Ữ' | 'Ự' => 'U',
        'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
        'Ỳ' | 'Ý' | 'Ỷ' | 'Ỹ' | 'Ỵ' => 'Y',
        'đ' => 'd',
        'Đ' => 'D',
        other => other,
    }
}

fn char_to_linux_keycode(ch: char) -> Option<u16> {
    match ch.to_ascii_lowercase() {
        'a' => Some(30),
        'b' => Some(48),
        'c' => Some(46),
        'd' => Some(32),
        'e' => Some(18),
        'f' => Some(33),
        'g' => Some(34),
        'h' => Some(35),
        'i' => Some(23),
        'j' => Some(36),
        'k' => Some(37),
        'l' => Some(38),
        'm' => Some(50),
        'n' => Some(49),
        'o' => Some(24),
        'p' => Some(25),
        'q' => Some(16),
        'r' => Some(19),
        's' => Some(31),
        't' => Some(20),
        'u' => Some(22),
        'v' => Some(47),
        'w' => Some(17),
        'x' => Some(45),
        'y' => Some(21),
        'z' => Some(44),
        '0' => Some(11),
        '1' => Some(2),
        '2' => Some(3),
        '3' => Some(4),
        '4' => Some(5),
        '5' => Some(6),
        '6' => Some(7),
        '7' => Some(8),
        '8' => Some(9),
        '9' => Some(10),
        ' ' => Some(57),
        '.' => Some(52),
        ',' => Some(51),
        '-' => Some(12),
        '=' => Some(13),
        ';' => Some(39),
        '\'' => Some(40),
        '/' => Some(53),
        '\\' => Some(43),
        _ => None,
    }
}

// ioctl helper
fn ioctl(
    fd: std::os::unix::io::RawFd,
    request: u64,
    arg: u64,
) -> Result<i32, Box<dyn std::error::Error>> {
    unsafe {
        let result = libc::ioctl(fd, request, arg);
        if result < 0 {
            Err(format!("ioctl failed: {}", std::io::Error::last_os_error()).into())
        } else {
            Ok(result)
        }
    }
}

#[repr(C)]
struct input_event {
    time: timeval,
    type_: u16,
    code: u16,
    value: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct timeval {
    tv_sec: libc::time_t,
    tv_usec: libc::suseconds_t,
}

#[repr(C)]
struct uinput_setup {
    id: input_id,
    name: [i8; UINPUT_MAX_NAME_SIZE],
    ff_effects_max: u32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct input_id {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
}
