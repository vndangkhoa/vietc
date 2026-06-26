use crate::english::EnglishDict;
use crate::telex::TelexEngine;
use crate::vni::VniEngine;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
pub enum InputMethod {
    Telex,
    Vni,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum EngineEvent {
    Replace {
        backspaces: usize,
        insert: String,
    },
    Insert(String),
    Flush(String),
    AutoRestore(String),
    /// ESC undo: strip all tone marks from current word
    UndoTones {
        backspaces: usize,
        restored: String,
    },
    /// Text was pasted via clipboard - update buffer directly without telex parsing
    Paste(String),
}

pub struct Engine {
    input_method: InputMethod,
    telex: TelexEngine,
    vni: VniEngine,
    english: EnglishDict,
    enabled: bool,
    macros: std::collections::HashMap<String, String>,
    raw_buffer: String,
    /// Flag to bypass telex/vni parsing when Unicode text has been pasted via clipboard
    paste_mode: bool,
}

impl Engine {
    pub fn new(method: InputMethod) -> Self {
        Self {
            input_method: method,
            telex: TelexEngine::new(),
            vni: VniEngine::new(),
            english: EnglishDict::new(),
            enabled: true,
            macros: std::collections::HashMap::new(),
            raw_buffer: String::new(),
            paste_mode: false,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.flush();
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_method(&mut self, method: InputMethod) {
        self.input_method = method;
        self.reset();
    }

    /// Enter "paste mode" - bypass telex/vni parsing for Unicode pasted text
    pub fn enter_paste_mode(&mut self) {
        self.paste_mode = true;
    }

    /// Exit paste mode (for Paste event handling)
    pub fn exit_paste_mode(&mut self) {
        self.paste_mode = false;
    }

    /// Paste raw text into buffer without telex/vni processing
    pub fn paste(&mut self, text: &str) -> EngineEvent {
        // Clear buffer if entering paste mode and exit paste mode after
        if self.paste_mode {
            self.raw_buffer.clear();
        } else {
            self.enter_paste_mode();
        }

        let event = EngineEvent::Paste(text.to_string());
        self.raw_buffer.push_str(text);
        event
    }

    /// Replay a sequence of keystrokes through a fresh engine and return the
    /// final screen output. This is the core of the Backspace-Replay pattern:
    /// instead of tracking incremental state, we always recompute from scratch.
    /// Returns (output_on_screen, did_flush).
    /// `did_flush` means the engine processed a word boundary and the cursor
    /// is now at a clean position — caller should clear keystroke history.
    pub fn replay_keystrokes(
        method: InputMethod,
        macros: &std::collections::HashMap<String, String>,
        keystrokes: &[char],
    ) -> (String, bool) {
        let mut engine = Engine::new(method);
        for (shortcut, expansion) in macros {
            engine.add_macro(shortcut.clone(), expansion.clone());
        }

        let mut last_output = String::new();
        let mut did_flush = false;

        for &ch in keystrokes {
            if let Some(event) = engine.process_key(ch) {
                match event {
                    EngineEvent::Replace { insert, .. } => {
                        last_output = insert;
                    }
                    EngineEvent::Flush(_word) => {
                        // Word was flushed. The flush char is NOT part of the word.
                        // The word is committed; clear tracking for current composing.
                        last_output.clear();
                        did_flush = true;
                    }
                    EngineEvent::Insert(text) => {
                        last_output = text;
                    }
                    EngineEvent::UndoTones { restored, .. } => {
                        last_output = restored;
                    }
                    EngineEvent::Paste(text) => {
                        last_output = text;
                    }
                    EngineEvent::AutoRestore(word) => {
                        last_output = word;
                    }
                }
            } else {
                // Key consumed but no screen change — buffer is building
                let buf = engine.buffer().to_string();
                if !buf.is_empty() {
                    last_output = buf;
                }
            }
        }

        // If the engine has a buffer that hasn't been flushed, that's on screen
        let buf = engine.buffer().to_string();
        if !buf.is_empty() {
            last_output = buf;
            did_flush = false; // Still composing
        } else if did_flush {
            // After flush, nothing is on screen for the composing word
            last_output.clear();
        }

        (last_output, did_flush)
    }

    /// Update buffer with pasted text for subsequent edit operations (delete/backspace)
    pub fn update_with_pasted_text(&mut self, text: &str) {
        self.raw_buffer.clear();
        self.raw_buffer.push_str(text);
    }

    pub fn reset(&mut self) {
        self.telex.reset();
        self.vni.reset();
        self.raw_buffer.clear();
    }

    pub fn flush(&mut self) -> Option<EngineEvent> {
        // If in paste mode, bypass telex/vni parsing and return raw text as-is
        if self.paste_mode && !self.raw_buffer.is_empty() {
            // Only set paste_mode if buffer contains non-ASCII Unicode chars (pasted content)
            let has_unicode = self.raw_buffer.chars().any(|c| !c.is_ascii());
            if has_unicode {
                let word = self.raw_buffer.clone();
                self.raw_buffer.clear();
                self.paste_mode = false; // Exit paste mode after flush
                return Some(EngineEvent::Flush(word));
            }
        }

        let event = match self.input_method {
            InputMethod::Telex => self.telex.flush(),
            InputMethod::Vni => self.vni.flush(),
        };
        if let Some(EngineEvent::Flush(word)) = event {
            let cased = match_casing(&self.raw_buffer, &word);
            self.raw_buffer.clear();
            Some(EngineEvent::Flush(cased))
        } else {
            event
        }
    }

    /// Add a macro shortcut
    pub fn add_macro(&mut self, shortcut: String, expansion: String) {
        self.macros.insert(shortcut, expansion);
    }

    /// Clear all macros
    pub fn clear_macros(&mut self) {
        self.macros.clear();
    }

    /// Process ESC key - undo tones from current word
    pub fn process_escape(&mut self) -> Option<EngineEvent> {
        let buffer = match self.input_method {
            InputMethod::Telex => self.telex.buffer(),
            InputMethod::Vni => self.vni.buffer(),
        };

        if buffer.is_empty() {
            return None;
        }

        // Strip all diacritics from the buffer
        let stripped = strip_diacritics(buffer);
        let backspaces = buffer.chars().count();
        let had_tones = stripped != buffer;
        let cased_stripped = match_casing(&self.raw_buffer, &stripped);
        self.reset();

        if had_tones {
            Some(EngineEvent::UndoTones {
                backspaces,
                restored: cased_stripped,
            })
        } else {
            Some(EngineEvent::Flush(cased_stripped))
        }
    }

    pub fn process_key(&mut self, ch: char) -> Option<EngineEvent> {
        if !self.enabled {
            return None;
        }

        // ESC = undo tones
        if ch == '\x1b' {
            return self.process_escape();
        }

        if ch == '\x08' {
            // Backspace handling: pop from inner engine and sync raw_buffer
            match self.input_method {
                InputMethod::Telex => self.telex.pop(),
                InputMethod::Vni => self.vni.pop(),
            }
            let inner_len = self.buffer().chars().count();
            // Truncate raw_buffer to match inner engine buffer's character count
            let char_indices: Vec<(usize, char)> = self.raw_buffer.char_indices().collect();
            if char_indices.len() > inner_len {
                if inner_len == 0 {
                    self.raw_buffer.clear();
                } else {
                    let cut_idx = char_indices[inner_len].0;
                    self.raw_buffer.truncate(cut_idx);
                }
            }
            return None;
        }

        let lowercase_ch = if ch.is_ascii() {
            ch.to_ascii_lowercase()
        } else {
            ch.to_lowercase().next().unwrap_or(ch)
        };

        if lowercase_ch == ' '
            || lowercase_ch == '\t'
            || lowercase_ch == '.'
            || lowercase_ch == ','
            || lowercase_ch == '!'
            || lowercase_ch == '?'
            || lowercase_ch == ';'
            || lowercase_ch == ':'
            || lowercase_ch == '\n'
        {
            if self.raw_buffer.is_empty() {
                return None;
            }

            // Check for macro expansion before auto-restore
            let macro_expansion = self.macros.get(&self.raw_buffer.to_lowercase()).cloned();
            if let Some(expansion) = macro_expansion {
                let previous_raw_len = self.raw_buffer.chars().count();
                self.reset();
                return Some(EngineEvent::Replace {
                    backspaces: previous_raw_len + 1,
                    insert: format!("{}{}", expansion, ch),
                });
            }

            // Try auto-restore before flushing
            let clean_raw = self.raw_buffer.to_lowercase();
            let inner_buf = self.buffer().to_string();
            let clean_inner = strip_diacritics(&inner_buf).to_lowercase();
            let has_diacritics = clean_inner != inner_buf.to_lowercase();

            let should_restore = self.english.should_restore(&clean_raw)
                || (has_diacritics && !crate::spelling::is_valid_vietnamese_syllable(&inner_buf));

            if should_restore {
                let original_raw = self.raw_buffer.clone();
                let inner_len = inner_buf.chars().count();
                self.reset();

                if has_diacritics {
                    return Some(EngineEvent::Replace {
                        backspaces: inner_len + 1,
                        insert: format!("{}{}", original_raw, ch),
                    });
                } else {
                    return None;
                }
            }

            // Flush buffer with trailing character
            let previous_inner = self.buffer().to_string();
            let previous_inner_len = previous_inner.chars().count();

            let previous_inner_cased = match_casing(&self.raw_buffer, &previous_inner);
            let flush_event = self.flush();
            let mut final_word = previous_inner_cased.clone();
            if let Some(EngineEvent::Flush(word)) = flush_event {
                final_word = word;
            }

            let result = if final_word != previous_inner_cased {
                Some(EngineEvent::Replace {
                    backspaces: previous_inner_len + 1,
                    insert: format!("{}{}", final_word, ch),
                })
            } else {
                None
            };

            self.reset();
            return result;
        }

        let previous_inner = self.buffer().to_string();
        self.raw_buffer.push(ch);

        let expected_screen = format!("{}{}", previous_inner, lowercase_ch);

        if self.paste_mode {
            if ch.is_ascii() {
                match self.input_method {
                    InputMethod::Telex => {
                        self.telex.process_key(lowercase_ch);
                    }
                    InputMethod::Vni => {
                        self.vni.process_key(lowercase_ch);
                    }
                }
                None
            } else {
                Some(EngineEvent::Replace {
                    backspaces: previous_inner.chars().count() + 1,
                    insert: ch.to_string(),
                })
            }
        } else {
            match self.input_method {
                InputMethod::Telex => {
                    self.telex.process_key(lowercase_ch);
                }
                InputMethod::Vni => {
                    self.vni.process_key(lowercase_ch);
                }
            }

            let new_inner = self.buffer().to_string();
            if new_inner != expected_screen {
                let cased_inner = match_casing(&self.raw_buffer, &new_inner);
                Some(EngineEvent::Replace {
                    backspaces: previous_inner.chars().count() + 1,
                    insert: cased_inner,
                })
            } else {
                None
            }
        }
    }

    pub fn buffer(&self) -> &str {
        match self.input_method {
            InputMethod::Telex => self.telex.buffer(),
            InputMethod::Vni => self.vni.buffer(),
        }
    }
}

/// Strip all Vietnamese diacritics from a string, returning base ASCII
fn strip_diacritics(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            // a variants
            'à' | 'á' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ' | 'â' | 'ầ' | 'ấ'
            | 'ẩ' | 'ẫ' | 'ậ' => 'a',
            // A variants
            'À' | 'Á' | 'Ả' | 'Ã' | 'Ạ' | 'Ă' | 'Ằ' | 'Ắ' | 'Ẳ' | 'Ẵ' | 'Ặ' | 'Â' | 'Ầ' | 'Ấ'
            | 'Ẩ' | 'Ẫ' | 'Ậ' => 'A',
            // e variants
            'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ' => {
                'e'
            }
            'È' | 'É' | 'Ẻ' | 'Ẽ' | 'Ẹ' | 'Ê' | 'Ề' | 'Ế' | 'Ể' | 'Ễ' | 'Ệ' => {
                'E'
            }
            // i variants
            'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị' => 'i',
            'Ì' | 'Í' | 'Ỉ' | 'Ĩ' | 'Ị' => 'I',
            // o variants
            'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ' | 'ơ' | 'ờ' | 'ớ'
            | 'ở' | 'ỡ' | 'ợ' => 'o',
            'Ò' | 'Ó' | 'Ỏ' | 'Õ' | 'Ọ' | 'Ô' | 'Ồ' | 'Ố' | 'Ổ' | 'Ỗ' | 'Ộ' | 'Ơ' | 'Ờ' | 'Ớ'
            | 'Ở' | 'Ỡ' | 'Ợ' => 'O',
            // u variants
            'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự' => {
                'u'
            }
            'Ù' | 'Ú' | 'Ủ' | 'Ũ' | 'Ụ' | 'Ư' | 'Ừ' | 'Ứ' | 'Ử' | 'Ữ' | 'Ự' => {
                'U'
            }
            // y variants
            'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
            'Ỳ' | 'Ý' | 'Ỷ' | 'Ỹ' | 'Ỵ' => 'Y',
            // đ
            'đ' => 'd',
            'Đ' => 'D',
            // Everything else unchanged
            other => other,
        })
        .collect()
}

fn match_casing(raw: &str, processed: &str) -> String {
    if raw.is_empty() || processed.is_empty() {
        return processed.to_string();
    }

    let alphabetic_chars: Vec<char> = raw.chars().filter(|c| c.is_alphabetic()).collect();
    if alphabetic_chars.is_empty() {
        return processed.to_string();
    }

    let all_upper = alphabetic_chars.iter().all(|c| c.is_uppercase());
    let first_upper = alphabetic_chars[0].is_uppercase();

    if all_upper {
        processed.to_uppercase()
    } else if first_upper {
        let mut chars = processed.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => processed.to_string(),
        }
    } else {
        processed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_diacritics() {
        assert_eq!(strip_diacritics("chào"), "chao");
        assert_eq!(strip_diacritics("cám ơn"), "cam on");
        assert_eq!(strip_diacritics("Việt Nam"), "Viet Nam");
        assert_eq!(strip_diacritics("hello"), "hello");
        assert_eq!(strip_diacritics("đường"), "duong");
        assert_eq!(strip_diacritics("Nguyễn"), "Nguyen");
    }

    #[test]
    fn test_esc_undo_tones() {
        let mut engine = Engine::new(InputMethod::Telex);

        // Type "chào" then ESC
        for ch in "chào".chars() {
            engine.process_key(ch);
        }
        let event = engine.process_escape();
        match event {
            Some(EngineEvent::UndoTones {
                backspaces,
                restored,
            }) => {
                assert_eq!(backspaces, 4); // "chào" is 4 chars
                assert_eq!(restored, "chao");
            }
            _ => panic!("Expected UndoTones event, got {:?}", event),
        }
    }

    #[test]
    fn test_macro_expansion() {
        let mut engine = Engine::new(InputMethod::Telex);
        engine.add_macro("ko".into(), "không".into());
        engine.add_macro("ok".into(), "được".into());

        // Type "ko" + space
        let events: Vec<_> = "ko "
            .chars()
            .filter_map(|ch| engine.process_key(ch))
            .collect();

        // Should contain the macro expansion
        let output: String = events
            .iter()
            .filter_map(|e| match e {
                EngineEvent::Flush(s) => Some(s.as_str()),
                EngineEvent::Insert(s) => Some(s.as_str()),
                EngineEvent::Replace { insert, .. } => Some(insert.as_str()),
                _ => None,
            })
            .collect();

        assert!(output.contains("không"));
    }

    #[test]
    fn test_casing_preservation() {
        let mut engine = Engine::new(InputMethod::Telex);

        // Lowercase: "sats" -> "sát"
        engine.reset();
        let _ = engine.process_key('s');
        let _ = engine.process_key('a');
        let _ = engine.process_key('t');
        let _ = engine.process_key('s');
        assert_eq!(engine.buffer(), "sát");

        // Titlecase: "Sats" -> "Sát"
        engine.reset();
        engine.process_key('S');
        engine.process_key('a');
        engine.process_key('t');
        let event = engine.process_key('s');
        if let Some(EngineEvent::Replace { insert, .. }) = event {
            assert_eq!(insert, "Sát");
        } else {
            panic!("Expected Replace event, got {:?}", event);
        }

        // Uppercase: "SATS" -> "SÁT"
        engine.reset();
        engine.process_key('S');
        engine.process_key('A');
        engine.process_key('T');
        let event2 = engine.process_key('S');
        if let Some(EngineEvent::Replace { insert, .. }) = event2 {
            assert_eq!(insert, "SÁT");
        } else {
            panic!("Expected Replace event, got {:?}", event2);
        }
    }

    #[test]
    fn test_replay_keystrokes_telex() {
        let macros = std::collections::HashMap::new();

        // Replay "chao" -> should produce "chao" (no tone yet)
        let (output, flush) = Engine::replay_keystrokes(
            InputMethod::Telex,
            &macros,
            &['c', 'h', 'a', 'o'],
        );
        assert_eq!(output, "chao");
        assert!(!flush);

        // Replay "chaos" -> s adds acute accent: "cháo"
        let (output, flush) = Engine::replay_keystrokes(
            InputMethod::Telex,
            &macros,
            &['c', 'h', 'a', 'o', 's'],
        );
        assert_eq!(output, "cháo");
        assert!(!flush);

        // Replay "chaof" -> f adds grave accent: "chào"
        let (output, flush) = Engine::replay_keystrokes(
            InputMethod::Telex,
            &macros,
            &['c', 'h', 'a', 'o', 'f'],
        );
        assert_eq!(output, "chào");
        assert!(!flush);
    }

    #[test]
    fn test_replay_keystrokes_backspace() {
        let macros = std::collections::HashMap::new();

        // Replay "chaos" then backspace -> engine pops 'o' from "cháo" → "chá"
        let (output, _) = Engine::replay_keystrokes(
            InputMethod::Telex,
            &macros,
            &['c', 'h', 'a', 'o', 's', '\x08'],
        );
        assert_eq!(output, "chá");
    }

    #[test]
    fn test_replay_keystrokes_vni() {
        let macros = std::collections::HashMap::new();

        // VNI: "chao1" → acute accent on last vowel
        let (output, _) = Engine::replay_keystrokes(
            InputMethod::Vni,
            &macros,
            &['c', 'h', 'a', 'o', '1'],
        );
        // Verify it produces accented output (engine applies tone to last vowel)
        assert!(output.contains('á') || output.contains('ó'), "Expected toned output, got: {}", output);
    }
}
