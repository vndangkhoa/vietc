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

        ioctl(fd, UI_DEV_CREATE, 0)
            .map_err(|e| format!("UI_DEV_CREATE failed: {}", e))?;

        // Small delay for device to be ready
        std::thread::sleep(std::time::Duration::from_millis(10));

        Ok(Self { file })
    }

    fn send_uinput_event(&self, type_: u16, code: u16, value: i32) {
        let event = input_event {
            time: timeval { tv_sec: 0, tv_usec: 0 },
            type_,
            code,
            value,
        };

        unsafe {
            let ptr = &event as *const input_event as *const u8;
            let len = std::mem::size_of::<input_event>();
            let _ = libc::write(self.file.as_raw_fd() as libc::c_int, ptr as *const libc::c_void, len);
        }
    }
}

impl KeyInjector for UinputInjector {
    fn send_backspace(&self) -> InjectResult {
        self.send_uinput_event(EV_KEY, 14, 1); // KEY_BACKSPACE press
        self.send_uinput_event(EV_KEY, 14, 0); // KEY_BACKSPACE release
        self.send_uinput_event(0, 0, 0);       // EV_SYN
        InjectResult::Success
    }

    fn send_key_event(&self, keycode: u16, value: i32) -> InjectResult {
        self.send_uinput_event(EV_KEY, keycode, value);
        self.send_uinput_event(0, 0, 0);
        InjectResult::Success
    }

    fn send_char(&self, ch: char) -> InjectResult {
        if let Some(keycode) = char_to_linux_keycode(ch) {
            let needs_shift = ch.is_uppercase() || "!@#$%^&*()_+{}|:\"<>?".contains(ch);
            if needs_shift {
                self.send_uinput_event(EV_KEY, 42, 1); // KEY_LEFTSHIFT
            }
            self.send_uinput_event(EV_KEY, keycode, 1);
            self.send_uinput_event(EV_KEY, keycode, 0);
            if needs_shift {
                self.send_uinput_event(EV_KEY, 42, 0);
            }
            self.send_uinput_event(0, 0, 0);
            return InjectResult::Success;
        }
        // Unicode: copy to clipboard and paste (preserves uinput ordering)
        self.paste_string(&ch.to_string());
        InjectResult::Success
    }

