// SPDX-License-Identifier: MIT
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

use super::inject::{InjectResult, KeyInjector};

fn socket_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(home).join(".vietc").join("uinput.sock")
}

pub struct UinputClient;

impl UinputClient {
    fn send_command(cmd: &str) -> InjectResult {
        match UnixStream::connect(socket_path()) {
            Ok(mut stream) => {
                if writeln!(stream, "{}", cmd).is_err() {
                    return InjectResult::Failed;
                }
                let mut reader = BufReader::new(&stream);
                let mut response = String::new();
                if reader.read_line(&mut response).is_err() {
                    return InjectResult::Failed;
                }
                if response.trim() == "OK" {
                    InjectResult::Success
                } else {
                    InjectResult::Failed
                }
            }
            Err(_) => InjectResult::Failed,
        }
    }

    pub fn is_available() -> bool {
        UnixStream::connect(socket_path()).is_ok()
    }
}

impl KeyInjector for UinputClient {
    fn send_key_event(&self, _keycode: u16, _value: i32) -> InjectResult {
        InjectResult::Success
    }

    fn send_backspace(&self) -> InjectResult {
        InjectResult::Success
    }

    fn send_char(&self, _ch: char) -> InjectResult {
        InjectResult::Success
    }

    fn send_string(&self, s: &str) -> InjectResult {
        Self::send_command(&format!("TYPE:{}", s))
    }

    fn inject_replacement(&self, backspaces: usize, text: &str) -> InjectResult {
        if backspaces > 0 {
            let _ = Self::send_command(&format!("BACKSPACE:{}", backspaces));
        }
        if !text.is_empty() {
            let _ = Self::send_command(&format!("TYPE:{}", text));
        }
        InjectResult::Success
    }

    fn flush(&self) -> InjectResult {
        InjectResult::Success
    }

    fn update_pasted_text(&self, _text: &str) -> InjectResult {
        InjectResult::Success
    }
}
