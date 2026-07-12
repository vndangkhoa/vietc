// SPDX-License-Identifier: MIT
//! Rootless Wayland input method via zwp_input_method_v2.
//!
//! This makes vietc run as a normal user on Wayland with no evdev grab, no
//! uinput, no input-group udev rule and no file capabilities. The compositor
//! routes key events to us; we compose Vietnamese via the engine and commit
//! the result back through the input-method protocol (preedit while typing,
//! commit on word boundaries). Keys we do not compose (modifiers, shortcuts,
//! navigation, the trailing space of a word) are forwarded to the focused app
//! through a zwp_virtual_keyboard_v1, since zwp_input_method_v2 has no
//! forward_key request.
use std::collections::HashSet;
use std::os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use wayland_client::protocol::wl_keyboard::{KeyState, KeymapFormat};
use wayland_client::protocol::wl_registry::{Event as WlRegistryEvent, WlRegistry};
use wayland_client::protocol::wl_seat::WlSeat;
use wayland_client::{Connection, Dispatch, EventQueue, QueueHandle, WEnum};
use wayland_protocols_misc::zwp_input_method_v2::client::{
    zwp_input_method_keyboard_grab_v2::{
        Event as KbEvent, ZwpInputMethodKeyboardGrabV2,
    },
    zwp_input_method_manager_v2::{
        Event as ManagerEvent, ZwpInputMethodManagerV2,
    },
    zwp_input_method_v2::{Event as ImEvent, ZwpInputMethodV2},
};
use wayland_protocols_misc::zwp_virtual_keyboard_v1::client::{
    zwp_virtual_keyboard_manager_v1::{
        Event as VkMgrEvent, ZwpVirtualKeyboardManagerV1,
    },
    zwp_virtual_keyboard_v1::{Event as VkEvent, ZwpVirtualKeyboardV1},
};
use xkbcommon::xkb;

use vietc_engine::{Engine, EngineEvent, InputMethod};

use crate::config::Config;
use crate::display::DisplayServer;
use crate::signal::SIGNAL_EXIT;

fn parse_method(s: &str) -> InputMethod {
    match s.to_ascii_lowercase().as_str() {
        "vni" => InputMethod::Vni,
        _ => InputMethod::Telex,
    }
}

struct ImState {
    seat: Option<WlSeat>,
    im_manager: Option<ZwpInputMethodManagerV2>,
    vk_manager: Option<ZwpVirtualKeyboardManagerV1>,
    im: Option<ZwpInputMethodV2>,
    keyboard: Option<ZwpInputMethodKeyboardGrabV2>,
    vk: Option<ZwpVirtualKeyboardV1>,
    keymap_fd: Option<OwnedFd>,
    keymap_sent: bool,
    keymap_format: u32,
    keymap_size: u32,
    xkb: Option<xkb::State>,
    engine: Engine,
    engine_enabled: Arc<AtomicBool>,
    preedit: String,
    last_serial: u32,
    active: bool,
    forwarded: HashSet<u32>,
}

impl ImState {
    fn new(engine_enabled: Arc<AtomicBool>, method: InputMethod) -> Self {
        Self {
            seat: None,
            im_manager: None,
            vk_manager: None,
            im: None,
            keyboard: None,
            vk: None,
            keymap_fd: None,
            keymap_sent: false,
            keymap_format: 1,
            keymap_size: 0,
            xkb: None,
            engine: Engine::new(method),
            engine_enabled,
            preedit: String::new(),
            last_serial: 0,
            active: false,
            forwarded: HashSet::new(),
        }
    }

    fn maybe_send_keymap_to_vk(&mut self) {
        if self.vk.is_some() && self.keymap_fd.is_some() && !self.keymap_sent {
            let fd = self.keymap_fd.as_ref().unwrap().as_fd();
            if let Some(vk) = &self.vk {
                vk.keymap(self.keymap_format, fd, self.keymap_size);
                self.keymap_sent = true;
            }
        }
    }

