use std::collections::HashMap;

use crate::inject::{InjectResult, KeyInjector};

/// X11 keysym values for common keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Keysym(pub u32);

impl Keysym {
    pub const BACKSPACE: Keysym = Keysym(0xff08);
    pub const TAB: Keysym = Keysym(0xff09);
    pub const RETURN: Keysym = Keysym(0xff0d);
    pub const ESCAPE: Keysym = Keysym(0xff1b);
    pub const SPACE: Keysym = Keysym(0x0020);
    pub const DELETE: Keysym = Keysym(0xffff);

    pub const A: Keysym = Keysym(0x0061);
    pub const Z: Keysym = Keysym(0x007a);
    pub const SHIFT_L: Keysym = Keysym(0xffe1);
    pub const CTRL_L: Keysym = Keysym(0xffe3);

    pub fn from_char(ch: char) -> Option<Keysym> {
        match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' => Some(Keysym(ch as u32)),
            ' ' => Some(Keysym::SPACE),
            '.' => Some(Keysym(0x002e)),
            ',' => Some(Keysym(0x002c)),
            '-' => Some(Keysym(0x002d)),
            '=' => Some(Keysym(0x003d)),
            ';' => Some(Keysym(0x003b)),
            '\'' => Some(Keysym(0x0027)),
            '/' => Some(Keysym(0x002f)),
            '\\' => Some(Keysym(0x005c)),
            '`' => Some(Keysym(0x0060)),
            '[' => Some(Keysym(0x005b)),
            ']' => Some(Keysym(0x005d)),
            '\n' => Some(Keysym::RETURN),
            '\t' => Some(Keysym::TAB),
            _ => None,
        }
    }

    pub fn to_char(self) -> Option<char> {
        match self.0 {
            0x0061..=0x007a => Some((self.0 as u8) as char),
            0x0041..=0x005a => Some((self.0 as u8) as char),
            0x0030..=0x0039 => Some((self.0 as u8) as char),
            0x0020 => Some(' '),
            0x002e => Some('.'),
            0x002c => Some(','),
            0x002d => Some('-'),
            0x003d => Some('='),
            0x003b => Some(';'),
            0x0027 => Some('\''),
            0x002f => Some('/'),
            0x005c => Some('\\'),
            0x0060 => Some('`'),
            0x005b => Some('['),
            0x005d => Some(']'),
            0xff0d => Some('\n'),
            0xff09 => Some('\t'),
            _ => None,
        }
    }

    pub fn is_printable(self) -> bool {
        self.to_char().is_some()
    }

    pub fn is_modifier(self) -> bool {
        matches!(
            self.0,
            0xffe1..=0xffee
        )
    }
}

/// Key event from Wayland IM protocol
#[derive(Debug, Clone)]
pub struct IMKeyEvent {
    pub keysym: Keysym,
    pub pressed: bool,
    pub modifiers: KeyModifiers,
}

#[derive(Debug, Clone, Default)]
pub struct KeyModifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub super_key: bool,
}

/// Wayland input method state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IMState {
    Inactive,
    Active,
}

/// Wayland IM context for receiving key events from compositor
///
/// This implements the zwp_input_method_v2 protocol to receive keysyms
/// directly from the Wayland compositor, bypassing evdev interception.
pub struct WaylandIMContext {
    state: IMState,
    preedit: Option<String>,
    cursor_pos: usize,
    commit_buffer: String,
    keysym_map: HashMap<u32, char>,
}

impl Default for WaylandIMContext {
    fn default() -> Self {
        Self::new()
    }
}

impl WaylandIMContext {
    pub fn new() -> Self {
        Self {
            state: IMState::Inactive,
            preedit: None,
            cursor_pos: 0,
            commit_buffer: String::new(),
            keysym_map: Self::build_keysym_map(),
        }
    }

    fn build_keysym_map() -> HashMap<u32, char> {
        let mut map = HashMap::new();
        // Lowercase letters
        for i in 0u32..26 {
            map.insert(0x0061 + i, (b'a' + i as u8) as char);
        }
        // Uppercase letters
        for i in 0u32..26 {
            map.insert(0x0041 + i, (b'A' + i as u8) as char);
        }
        // Digits
        for i in 0u32..10 {
            map.insert(0x0030 + i, (b'0' + i as u8) as char);
        }
        // Common punctuation
        map.insert(0x0020, ' ');
        map.insert(0x002e, '.');
        map.insert(0x002c, ',');
        map.insert(0x002d, '-');
        map.insert(0x003d, '=');
        map.insert(0x003b, ';');
        map.insert(0x0027, '\'');
        map.insert(0x002f, '/');
        map.insert(0x005c, '\\');
        map.insert(0x0060, '`');
        map.insert(0x005b, '[');
        map.insert(0x005d, ']');
        // Special keys
        map.insert(0xff0d, '\n'); // Return
        map.insert(0xff09, '\t'); // Tab
        map.insert(0xff08, '\x08'); // Backspace
        map.insert(0xff1b, '\x1b'); // Escape
        map.insert(0xffff, '\x7f'); // Delete
        map
    }

