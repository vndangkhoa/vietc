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
        ioctl(fd, UI_SET_EVBIT, EV_KEY as u64)?;

        // Enable all key codes we'll need
        for code in 0..=KEY_MAX {
            ioctl(fd, UI_SET_KEYBIT, code as u64)?;
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

        ioctl(fd, UI_DEV_CREATE, &usetup as *const uinput_setup as u64)?;

        // Wait a bit for device to be ready
        std::thread::sleep(std::time::Duration::from_millis(100));

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

    fn send_char(&self, ch: char) -> InjectResult {
        if let Some(keycode) = char_to_linux_keycode(ch) {
            let needs_shift = ch.is_uppercase() || "!@#$%^&*()_+{}|:\"<>?".contains(ch);
            let shift_keycode: u16 = 42; // KEY_LEFTSHIFT

            if needs_shift {
                self.send_uinput_event(EV_KEY, shift_keycode, 1);
            }
            self.send_uinput_event(EV_KEY, keycode, 1);
            self.send_uinput_event(EV_KEY, keycode, 0);
            if needs_shift {
                self.send_uinput_event(EV_KEY, shift_keycode, 0);
            }
            self.send_uinput_event(0, 0, 0); // EV_SYN
            return InjectResult::Success;
        }

        // For Unicode, we can't use uinput directly
        // Fall back to clipboard paste or xdotool
        InjectResult::NotSupported
    }

    fn send_string(&self, s: &str) -> InjectResult {
        for ch in s.chars() {
            let r = self.send_char(ch);
            if r != InjectResult::Success {
                return r;
            }
        }
        InjectResult::Success
    }

    fn flush(&self) -> InjectResult {
        InjectResult::Success
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
    name: [i8; UINPUT_MAX_NAME_SIZE],
    id: input_id,
    ff_effects_max: u32,
    absmax: [i32; 64],
    absmin: [i32; 64],
    absfuzz: [i32; 64],
    absflat: [i32; 64],
}

#[repr(C)]
#[derive(Clone, Copy)]
struct input_id {
    bustype: u16,
    vendor: u16,
    product: u16,
    version: u16,
}
