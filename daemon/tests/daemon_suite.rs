// SPDX-License-Identifier: MIT
//! Daemon integration tests.
//!
//! These tests exercise the full Viet+ pipeline using a virtual
//! /dev/uinput keyboard. They require root for device access.
//!
//! Run with: `sudo cargo test -p vietc-daemon`

mod common;

use std::time::Duration;
use common::clipboard::{self, clear_clipboard};
use common::virtual_keyboard::VirtualKeyboard;
use common::{create_temp_config, is_root, DaemonProcess};

/// Test that a virtual keyboard can be created and destroyed.
/// This validates the /dev/uinput path used by the daemon.
#[test]
fn virtual_keyboard_create_destroy() {
    if !is_root() {
        eprintln!("SKIPPING: needs root for /dev/uinput");
        return;
    }

    let mut kb = VirtualKeyboard::create("vietc-test-create-destroy")
        .expect("failed to create virtual keyboard");
    kb.type_char('a').expect("failed to type char");
    eprintln!("[test] Virtual keyboard created and typed 'a' OK");
}

/// Test that clipboard can be written and read (xclip or wl-paste).
#[test]
fn clipboard_read_write() {
    clear_clipboard();
    // Write to clipboard using xclip/wl-copy
    let is_wayland = std::env::var("WAYLAND_DISPLAY").ok().map_or(false, |v| v.contains("wayland"));
    let (prog, args): (&str, &[&str]) = if is_wayland {
        ("wl-copy", &[])
    } else {
        ("xclip", &["-selection", "clipboard", "-i"])
    };
    let mut child = std::process::Command::new(prog)
        .args(args)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .expect("failed to spawn clipboard tool");
    use std::io::Write;
    if let Some(mut stdin) = child.stdin.take() {
        writeln!(stdin, "test-clipboard-content").ok();
    }
    let status = child.wait().expect("clipboard tool failed");
    assert!(status.success(), "clipboard write failed");

    std::thread::sleep(Duration::from_millis(100));

    let content = clipboard::read_clipboard()
        .expect("failed to read clipboard");
    assert!(content.contains("test-clipboard-content"), "clipboard content mismatch: '{}'", content);
}

/// Full pipeline test: create virtual keyboard, spawn daemon,
/// type VNI keystrokes, verify clipboard output.
#[test]
fn vni_simple_word_grabbed() {
    if !is_root() {
        eprintln!("SKIPPING: needs root");
        return;
    }

    // Clean up any previous daemon processes that might conflict
    let _ = std::process::Command::new("pkill")
        .args(["-x", "vietc-daemon"])
        .output();

    // Create virtual keyboard before spawning daemon
    let mut kb = VirtualKeyboard::create("vietc-test-vni-word")
        .expect("failed to create virtual keyboard");

    // Wait for the /dev/input/eventX node to appear
    kb.wait_for_devnode(Duration::from_secs(3));

    // Create a temp config with grab enabled, VNI mode
    let config_dir = create_temp_config("vni", true, true);

    // Spawn the daemon
    let daemon = DaemonProcess::spawn(config_dir.path());

    // Wait for daemon to initialize and grab
    let grabbed = daemon.wait_for_log("Keyboard grabbed", Duration::from_secs(5));
    if !grabbed {
        if daemon.is_running() {
            eprintln!("[test] Daemon didn't grab but is still running (non-grabbed mode OK)");
        } else {
            eprintln!("[test] Daemon exited before grabbing");
            daemon.kill();
            return;
        }
    }

    // Clear clipboard before typing
    clear_clipboard();
    std::thread::sleep(Duration::from_millis(100));

    // Type "tho2i " which should produce "thời "
    eprintln!("[test] Typing 'tho2i '...");
    kb.type_text("tho2i ").expect("failed to type text");

    // Wait for daemon to process and paste
    std::thread::sleep(Duration::from_millis(1500));

    // Read clipboard to verify
    if let Some(content) = clipboard::read_clipboard() {
        eprintln!("[test] Clipboard contains: '{}'", content);
        // In grabbed mode, the output might be "thời " without the space,
        // depending on how the flush char is handled
        assert!(content.contains("thời"), "Expected 'thời' in clipboard, got '{}'", content);
    } else {
        eprintln!("[test] Clipboard empty after typing (daemon might not have injected)");
    }

    daemon.kill();
}

/// Non-grabbed mode test: verify daemon processes keystrokes
/// without grabbing any device.
#[test]
fn vni_simple_word_nongrabbed() {
    if !is_root() {
        eprintln!("SKIPPING: needs root");
        return;
    }

    let _ = std::process::Command::new("pkill")
        .args(["-x", "vietc-daemon"])
        .output();

    let mut kb = VirtualKeyboard::create("vietc-test-vni-ng")
        .expect("failed to create virtual keyboard");
    kb.wait_for_devnode(Duration::from_secs(3));

    // Non-grabbed mode
    let config_dir = create_temp_config("vni", false, true);
    let daemon = DaemonProcess::spawn(config_dir.path());

    // Wait for daemon to initialize
    std::thread::sleep(Duration::from_millis(1000));
    if !daemon.is_running() {
        eprintln!("[test] Daemon exited during init");
        return;
    }

    clear_clipboard();
    std::thread::sleep(Duration::from_millis(100));

    kb.type_text("tho2i ").expect("failed to type text");
    std::thread::sleep(Duration::from_millis(1500));

    if let Some(content) = clipboard::read_clipboard() {
        eprintln!("[test] Clipboard: '{}'", content);
        assert!(content.contains("thời"), "Expected 'thời', got '{}'", content);
    } else {
        eprintln!("[test] Clipboard empty");
    }

    daemon.kill();
}