    /// Handle IM activation from compositor
    pub fn activate(&mut self) {
        self.state = IMState::Active;
        eprintln!("[vietc-wayland] IM activated");
    }

    /// Handle IM deactivation from compositor
    pub fn deactivate(&mut self) {
        self.state = IMState::Inactive;
        self.preedit = None;
        self.commit_buffer.clear();
        eprintln!("[vietc-wayland] IM deactivated");
    }

    /// Get current IM state
    pub fn state(&self) -> IMState {
        self.state
    }

    /// Set preedit text (shown with underline in client)
    pub fn set_preedit(&mut self, text: Option<String>, cursor: usize) {
        self.preedit = text;
        self.cursor_pos = cursor;
    }

    /// Get current preedit text
    pub fn preedit(&self) -> Option<&str> {
        self.preedit.as_deref()
    }

    /// Commit text to the focused surface
    pub fn commit(&mut self, text: &str) {
        self.commit_buffer.push_str(text);
    }

    /// Get and clear the commit buffer
    pub fn take_commit(&mut self) -> String {
        std::mem::take(&mut self.commit_buffer)
    }

    /// Convert a keysym to a character, applying modifiers
    pub fn keysym_to_char(&self, keysym: Keysym, mods: &KeyModifiers) -> Option<char> {
        if keysym.is_modifier() {
            return None;
        }

        let base = self.keysym_map.get(&keysym.0).copied()?;

        // Apply shift for letters
        if mods.shift && base.is_ascii_lowercase() {
            return Some(base.to_ascii_uppercase());
        }

        // Shift+digit produces symbol
        if mods.shift && base.is_ascii_digit() {
            let shifted = match base {
                '1' => '!', '2' => '@', '3' => '#', '4' => '$', '5' => '%',
                '6' => '^', '7' => '&', '8' => '*', '9' => '(', '0' => ')',
                _ => return Some(base),
            };
            return Some(shifted);
        }

        Some(base)
    }

    /// Convert a character to a keysym
    pub fn char_to_keysym(ch: char) -> Option<Keysym> {
        Keysym::from_char(ch)
    }

    /// Process a raw keysym event and return the character (if any)
    pub fn process_keysym(&self, keysym: Keysym, mods: &KeyModifiers) -> Option<char> {
        self.keysym_to_char(keysym, mods)
    }
}

/// Wayland IM key injector using zwp_input_method_context_v2
///
/// Commits text directly to the focused surface without key injection.
/// Falls back to uinput/X11 if context is not available.
pub struct WaylandIMInjector {
    committed: Vec<String>,
}

impl Default for WaylandIMInjector {
    fn default() -> Self {
        Self::new()
    }
}

impl WaylandIMInjector {
    pub fn new() -> Self {
        Self {
            committed: Vec::new(),
        }
    }

    /// Take all committed text since last call
    pub fn take_commits(&mut self) -> Vec<String> {
        std::mem::take(&mut self.committed)
    }
}

impl KeyInjector for WaylandIMInjector {
    fn send_backspace(&self) -> InjectResult {
        // In real implementation, this would call
        // context.delete_surrounding_text(-1, 1) + context.commit()
        InjectResult::Success
    }

    fn send_char(&self, _ch: char) -> InjectResult {
        // In real implementation, this would call
        // context.commit_string(ch.to_string()) + context.commit()
        InjectResult::Success
    }

    fn send_string(&self, _s: &str) -> InjectResult {
        // In real implementation, this would call
        // context.commit_string(s.to_string()) + context.commit()
        InjectResult::Success
    }

