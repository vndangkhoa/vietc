// SPDX-License-Identifier: MIT
use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use super::inject::{InjectResult, KeyInjector};

/// How long to wait after the last Unicode paste before restoring the user's
/// real clipboard. Each paste pushes this deadline back, so a burst of typing
/// only triggers a single restore once the user pauses — the user's clipboard
/// is never pasted into the text while the target app might still be reading
/// our freshly pasted word.
const RESTORE_DEBOUNCE: Duration = Duration::from_millis(600);

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

/// Shared clipboard bookkeeping between the injection path and the background
/// restorer thread.
struct ClipInner {
    /// The user's real clipboard contents, saved before we overwrite the
    /// clipboard to inject Unicode text, so we can restore it afterwards.
    saved_clipboard: Option<String>,
    /// The last text we wrote to the clipboard ourselves (an injected word or
    /// the restored user content). Used to tell our own writes apart from text
    /// the user copied with Ctrl+C.
    last_injected: Option<String>,
    /// Whether we have already snapshot the user's clipboard this session.
    /// After the first snapshot, subsequent pastes skip the read_clipboard
    /// call (saving ~10-50ms per paste).
    clipboard_saved: bool,
    /// When set, the restorer thread should rewrite the user's clipboard at
    /// this instant. `None` means no restore is pending.
    restore_due: Option<Instant>,
    /// Set on shutdown so the restorer thread can exit.
    shutdown: bool,
}

struct ClipState {
    inner: Mutex<ClipInner>,
    cv: Condvar,
}

pub struct UinputInjector {
    file: File,
    clip: Arc<ClipState>,
}

unsafe impl Send for UinputInjector {}
unsafe impl Sync for UinputInjector {}

impl UinputInjector {
    fn send_enter(&self) {
        self.send_uinput_event(EV_KEY, 28, 1);
        self.send_uinput_event(0, 0, 0);
        std::thread::sleep(std::time::Duration::from_micros(100));

        self.send_uinput_event(EV_KEY, 28, 0);
        self.send_uinput_event(0, 0, 0);
        std::thread::sleep(std::time::Duration::from_micros(100));
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

        let clip = Arc::new(ClipState {
            inner: Mutex::new(ClipInner {
                saved_clipboard: None,
                last_injected: None,
                clipboard_saved: false,
                restore_due: None,
                shutdown: false,
            }),
            cv: Condvar::new(),
        });
        {
            let clip = Arc::clone(&clip);
            std::thread::spawn(move || run_restorer(clip));
        }

        Ok(Self { file, clip })
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
            self.send_uinput_event(EV_KEY, 42, 1);
            self.send_uinput_event(0, 0, 0);
            std::thread::sleep(std::time::Duration::from_micros(100));
        }

        self.send_uinput_event(EV_KEY, keycode, 1);
        self.send_uinput_event(0, 0, 0);
        std::thread::sleep(std::time::Duration::from_micros(100));

        self.send_uinput_event(EV_KEY, keycode, 0);
        self.send_uinput_event(0, 0, 0);
        std::thread::sleep(std::time::Duration::from_micros(100));

        if shift {
            self.send_uinput_event(EV_KEY, 42, 0);
            self.send_uinput_event(0, 0, 0);
            std::thread::sleep(std::time::Duration::from_micros(100));
        }
    }
}

