use std::fs;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::Command;

const UINPUT_MAX_NAME_SIZE: usize = 80;
const UI_SET_EVBIT: u64 = 0x40045564;
const UI_SET_KEYBIT: u64 = 0x40045565;
const UI_DEV_CREATE: u64 = 0x5501;
const UI_DEV_DESTROY: u64 = 0x5502;
const UI_DEV_SETUP: u64 = 0x405c5503;
const EV_KEY: u16 = 0x01;

fn ioctl(fd: i32, request: u64, arg: u64) -> Result<i32, String> {
    let result = unsafe { libc::ioctl(fd, request, arg) };
    if result < 0 {
        Err(format!("ioctl failed: {}", std::io::Error::last_os_error()))
    } else {
        Ok(result)
    }
}

#[repr(C)]
struct input_event {
    time: libc::timeval,
    type_: u16,
    code: u16,
    value: i32,
}

#[repr(C)]
struct uinput_setup {
    id: input_id,
    name: [i8; UINPUT_MAX_NAME_SIZE],
    ff_effects_max: u32,
}

#[repr(C)]
struct input_id {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
}

struct UinputDevice {
    fd: i32,
}

impl UinputDevice {
    fn new(name: &str) -> Result<Self, String> {
        let file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/uinput")
            .map_err(|e| format!("Cannot open /dev/uinput: {} (are you root?)", e))?;

        let fd = file.as_raw_fd();

        ioctl(fd, UI_SET_EVBIT, EV_KEY as u64)?;

        for code in 0..=0x1ffu32 {
            ioctl(fd, UI_SET_KEYBIT, code as u64)?;
        }

        let mut usetup: uinput_setup = unsafe { std::mem::zeroed() };
        let name_bytes = name.as_bytes();
        let copy_len = name_bytes.len().min(UINPUT_MAX_NAME_SIZE - 1);
        for (i, &byte) in name_bytes.iter().enumerate().take(copy_len) {
            usetup.name[i] = byte as i8;
        }
        usetup.id.bustype = 0x03;
        usetup.id.vendor = 0x1234;
        usetup.id.product = 0x5678;
        usetup.id.version = 1;

        ioctl(fd, UI_DEV_SETUP, &usetup as *const uinput_setup as u64)?;
        ioctl(fd, UI_DEV_CREATE, 0)?;

        std::mem::forget(file);
        std::thread::sleep(std::time::Duration::from_millis(10));

        eprintln!("[vietc-uinputd] Device '{}' created", name);
        Ok(Self { fd })
    }

    fn send_event(&self, type_: u16, code: u16, value: i32) {
        let event = input_event {
            time: libc::timeval { tv_sec: 0, tv_usec: 0 },
            type_,
            code,
            value,
        };
        unsafe {
            libc::write(self.fd, &event as *const input_event as *const libc::c_void, std::mem::size_of::<input_event>());
        }
    }