    fn send_string(&self, s: &str) -> InjectResult {
        // If all ASCII, use keycodes directly (fast path)
        if s.chars().all(|c| char_to_linux_keycode(c).is_some()) {
            for ch in s.chars() {
                self.send_char(ch);
            }
        } else {
            // Contains Unicode: single clipboard copy + paste via uinput
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
                            .to_string_lossy().into_owned();
                        if !name.is_empty() {
                            return Some(name);
                        }
                    }
                }
            }
        }

        if let Ok(content) = std::fs::read_to_string("/proc/self/loginuid") {
            if let Ok(uid) = content.trim().parse::<u32>() {
                unsafe {
                    let pw = libc::getpwuid(uid);
                    if !pw.is_null() {
                        let name = std::ffi::CStr::from_ptr((*pw).pw_name)
                            .to_string_lossy().into_owned();
                        if !name.is_empty() {
                            return Some(name);
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

    /// Run an external command as the original user if we're root.
    /// Wayland tools (wtype, wl-copy) need the user's session, not root.
    /// Uses explicit `env VAR=val` instead of `--preserve-env` for
    /// compatibility with all sudo versions.
    fn run_as_user(program: &str, args: &[&str]) -> std::process::Output {
        let is_root = unsafe { libc::getuid() == 0 };
        if is_root {
            if let Some(original_user) = Self::get_original_username() {
                let wayland_display = std::env::var("WAYLAND_DISPLAY").unwrap_or_default();
                let xdg_runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_default();
                let display = std::env::var("DISPLAY").unwrap_or_default();
                let mut cmd = std::process::Command::new("sudo");
                cmd.args(["-u", &original_user, "env"]);
                if !wayland_display.is_empty() {
                    cmd.arg(format!("WAYLAND_DISPLAY={}", wayland_display));
                }
                if !xdg_runtime_dir.is_empty() {
                    cmd.arg(format!("XDG_RUNTIME_DIR={}", xdg_runtime_dir));
                }
                if !display.is_empty() {
                    cmd.arg(format!("DISPLAY={}", display));
                }
                cmd.arg(program);
                cmd.args(args);
                match cmd.output() {
                    Ok(output) => return output,
                    Err(e) => {
                        eprintln!("[vietc] Failed to run sudo -u {} env ... {} {}: {}", original_user, program, args.join(" "), e);
                        return std::process::Output {
                            status: std::process::ExitStatus::default(),
                            stdout: vec![],
                            stderr: format!("{}\n", e).into_bytes(),
                        };
                    }
                }
            } else {
                eprintln!("[vietc] Running as root but could not determine original user");
            }
        }
        match std::process::Command::new(program).args(args).output() {
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
        let is_ascii = text.chars().all(|c| char_to_linux_keycode(c).is_some());
        
        if is_ascii {
            if backspaces > 0 {
                for _ in 0..backspaces {
                    let _ = self.send_backspace();
                }
            }
            for ch in text.chars() {
                let _ = self.send_char(ch);
            }
            return InjectResult::Success;
        }

        // It is Unicode. We must use a single unified channel.
        let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        
        if is_wayland {
            // Under Wayland, we try to use `wtype` for both backspaces and text.
            let has_wtype = std::process::Command::new("which")
                .arg("wtype")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
                
            if has_wtype {
                let mut args = Vec::new();
                for _ in 0..backspaces {
                    args.push("-k");
                    args.push("BackSpace");
                }
                args.push("--");
                args.push(text);
                
                let output = Self::run_as_user("wtype", &args);
                if output.status.success() {
                    return InjectResult::Success;
                }
                eprintln!("[vietc] wtype inject failed: {}", String::from_utf8_lossy(&output.stderr).trim());
            }
        } else {
            // Under X11, we try to use `xdotool` for both backspaces and text.
            let has_xdotool = std::process::Command::new("which")
                .arg("xdotool")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
                
            if has_xdotool {
                let mut args = Vec::new();
                if backspaces > 0 {
                    args.push("key");
                    for _ in 0..backspaces {
                        args.push("BackSpace");
                    }
                }
                if !text.is_empty() {
                    args.push("type");
                    args.push("--clearmodifiers");
                    args.push(text);
                }
                
                let output = Self::run_as_user("xdotool", &args);
                if output.status.success() {
                    return InjectResult::Success;
                }
                eprintln!("[vietc] xdotool inject failed: {}", String::from_utf8_lossy(&output.stderr).trim());
            }
        }

        // Fallback: Clipboard copy + paste.
        // This is safe because both backspaces and Ctrl+V are injected into the SAME uinput device.
        let copied = self.copy_to_clipboard(text);
        if copied {
            if backspaces > 0 {
                for _ in 0..backspaces {
                    let _ = self.send_backspace();
                }
            }
            self.send_ctrl_v();
            InjectResult::Success
        } else {
            eprintln!("[vietc] clipboard copy failed during fallback");
            // Absolute last resort: try uinput backspaces followed by individual unicode paste_string
            if backspaces > 0 {
                for _ in 0..backspaces {
                    let _ = self.send_backspace();
                }
            }
            self.paste_string(text);
            InjectResult::Success
        }
    }

    /// Copy text to clipboard and paste via Ctrl+V through our uinput device.
    /// Only used as a last resort if Wayland/X11 direct typing tools are
    /// unavailable. Prefers ydotool (uinput, works everywhere) to avoid
    /// clipboard pollution.
    fn paste_string(&self, s: &str) {
        // Try ydotool first (uinput-based, no display server needed).
        let ydotool_result = std::process::Command::new("ydotool")
            .args(["type", s])
            .output();
        if let Ok(output) = ydotool_result {
            if output.status.success() {
                eprintln!("[vietc] ydotool OK");
                return;
            }
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.is_empty() {
                eprintln!("[vietc] ydotool failed: {}", stderr.trim());
            }
        }
        eprintln!("[vietc] ydotool failed, trying xdotool...");

        // Try xdotool (X11): needs DISPLAY, run through run_as_user
        eprintln!("[vietc] trying xdotool...");
        let output = Self::run_as_user("xdotool", &["type", "--clearmodifiers", s]);
        if output.status.success() {
            eprintln!("[vietc] xdotool OK");
            return;
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            eprintln!("[vietc] xdotool failed: {}", stderr.trim());
        }

        // Try wtype (Wayland-native): needs Wayland session, run through run_as_user
        eprintln!("[vietc] xdotool failed, trying wtype...");
        let output = Self::run_as_user("wtype", &[s]);
        if output.status.success() {
            eprintln!("[vietc] wtype OK");
            return;
        }
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            eprintln!("[vietc] wtype failed: {}", stderr.trim());
        }

        // Clipboard fallback: copy + paste via our uinput
        eprintln!("[vietc] wtype failed, trying clipboard paste...");
        let copied = self.copy_to_clipboard(s);
        if copied {
            eprintln!("[vietc] clipboard OK, sending Ctrl+V");
            self.send_ctrl_v();
            return;
        }

        eprintln!("[vietc] WARNING: No injection method works for '{}'!", s);
    }

    /// Build a command to run as the original user with display environment.
    fn user_cmd(program: &str) -> std::process::Command {
        let is_root = unsafe { libc::getuid() == 0 };
        if is_root {
            if let Some(original_user) = Self::get_original_username() {
                let wayland_display = std::env::var("WAYLAND_DISPLAY").unwrap_or_default();
                let xdg_runtime_dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_default();
                let display = std::env::var("DISPLAY").unwrap_or_default();
                let mut cmd = std::process::Command::new("sudo");
                cmd.args(["-u", &original_user, "env"]);
                if !wayland_display.is_empty() {
                    cmd.arg(format!("WAYLAND_DISPLAY={}", wayland_display));
                }
                if !xdg_runtime_dir.is_empty() {
                    cmd.arg(format!("XDG_RUNTIME_DIR={}", xdg_runtime_dir));
                }
                if !display.is_empty() {
                    cmd.arg(format!("DISPLAY={}", display));
                }
                cmd.arg(program);
                return cmd;
            }
        }
        std::process::Command::new(program)
    }

    /// Copy text to clipboard using wl-copy (Wayland) or xclip (X11).
    fn copy_to_clipboard(&self, s: &str) -> bool {
        let is_root = unsafe { libc::getuid() == 0 };
        eprintln!("[vietc] clipboard: is_root={}", is_root);

        // Try wl-copy (Wayland) via user_cmd
        {
            let mut cmd = Self::user_cmd("wl-copy");
            eprintln!("[vietc] clipboard: trying wl-copy via {:?}", cmd);
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
                    eprintln!("[vietc] clipboard: wl-copy OK");
                    return true;
                }
                eprintln!("[vietc] clipboard: wl-copy failed (exit={:?})", status.code());
            } else if let Err(ref e) = result {
                eprintln!("[vietc] clipboard: wl-copy error: {}", e);
            }
        }

        // Try xclip (X11) via user_cmd
        eprintln!("[vietc] clipboard: trying xclip...");
        {
            let mut cmd = Self::user_cmd("xclip");
            cmd.args(["-selection", "clipboard"]);
            eprintln!("[vietc] clipboard: xclip via {:?}", cmd);
            let result = cmd
                .stdin(std::process::Stdio::piped())
                .spawn()
                .and_then(|mut child| {
                    use std::io::Write;
                    child.stdin.take().unwrap().write_all(s.as_bytes())?;
                    child.wait()
                })
                .map(|status| {
                    if status.success() {
                        eprintln!("[vietc] clipboard: xclip OK");
                    } else {
                        eprintln!("[vietc] clipboard: xclip failed (exit={:?})", status.code());
                    }
                    status.success()
                })
                .unwrap_or_else(|e| {
                    eprintln!("[vietc] clipboard: xclip error: {}", e);
                    false
                });
            if result {
                return true;
            }
        }

        false
    }

    /// Send Ctrl+V through our uinput device.
    fn send_ctrl_v(&self) {
        self.send_uinput_event(EV_KEY, 29, 1); // KEY_LEFTCTRL
        self.send_uinput_event(EV_KEY, 47, 1); // KEY_V
        self.send_uinput_event(EV_KEY, 47, 0);
        self.send_uinput_event(EV_KEY, 29, 0);
        self.send_uinput_event(0, 0, 0);
    }

}

impl Drop for UinputInjector {
    fn drop(&mut self) {
        let _ = ioctl(self.file.as_raw_fd(), UI_DEV_DESTROY, 0);
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
fn ioctl(fd: std::os::unix::io::RawFd, request: u64, arg: u64) -> Result<i32, Box<dyn std::error::Error>> {
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