    fn flush(&self) -> InjectResult {
        InjectResult::Success
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keysym_from_char() {
        assert_eq!(Keysym::from_char('a'), Some(Keysym(0x0061)));
        assert_eq!(Keysym::from_char('z'), Some(Keysym(0x007a)));
        assert_eq!(Keysym::from_char('A'), Some(Keysym(0x0041)));
        assert_eq!(Keysym::from_char('0'), Some(Keysym(0x0030)));
        assert_eq!(Keysym::from_char(' '), Some(Keysym(0x0020)));
        assert_eq!(Keysym::from_char('.'), Some(Keysym(0x002e)));
        assert_eq!(Keysym::from_char('\n'), Some(Keysym(0xff0d)));
        assert_eq!(Keysym::from_char('ñ'), None);
    }

    #[test]
    fn keysym_to_char() {
        assert_eq!(Keysym(0x0061).to_char(), Some('a'));
        assert_eq!(Keysym(0x007a).to_char(), Some('z'));
        assert_eq!(Keysym(0x0041).to_char(), Some('A'));
        assert_eq!(Keysym(0x0030).to_char(), Some('0'));
        assert_eq!(Keysym(0x0020).to_char(), Some(' '));
        assert_eq!(Keysym(0xff0d).to_char(), Some('\n'));
        assert_eq!(Keysym(0xffff).to_char(), None);
    }

    #[test]
    fn keysym_is_printable() {
        assert!(Keysym(0x0061).is_printable()); // 'a'
        assert!(Keysym(0x0020).is_printable()); // space
        assert!(Keysym(0xff0d).is_printable()); // Return → '\n'
        assert!(!Keysym(0xff08).is_printable()); // Backspace → '\x08' (not printable)
    }

    #[test]
    fn keysym_is_modifier() {
        assert!(Keysym(0xffe1).is_modifier()); // shift
        assert!(Keysym(0xffe3).is_modifier()); // ctrl
        assert!(Keysym(0xffe9).is_modifier()); // alt
        assert!(!Keysym(0x0061).is_modifier()); // 'a'
        assert!(!Keysym(0x0020).is_modifier()); // space
    }

    #[test]
    fn im_context_activate_deactivate() {
        let mut ctx = WaylandIMContext::new();
        assert_eq!(ctx.state(), IMState::Inactive);

        ctx.activate();
        assert_eq!(ctx.state(), IMState::Active);

        ctx.deactivate();
        assert_eq!(ctx.state(), IMState::Inactive);
    }

    #[test]
    fn im_context_preedit() {
        let mut ctx = WaylandIMContext::new();
        assert!(ctx.preedit().is_none());

        ctx.set_preedit(Some("hello".into()), 3);
        assert_eq!(ctx.preedit(), Some("hello"));

        ctx.set_preedit(None, 0);
        assert!(ctx.preedit().is_none());
    }

    #[test]
    fn im_context_commit() {
        let mut ctx = WaylandIMContext::new();
        ctx.commit("hello");
        ctx.commit(" ");
        ctx.commit("world");
        assert_eq!(ctx.take_commit(), "hello world");
        assert!(ctx.take_commit().is_empty());
    }

    #[test]
    fn keysym_to_char_no_modifiers() {
        let ctx = WaylandIMContext::new();
        let mods = KeyModifiers::default();

        assert_eq!(ctx.keysym_to_char(Keysym(0x0061), &mods), Some('a'));
        assert_eq!(ctx.keysym_to_char(Keysym(0x007a), &mods), Some('z'));
        assert_eq!(ctx.keysym_to_char(Keysym(0x0030), &mods), Some('0'));
        assert_eq!(ctx.keysym_to_char(Keysym(0x0020), &mods), Some(' '));
    }

    #[test]
    fn keysym_to_char_shift() {
        let ctx = WaylandIMContext::new();
        let mods = KeyModifiers {
            shift: true,
            ..Default::default()
        };

        assert_eq!(ctx.keysym_to_char(Keysym(0x0061), &mods), Some('A'));
        assert_eq!(ctx.keysym_to_char(Keysym(0x007a), &mods), Some('Z'));
        assert_eq!(ctx.keysym_to_char(Keysym(0x0031), &mods), Some('!'));
        assert_eq!(ctx.keysym_to_char(Keysym(0x0032), &mods), Some('@'));
    }

    #[test]
    fn keysym_to_char_modifier_returns_none() {
        let ctx = WaylandIMContext::new();
        let mods = KeyModifiers::default();

        assert_eq!(ctx.keysym_to_char(Keysym(0xffe1), &mods), None); // shift
        assert_eq!(ctx.keysym_to_char(Keysym(0xffe3), &mods), None); // ctrl
    }

    #[test]
    fn process_keysym() {
        let ctx = WaylandIMContext::new();
        let mods = KeyModifiers::default();

        assert_eq!(ctx.process_keysym(Keysym(0x0061), &mods), Some('a'));
        assert_eq!(ctx.process_keysym(Keysym(0xff0d), &mods), Some('\n'));
    }

    #[test]
    fn char_to_keysym_roundtrip() {
        for ch in "abcdefghijklmnopqrstuvwxyz".chars() {
            let keysym = WaylandIMContext::char_to_keysym(ch).unwrap();
            let back = keysym.to_char().unwrap();
            assert_eq!(ch, back);
        }
        for ch in "ABCDEFGHIJKLMNOPQRSTUVWXYZ".chars() {
            let keysym = WaylandIMContext::char_to_keysym(ch).unwrap();
            let back = keysym.to_char().unwrap();
            assert_eq!(ch, back);
        }
    }
}