    fn send_key(&self, code: u16, value: i32) {
        self.send_event(EV_KEY, code, value);
        self.send_event(0, 0, 0);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

    fn backspace_n(&self, count: usize) {
        for _ in 0..count {
            self.send_key(14, 1);
            self.send_key(14, 0);
        }
    }

    fn char_to_keycode(ch: u8) -> Option<(u16, bool)> {
        let lower = ch.to_ascii_lowercase();
        let keycode = match lower {
            b'a' => 30, b'b' => 48, b'c' => 46, b'd' => 32, b'e' => 18,
            b'f' => 33, b'g' => 34, b'h' => 35, b'i' => 23, b'j' => 36,
            b'k' => 37, b'l' => 38, b'm' => 50, b'n' => 49, b'o' => 24,
            b'p' => 25, b'q' => 16, b'r' => 19, b's' => 31, b't' => 20,
            b'u' => 22, b'v' => 47, b'w' => 17, b'x' => 45, b'y' => 21,
            b'z' => 44,
            b'0' => 11, b'1' => 2, b'2' => 3, b'3' => 4, b'4' => 5,
            b'5' => 6, b'6' => 7, b'7' => 8, b'8' => 9, b'9' => 10,
            b' ' => 57, b'.' => 52, b',' => 51, b'-' => 12, b'=' => 13,
            b';' => 39, b'\'' => 40, b'/' => 53, b'\\' => 43,
            b'[' => 26, b']' => 27,
            _ => return None,
        };
        let shift = ch.is_ascii_uppercase()
            || matches!(ch, b'!' | b'@' | b'#' | b'$' | b'%' | b'^' | b'&' | b'*'
                | b'(' | b')' | b'_' | b'+' | b'{' | b'}' | b'|' | b':' | b'"'
                | b'<' | b'>' | b'?' | b'~');
        Some((keycode, shift))
    }

    fn type_ascii(&self, text: &str) {
        for byte in text.bytes() {
            if let Some((keycode, shift)) = Self::char_to_keycode(byte) {
                if shift {
                    self.send_key(42, 1);
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                self.send_key(keycode, 1);
                self.send_key(keycode, 0);
                if shift {
                    self.send_key(42, 0);
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
    }

    fn paste_unicode(&self, text: &str) {
        copy_to_clipboard(text);
        self.send_key(29, 1);
        std::thread::sleep(std::time::Duration::from_millis(2));
        self.send_key(47, 1);
        self.send_key(47, 0);
        self.send_key(29, 0);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

impl Drop for UinputDevice {
    fn drop(&mut self) {
        let _ = unsafe { libc::ioctl(self.fd, UI_DEV_DESTROY, 0) };
        let _ = unsafe { libc::close(self.fd) };
        eprintln!("[vietc-uinputd] Device destroyed");
    }
}

fn copy_to_clipboard(text: &str) {
    let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok();
    if is_wayland {
        if let Ok(mut child) = Command::new("wl-copy")
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        }
    } else {
        if let Ok(mut child) = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            let _ = child.wait();
        }
    }
}

fn find_socket_path() -> String {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    let dir = format!("{}/.vietc", home);
    let _ = fs::create_dir_all(&dir);

    if unsafe { libc::getuid() == 0 } {
        let socket = format!("{}/uinput.sock", dir);
        unsafe {
            let _ = libc::chown(
                socket.as_ptr() as *const libc::c_char,
                0,
                0,
            );
        }
        socket
    } else {
        format!("{}/uinput.sock", dir)
    }
}

fn handle_client(stream: UnixStream, uinput: &UinputDevice) {
    let reader = BufReader::new(&stream);
    let mut writer = &stream;

    for line in reader.lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };

        let line = line.trim().to_string();
        if line.is_empty() { continue; }

        if line == "PING" {
            let _ = writeln!(writer, "PONG");
        } else if line == "FLUSH" {
            let _ = writeln!(writer, "OK");
        } else if line == "QUIT" {
            let _ = writeln!(writer, "BYE");
            break;
        } else if let Some(n_str) = line.strip_prefix("BACKSPACE:") {
            if let Ok(n) = n_str.parse::<usize>() {
                uinput.backspace_n(n);
                let _ = writeln!(writer, "OK");
            } else {
                let _ = writeln!(writer, "ERR bad count");
            }
        } else if let Some(text) = line.strip_prefix("TYPE:") {
            let is_ascii = text.bytes().all(|b| UinputDevice::char_to_keycode(b).is_some());
            if is_ascii {
                uinput.type_ascii(text);
            } else {
                uinput.paste_unicode(text);
            }
            let _ = writeln!(writer, "OK");
        } else if let Some(text) = line.strip_prefix("PASTE:") {
            uinput.paste_unicode(text);
            let _ = writeln!(writer, "OK");
        } else {
            let _ = writeln!(writer, "ERR unknown command");
        }
    }
}

fn main() {
    let socket_path = find_socket_path();
    let path = Path::new(&socket_path);

    let _ = fs::remove_file(path);

    let listener = match UnixListener::bind(path) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[vietc-uinputd] Cannot bind socket {}: {}", socket_path, e);
            std::process::exit(1);
        }
    };

    // Make socket world-writable so non-root daemon can connect
    unsafe {
        let _ = libc::chmod(
            socket_path.as_ptr() as *const libc::c_char,
            0o666,
        );
    }

    let uinput = match UinputDevice::new("vietc") {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[vietc-uinputd] {}", e);
            std::process::exit(1);
        }
    };

    eprintln!("[vietc-uinputd] Listening on {}", socket_path);

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                handle_client(stream, &uinput);
            }
            Err(e) => {
                eprintln!("[vietc-uinputd] Connection error: {}", e);
            }
        }
    }
}