impl KeyInjector for UinputInjector {
    fn send_backspace(&self) -> InjectResult {
        self.send_uinput_event(EV_KEY, 14, 1);
        self.send_uinput_event(0, 0, 0);
        std::thread::sleep(std::time::Duration::from_micros(100));

        self.send_uinput_event(EV_KEY, 14, 0);
        self.send_uinput_event(0, 0, 0);
        std::thread::sleep(std::time::Duration::from_micros(100));

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

        if is_ascii {
            for ch in s.chars() {
                self.send_char(ch);
            }
            return InjectResult::Success;
        }

        // Unicode text: clipboard copy + paste (reliable method)
        if !self.paste_via_clipboard(s) {
            eprintln!(
                "[vietc] send_string failed for '{}' (clipboard unavailable)",
                s.escape_default()
            );
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

    /// Send backspaces and text through a single injection channel to avoid
    /// reordering between input methods. Backspaces always go through uinput
    /// (kernel device, no display server dependency). Text is typed via the
    /// best available method: ydotool (uinput) for ASCII, xdotool (X11) or
    /// clipboard for Unicode.
    fn inject_replacement_atomic(&self, backspaces: usize, text: &str) -> InjectResult {
        let t0 = std::time::Instant::now();
        // If all ASCII, send keycodes directly
        if text.chars().all(|c| char_to_linux_keycode(c).is_some() || c == '\n') {
            if backspaces > 0 {
                for _ in 0..backspaces { let _ = self.send_backspace(); }
            }
            for ch in text.chars() {
                if ch == '\n' { self.send_enter(); }
                else { let _ = self.send_char(ch); }
            }
            eprintln!("[vietc] inject: ASCII backspaces={} text='{}' took {}ms", backspaces, text.escape_default(), (std::time::Instant::now() - t0).as_millis());
            return InjectResult::Success;
        }

        // Unicode: backspaces via uinput, then delegate to send_string()
        if backspaces > 0 {
            for _ in 0..backspaces { let _ = self.send_backspace(); }
        }
        self.send_string(text);
        InjectResult::Success
    }

    /// Read the user's current clipboard contents (wl-paste on Wayland, xclip
    /// on X11). Returns None if no clipboard tool is available or it is empty.
    fn read_clipboard() -> Option<String> {
        let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        let (prog, args): (&str, &[&str]) = if is_wayland {
            ("wl-paste", &["-n"])
        } else {
            ("xclip", &["-selection", "clipboard", "-o"])
        };
        let mut cmd = Self::user_cmd(prog);
        cmd.args(args);
        let output = cmd.output().ok()?;
        if !output.status.success() {
            return None;
        }
        Some(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    /// Inject Unicode `text` by placing it on the clipboard and sending Ctrl+V,
    /// while preserving the user's own clipboard contents. Without this, every
    /// Vietnamese word the user types would overwrite whatever they had copied
    /// with Ctrl+C, so a subsequent Ctrl+V would paste the wrong thing.
    ///
    /// Returns whether the text was successfully copied to the clipboard.
    fn paste_via_clipboard(&self, text: &str) -> bool {
        let t_total = std::time::Instant::now();
        // Critical section: snapshot the clipboard, decide what to preserve,
        // cancel any pending restore so the restorer cannot fire while we
        // paste, and put our word on the clipboard. The read and write happen
        // under the lock so they can never interleave with the restorer.
        {
            let mut st = self.clip.inner.lock().unwrap();
            if !st.clipboard_saved {
                let current = Self::read_clipboard();
                let is_our_write =
                    matches!((&current, &st.last_injected), (Some(c), Some(l)) if c == l);
                if !is_our_write {
                    st.saved_clipboard = current;
                }
                st.clipboard_saved = true;
            }
            st.restore_due = None;
            let copied = Self::copy_to_clipboard(text);
            if !copied {
                return false;
            }
            st.last_injected = Some(text.to_string());
        }

        // Give the selection owner a moment to take ownership before pasting.
        std::thread::sleep(std::time::Duration::from_micros(200));

        self.send_ctrl_v();
        let elapsed = (std::time::Instant::now() - t_total).as_millis();
        if elapsed > 20 {
            eprintln!("[vietc] paste took {}ms", elapsed);
        }

        // Schedule a debounced restore. While the user keeps typing this gets
        // pushed back, so the user's clipboard is only restored once typing
        // settles — never overwriting our freshly pasted word mid-stream.
        {
            let mut st = self.clip.inner.lock().unwrap();
            st.restore_due = Some(Instant::now() + RESTORE_DEBOUNCE);
        }
        self.clip.cv.notify_all();
        true
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

    /// Copy text to clipboard using xclip (X11) or wl-copy (Wayland).
    /// NOTE: direct X11 API is avoided here because it can interact badly with
    /// the evdev keyboard grab and/or focus — xclip is simpler and works reliably
    /// on the host.
    fn copy_to_clipboard(s: &str) -> bool {
        let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
        let (prog, args): (&str, &[&str]) = if is_wayland {
            // On Wayland/GNOME, wl-copy exits before the compositor reads
            // the clipboard data.  --paste-once keeps it alive until pasted,
            // eliminating the 300–900 ms compositor lookup delay.  We spawn
            // it detached (no .wait()) — the child lives until Ctrl+V lands.
            ("wl-copy", &["--paste-once"])
        } else {
            ("xclip", &["-selection", "clipboard", "-i"])
        };
        let mut cmd = Self::user_cmd(prog);
        cmd.args(args);
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());

        match cmd.spawn() {
            Ok(mut child) => {
                use std::io::Write;
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(s.as_bytes());
                }
                if is_wayland {
                    // --paste-once: don't wait — child stays alive until the
                    // compositor reads the data (Ctrl+V arrives later).
                    // Detach the wait so we don't block.
                    std::thread::spawn(move || {
                        let _ = child.wait();
                    });
                    return true;
                }
                // X11: wait for xclip to finish writing
                child.wait().map(|s| s.success()).unwrap_or(false)
            }
            Err(e) => {
                eprintln!("[vietc] copy_to_clipboard: {} spawn failed: {}", prog, e);
                false
            }
        }
    }

    /// Send Ctrl+V through our uinput device.
    fn send_ctrl_v(&self) {
        self.send_uinput_event(EV_KEY, 29, 1); // KEY_LEFTCTRL press
        self.send_uinput_event(0, 0, 0); // SYN
        std::thread::sleep(std::time::Duration::from_micros(100));

        self.send_uinput_event(EV_KEY, 47, 1); // KEY_V press
        self.send_uinput_event(0, 0, 0); // SYN
        std::thread::sleep(std::time::Duration::from_micros(100));

        self.send_uinput_event(EV_KEY, 47, 0); // KEY_V release
        self.send_uinput_event(0, 0, 0); // SYN
        std::thread::sleep(std::time::Duration::from_micros(100));

        self.send_uinput_event(EV_KEY, 29, 0); // KEY_LEFTCTRL release
        self.send_uinput_event(0, 0, 0); // SYN
        std::thread::sleep(std::time::Duration::from_micros(100));
    }

}

impl Drop for UinputInjector {
    fn drop(&mut self) {
        {
            let mut st = self.clip.inner.lock().unwrap();
            st.shutdown = true;
        }
        self.clip.cv.notify_all();
        let _ = ioctl(self.file.as_raw_fd(), UI_DEV_DESTROY, 0);
    }
}

/// Background thread: once no Unicode paste has happened for `RESTORE_DEBOUNCE`,
/// rewrite the user's real clipboard so Ctrl+V keeps working.
fn run_restorer(state: Arc<ClipState>) {
    loop {
        let mut st = state.inner.lock().unwrap();
        loop {
            if st.shutdown {
                return;
            }
            match st.restore_due {
                None => {
                    st = state.cv.wait(st).unwrap();
                }
                Some(due) => {
                    let now = Instant::now();
                    if now >= due {
                        break;
                    }
                    let (guard, _) = state.cv.wait_timeout(st, due - now).unwrap();
                    st = guard;
                }
            }
        }
        // Deadline reached. Restore under the lock so the write cannot
        // interleave with a concurrent paste's clipboard write.
        if let Some(restored) = st.saved_clipboard.clone() {
            let _ = UinputInjector::copy_to_clipboard(&restored);
            st.last_injected = Some(restored);
        }
        st.restore_due = None;
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
