// SPDX-License-Identifier: MIT
//! Virtual keyboard backend using evdev's uinput support.

use std::io::Result;
use std::path::PathBuf;
use std::time::Duration;

use evdev::uinput::VirtualDeviceBuilder;
use evdev::{AttributeSet, EventType, InputEvent, Key};

pub struct VirtualKeyboard {
    device: evdev::uinput::VirtualDevice,
}

fn add_all_keys(set: &mut AttributeSet<Key>) {
    for code in 1..=255u16 {
        set.insert(Key::new(code));
    }
}

impl VirtualKeyboard {
    pub fn create(name: &str) -> Result<Self> {
        let mut keys = AttributeSet::new();
        add_all_keys(&mut keys);

        let device = VirtualDeviceBuilder::new()?
            .name(name)
            .with_keys(&keys)?
            .build()?;

        eprintln!("[test] Created virtual keyboard '{}'", name);
        Ok(Self { device })
    }

    pub fn wait_for_devnode(&mut self, timeout: Duration) -> Option<PathBuf> {
        let start = std::time::Instant::now();
        loop {
            match self.device.enumerate_dev_nodes_blocking() {
                Ok(mut nodes) => {
                    if let Some(Ok(path)) = nodes.next() {
                        eprintln!("[test] Virtual keyboard devnode: {}", path.display());
                        return Some(path);
                    }
                }
                Err(e) => {
                    eprintln!("[test] enumerate_dev_nodes failed: {}", e);
                }
            }
            if start.elapsed() >= timeout {
                eprintln!("[test] Timeout waiting for devnode");
                return None;
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }

    /// Press and release a key.
    pub fn tap_key(&mut self, keycode: u16) -> Result<()> {
        let press = InputEvent::new(EventType::KEY, keycode, 1);
        let release = InputEvent::new(EventType::KEY, keycode, 0);
        let sync = InputEvent::new(EventType::SYNCHRONIZATION, 0, 0);
        self.device.emit(&[press, sync])?;
        std::thread::sleep(Duration::from_millis(5));
        self.device.emit(&[release, sync])?;
        Ok(())
    }

    /// Type a single character via US layout keycodes.
    pub fn type_char(&mut self, ch: char) -> Result<()> {
        let keycode = char_to_evdev(ch);
        if keycode == 0 {
            return Ok(());
        }
        let shift = ch.is_ascii_uppercase()
            || matches!(ch, '!' | '@' | '#' | '$' | '%' | '^' | '&' | '*' | '(' | ')' | '_' | '+' | '{' | '}' | '|' | ':' | '"' | '<' | '>' | '?' | '~');
        if shift {
            self.tap_key(42)?;
        }
        self.tap_key(keycode)?;
        if shift {
            self.tap_key(42)?;
        }
        Ok(())
    }

    /// Type a string of characters.
    pub fn type_text(&mut self, text: &str) -> Result<()> {
        for ch in text.chars() {
            self.type_char(ch)?;
        }
        Ok(())
    }
}

fn char_to_evdev(ch: char) -> u16 {
    match ch {
        'a' | 'A' => 30, 'b' | 'B' => 48, 'c' | 'C' => 46,
        'd' | 'D' => 32, 'e' | 'E' => 18, 'f' | 'F' => 33,
        'g' | 'G' => 34, 'h' | 'H' => 35, 'i' | 'I' => 23,
        'j' | 'J' => 36, 'k' | 'K' => 37, 'l' | 'L' => 38,
        'm' | 'M' => 50, 'n' | 'N' => 49, 'o' | 'O' => 24,
        'p' | 'P' => 25, 'q' | 'Q' => 16, 'r' | 'R' => 19,
        's' | 'S' => 31, 't' | 'T' => 20, 'u' | 'U' => 22,
        'v' | 'V' => 47, 'w' | 'W' => 17, 'x' | 'X' => 45,
        'y' | 'Y' => 21, 'z' | 'Z' => 44,
        '0' => 11, '1' => 2, '2' => 3, '3' => 4, '4' => 5,
        '5' => 6, '6' => 7, '7' => 8, '8' => 9, '9' => 10,
        ' ' => 57, '\n' => 28, '\t' => 15,
        '.' => 52, ',' => 51, '-' => 12, '=' => 13,
        ';' => 39, '\'' => 40, '/' => 53, '`' => 41,
        '[' => 26, ']' => 27, '\\' => 43,
        _ => {
            eprintln!("[test] WARNING: no evdev mapping for '{}'", ch.escape_default());
            0
        }
    }
}