    fn handle_keymap(&mut self, fd: OwnedFd, size: u32) {
        // Keep a copy of the fd for the virtual keyboard.
        let raw = fd.as_raw_fd();
        let dup = unsafe { libc::dup(raw) };
        self.keymap_fd = if dup >= 0 {
            Some(unsafe { OwnedFd::from_raw_fd(dup) })
        } else {
            None
        };
        self.keymap_format = 1;
        self.keymap_size = size;

        let mut buf = vec![0u8; size as usize];
        use std::io::Read;
        if std::fs::File::from(fd).read_exact(&mut buf).is_ok() {
            let text = String::from_utf8_lossy(&buf);
            let ctx = xkb::Context::new(xkb::CONTEXT_NO_FLAGS);
            if let Some(keymap) = xkb::Keymap::new_from_string(
                &ctx,
                text.to_string(),
                xkb::KEYMAP_FORMAT_TEXT_V1,
                xkb::KEYMAP_COMPILE_NO_FLAGS,
            ) {
                self.xkb = Some(xkb::State::new(&keymap));
                eprintln!("[vietc-wayland] keymap loaded");
            }
        }
        self.maybe_send_keymap_to_vk();
    }

    fn send_preedit(&mut self) {
        if let Some(im) = &self.im {
            let end = self.preedit.len();
            im.set_preedit_string(self.preedit.clone(), end as i32, end as i32);
            im.commit_string(String::new());
            im.commit(self.last_serial);
        }
    }

    fn commit_preedit(&mut self) {
        if self.preedit.is_empty() {
            return;
        }
        if let Some(im) = &self.im {
            im.set_preedit_string(String::new(), 0, 0);
            im.commit_string(self.preedit.clone());
            im.commit(self.last_serial);
        }
        self.preedit.clear();
    }

    fn forward_press(&mut self, time: u32, key: u32) {
        if let Some(vk) = &self.vk {
            vk.key(time, key, 1);
        }
        self.forwarded.insert(key);
    }

    fn forward_release(&mut self, time: u32, key: u32) {
        if self.forwarded.remove(&key) {
            if let Some(vk) = &self.vk {
                vk.key(time, key, 0);
            }
        }
    }

    fn handle_key(
        &mut self,
        serial: u32,
        time: u32,
        key: u32,
        state: WEnum<KeyState>,
    ) {
        let pressed = matches!(state, WEnum::Value(KeyState::Pressed));
        if !pressed {
            self.forward_release(time, key);
            return;
        }

        if !self.active {
            return;
        }

        let Some(xkb) = self.xkb.as_ref() else {
            return;
        };

        self.last_serial = serial;

        // Ctrl/Alt combos are shortcuts — forward untouched, don't compose.
        let ctrl = xkb.mod_name_is_active(xkb::MOD_NAME_CTRL, xkb::STATE_MODS_EFFECTIVE);
        let alt = xkb.mod_name_is_active(xkb::MOD_NAME_ALT, xkb::STATE_MODS_EFFECTIVE);
        if ctrl || alt {
            self.forward_press(time, key);
            return;
        }

        let kc = xkb::Keycode::from(key + 8);
        let codepoint = xkb.key_get_utf32(kc);
        if codepoint == 0 {
            // Modifier or non-printable key: forward so the app behaves normally.
            self.forward_press(time, key);
            return;
        }

        let Some(ch) = char::from_u32(codepoint) else {
            return;
        };

        self.feed_char(ch, time, key);
    }

    fn feed_char(&mut self, ch: char, time: u32, key: u32) {
        let enabled = self.engine_enabled.load(Ordering::SeqCst);
        let preedit = self.preedit.clone();
        let actions = plan_char(enabled, &mut self.engine, &preedit, ch, key);
        for action in actions {
            match action {
                ImAction::SetPreedit(s) => {
                    self.preedit = s;
                    self.send_preedit();
                }
                ImAction::Commit(s) => {
                    self.preedit = s;
                    self.commit_preedit();
                }
                ImAction::ForwardKey(k) => {
                    self.forward_press(time, k);
                }
            }
        }
    }
}

fn is_flush_char(ch: char) -> bool {
    matches!(
        ch,
        ' ' | '\t' | '.' | ',' | '!' | '?' | ';' | ':' | '\n'
    )
}

/// Result of deciding what to do with a single composed character. Kept as data
/// so the mapping can be unit-tested without a live Wayland compositor.
#[derive(Debug, PartialEq, Eq)]
enum ImAction {
    /// Show `s` as the in-progress preedit (composing, not committed yet).
    SetPreedit(String),
    /// Finalize `s` into the focused app and clear the preedit.
    Commit(String),
    /// Forward the raw physical key (already grabbed, so the app needs it).
    ForwardKey(u32),
}

/// Decide how to present `ch` (from physical key `key`) to the app, given the
/// engine state and the current preedit. Mirrors how the evdev/daemon model
/// treats every raw keystroke as already on screen and the engine events as
/// corrections: here `None`/`Replace` keep a preedit, while `Flush`/
/// `AutoRestore`/`UndoTones`/`Paste` finalize, and flush characters commit the
/// word and let the separator through.
fn plan_char(
    enabled: bool,
    engine: &mut Engine,
    preedit: &str,
    ch: char,
    key: u32,
) -> Vec<ImAction> {
    if !enabled {
        let mut a = Vec::new();
        if !preedit.is_empty() {
            a.push(ImAction::Commit(preedit.to_string()));
        }
        a.push(ImAction::ForwardKey(key));
        return a;
    }

    let is_flush = is_flush_char(ch);
    let mut actions = Vec::new();
    match engine.process_key(ch) {
        Some(EngineEvent::Insert(_)) => {
            actions.push(ImAction::ForwardKey(key));
        }
        Some(EngineEvent::Replace { insert, .. }) => {
            if is_flush {
                actions.push(ImAction::Commit(insert));
                actions.push(ImAction::ForwardKey(key));
            } else {
                actions.push(ImAction::SetPreedit(insert));
            }
        }
        Some(EngineEvent::AutoRestore(s))
        | Some(EngineEvent::UndoTones { restored: s, .. })
        | Some(EngineEvent::Flush(s))
        | Some(EngineEvent::Paste(s)) => {
            actions.push(ImAction::Commit(s));
            if is_flush {
                actions.push(ImAction::ForwardKey(key));
            }
        }
        None => {
            if is_flush {
                if !preedit.is_empty() {
                    actions.push(ImAction::Commit(preedit.to_string()));
                }
                actions.push(ImAction::ForwardKey(key));
            } else {
                actions.push(ImAction::SetPreedit(engine.buffer()));
            }
        }
    }
    actions
}

impl Dispatch<WlRegistry, ()> for ImState {
    fn event(
        state: &mut Self,
        registry: &WlRegistry,
        event: WlRegistryEvent,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let WlRegistryEvent::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_seat" => {
                    let seat =
                        registry.bind::<WlSeat, (), ImState>(name, version.min(7), qh, ());
                    state.seat = Some(seat);
                }
                "zwp_input_method_manager_v2" => {
                    let mgr = registry.bind::<ZwpInputMethodManagerV2, (), ImState>(
                        name, 1, qh, (),
                    );
                    state.im_manager = Some(mgr);
                }
                "zwp_virtual_keyboard_manager_v1" => {
                    let mgr = registry.bind::<ZwpVirtualKeyboardManagerV1, (), ImState>(
                        name, 1, qh, (),
                    );
                    state.vk_manager = Some(mgr);
                }
                _ => {}
            }
        }
    }
}

impl Dispatch<WlSeat, ()> for ImState {
    fn event(
        _state: &mut Self,
        _proxy: &WlSeat,
        _event: wayland_client::protocol::wl_seat::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpInputMethodManagerV2, ()> for ImState {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpInputMethodManagerV2,
        _event: ManagerEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpInputMethodV2, ()> for ImState {
    fn event(
        state: &mut Self,
        proxy: &ZwpInputMethodV2,
        event: ImEvent,
        _data: &(),
        _conn: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        match event {
            ImEvent::Activate { .. } => {
                state.active = true;
                state.engine.reset();
                state.preedit.clear();
                let kb = proxy.grab_keyboard(qh, ());
                state.keyboard = Some(kb);
                eprintln!("[vietc-wayland] IM activated");
            }
            ImEvent::Deactivate { .. } => {
                state.active = false;
                if let Some(kb) = state.keyboard.take() {
                    kb.release();
                }
                // Finalize any in-progress word so it isn't lost on focus loss.
                state.commit_preedit();
                state.engine.reset();
                state.preedit.clear();
                eprintln!("[vietc-wayland] IM deactivated");
            }
            ImEvent::Done => {
                // The compositor finished applying our preedit/commit. Nothing to
                // do beyond acknowledging; serials come from key events.
            }
            ImEvent::Unavailable => {
                eprintln!("[vietc-wayland] IM unavailable (another IM active?)");
                state.active = false;
            }
            _ => {}
        }
    }
}

impl Dispatch<ZwpInputMethodKeyboardGrabV2, ()> for ImState {
    fn event(
        state: &mut Self,
        _proxy: &ZwpInputMethodKeyboardGrabV2,
        event: KbEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        match event {
            KbEvent::Keymap { format, fd, size } => {
                if matches!(format, WEnum::Value(KeymapFormat::XkbV1)) {
                    state.handle_keymap(fd, size);
                }
            }
            KbEvent::Key {
                serial,
                time,
                key,
                state: key_state,
            } => {
                state.handle_key(serial, time, key, key_state);
            }
            KbEvent::Modifiers {
                mods_depressed: depressed,
                mods_latched: latched,
                mods_locked: locked,
                group,
                ..
            } => {
                if let Some(xkb_state) = &mut state.xkb {
                    xkb_state.update_mask(
                        depressed.into(),
                        latched.into(),
                        locked.into(),
                        group.into(),
                        group.into(),
                        group.into(),
                    );
                }
                if let Some(vk) = &state.vk {
                    vk.modifiers(depressed, latched, locked, group);
                }
            }
            _ => {}
        }
    }
}

impl Dispatch<ZwpVirtualKeyboardManagerV1, ()> for ImState {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpVirtualKeyboardManagerV1,
        _event: VkMgrEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwpVirtualKeyboardV1, ()> for ImState {
    fn event(
        _state: &mut Self,
        _proxy: &ZwpVirtualKeyboardV1,
        _event: VkEvent,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }
}

/// Run vietc as a Wayland input method. Blocks until exit or a fatal error.
/// Returns Ok(()) when the loop ends normally (e.g. SIGINT), Err if the
/// compositor does not expose zwp_input_method_v2 so the caller can fall back
/// to the evdev/uinput path.
pub fn run_wayland_im(
    config: &Config,
    engine_enabled: Arc<AtomicBool>,
    _display: DisplayServer,
) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::connect_to_env()?;
    let mut event_queue: EventQueue<ImState> = conn.new_event_queue();
    let qh = event_queue.handle();

    let method = parse_method(&config.input_method);
    let mut state = ImState::new(engine_enabled, method);

    let _registry = conn.display().get_registry(&qh, ());

    // First dispatch: discover globals (wl_seat + managers).
    event_queue.roundtrip(&mut state)?;

    // Create the virtual keyboard used to forward non-composed keys.
    if let (Some(seat), Some(vkmgr)) = (state.seat.as_ref(), state.vk_manager.as_ref()) {
        state.vk = Some(vkmgr.create_virtual_keyboard(seat, &qh, ()));
        state.maybe_send_keymap_to_vk();
    }

    if state.seat.is_none() || state.im_manager.is_none() {
        eprintln!("[vietc-wayland] compositor lacks zwp_input_method_manager_v2; falling back");
        return Err("no input method manager".into());
    }

    // Request the input method object for this seat.
    if let (Some(seat), Some(mgr)) = (state.seat.as_ref(), state.im_manager.as_ref()) {
        state.im = Some(mgr.get_input_method(seat, &qh, ()));
    }

    // Second dispatch: receive activate + grab keyboard.
    event_queue.roundtrip(&mut state)?;

    eprintln!("[vietc-wayland] running as Wayland input method (no root required)");

    while !SIGNAL_EXIT.load(Ordering::SeqCst) {
        if event_queue.dispatch_pending(&mut state).is_err() {
            break;
        }
        std::thread::sleep(Duration::from_millis(20));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use vietc_engine::InputMethod;

    fn telex() -> Engine {
        Engine::new(InputMethod::Telex)
    }

    #[test]
    fn composing_keeps_preedit_until_flush() {
        let mut engine = telex();
        // 'a' -> preedit "a"
        let a = plan_char(true, &mut engine, "", 'a', 30);
        assert_eq!(a, vec![ImAction::SetPreedit("a".to_string())]);
        // 'w' tones it to "ă", still composing
        let w = plan_char(true, &mut engine, "a", 'w', 17);
        assert_eq!(w, vec![ImAction::SetPreedit("ă".to_string())]);
    }

    #[test]
    fn flush_char_commits_word_and_forwards_separator() {
        let mut engine = telex();
        plan_char(true, &mut engine, "", 'a', 30);
        plan_char(true, &mut engine, "a", 'w', 17);
        // space flushes: commit the composed word, forward the space key (57)
        let space = plan_char(true, &mut engine, "ă", ' ', 57);
        assert_eq!(
            space,
            vec![
                ImAction::Commit("ă".to_string()),
                ImAction::ForwardKey(57),
            ]
        );
    }

    #[test]
    fn full_word_stays_composed_until_flush() {
        let mut engine = telex();
        let keys: &[(char, u32)] = &[
            ('h', 35), ('o', 18), ('a', 30), ('n', 57), ('g', 10), ('w', 17),
        ];
        let mut preedit = String::new();
        let mut committed = Vec::new();
        for (ch, key) in keys {
            for action in plan_char(true, &mut engine, &preedit, *ch, *key) {
                match action {
                    ImAction::SetPreedit(s) => preedit = s,
                    ImAction::Commit(s) => {
                        committed.push(s);
                        preedit.clear();
                    }
                    ImAction::ForwardKey(_) => {}
                }
            }
        }
        // Everything typed so far is still in the preedit (composing), nothing
        // committed yet.
        assert!(!preedit.is_empty());
        assert!(committed.is_empty());
        // Flushing with space finalizes exactly the composed preedit.
        let flush = plan_char(true, &mut engine, &preedit, ' ', 57);
        assert_eq!(
            flush,
            vec![
                ImAction::Commit(preedit.clone()),
                ImAction::ForwardKey(57),
            ]
        );
    }

    #[test]
    fn disabled_forwards_without_composing() {
        let mut engine = telex();
        // no preedit yet -> just forward
        let a = plan_char(false, &mut engine, "", 'a', 30);
        assert_eq!(a, vec![ImAction::ForwardKey(30)]);
        // mid-word disabled -> commit pending preedit then forward
        let w = plan_char(false, &mut engine, "ă", 'w', 17);
        assert_eq!(
            w,
            vec![
                ImAction::Commit("ă".to_string()),
                ImAction::ForwardKey(17),
            ]
        );
    }

    #[test]
    fn punctuation_flushes_word() {
        let mut engine = telex();
        plan_char(true, &mut engine, "", 'a', 30);
        plan_char(true, &mut engine, "a", 'w', 17);
        // '.' commits the word and forwards the punctuation key
        let dot = plan_char(true, &mut engine, "ă", '.', 52);
        assert_eq!(
            dot,
            vec![
                ImAction::Commit("ă".to_string()),
                ImAction::ForwardKey(52),
            ]
        );
    }

    /// Drive `plan_char` over a full key sequence (like a virtual keyboard
    /// would deliver keycodes) and collect only what reaches the
    /// zwp_virtual_keyboard_v1 (ForwardKey). This mirrors what vietc-vk /
    /// the compositor would actually forward to the focused app.
    fn virtual_keyboard_keys(enabled: bool, seq: &[(char, u32)]) -> Vec<u32> {
        let mut engine = telex();
        let mut preedit = String::new();
        let mut forwarded = Vec::new();
        for &(ch, key) in seq {
            for action in plan_char(enabled, &mut engine, &preedit, ch, key) {
                match action {
                    ImAction::SetPreedit(s) => preedit = s,
                    ImAction::Commit(s) => {
                        // committed text goes through the IM, not the vk
                        let _ = s;
                        preedit.clear();
                    }
                    ImAction::ForwardKey(k) => forwarded.push(k),
                }
            }
        }
        forwarded
    }

    #[test]
    fn virtual_keyboard_gets_only_separators_for_composed_word() {
        // a=30, w=17 compose to "ă"; space=57 is the only key forwarded.
        let keys = virtual_keyboard_keys(true, &[('a', 30), ('w', 17), (' ', 57)]);
        assert_eq!(keys, vec![57], "composing keys must NOT reach the virtual keyboard");
    }

    #[test]
    fn virtual_keyboard_gets_only_space_for_plain_word() {
        // "hello" (h=35,o=18,l=38,n=57,g=10) is not Vietnamese, but while the
        // engine is composing it stays in the preedit; only the space forwards.
        let keys = virtual_keyboard_keys(
            true,
            &[('h', 35), ('e', 18), ('l', 38), ('l', 38), ('o', 24), (' ', 57)],
        );
        assert_eq!(keys, vec![57], "plain word should forward only the separator");
    }

    #[test]
    fn virtual_keyboard_forwards_everything_when_disabled() {
        // With VN disabled, every raw key goes straight to the virtual keyboard.
        let keys = virtual_keyboard_keys(false, &[('a', 30), ('w', 17), (' ', 57)]);
        assert_eq!(keys, vec![30, 17, 57]);
    }
}
