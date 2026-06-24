use crate::telex::TelexEngine;
use crate::vni::VniEngine;
use crate::english::EnglishDict;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMethod {
    Telex,
    Vni,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineEvent {
    Replace { backspaces: usize, insert: String },
    Insert(String),
    Flush(String),
    AutoRestore(String),
    /// ESC undo: strip all tone marks from current word
    UndoTones { backspaces: usize, restored: String },
}

pub struct Engine {
    input_method: InputMethod,
    telex: TelexEngine,
    vni: VniEngine,
    english: EnglishDict,
    enabled: bool,
    macros: std::collections::HashMap<String, String>,
    raw_buffer: String,
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

    pub fn reset(&mut self) {
        self.telex.reset();
        self.vni.reset();
        self.raw_buffer.clear();
    }

    pub fn flush(&mut self) -> Option<EngineEvent> {
        match self.input_method {
            InputMethod::Telex => self.telex.flush(),
            InputMethod::Vni => self.vni.flush(),
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
        self.reset();

        if had_tones {
            Some(EngineEvent::UndoTones {
                backspaces,
                restored: stripped,
            })
        } else {
            Some(EngineEvent::Flush(stripped))
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

        if ch == ' ' || ch == '\t' || ch == '.' || ch == ',' || ch == '!' || ch == '?'
            || ch == ';' || ch == ':' || ch == '\n'
        {
            if self.raw_buffer.is_empty() {
                return None;
            }

            // Check for macro expansion before auto-restore
            let macro_expansion = self.macros.get(&self.raw_buffer).cloned();
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
            if self.english.should_restore(&clean_raw) {
                let inner_buf = self.buffer().to_string();
                let clean_inner = strip_diacritics(&inner_buf).to_lowercase();
                let has_diacritics = clean_inner != inner_buf.to_lowercase();
                
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
            
            let flush_event = self.flush();
            let mut final_word = previous_inner.clone();
            if let Some(EngineEvent::Flush(word)) = flush_event {
                final_word = word;
            }

            let result = if final_word != previous_inner {
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

        // Regular character processing
        let previous_inner = self.buffer().to_string();
        self.raw_buffer.push(ch);

        match self.input_method {
            InputMethod::Telex => { self.telex.process_key(ch); }
            InputMethod::Vni => { self.vni.process_key(ch); }
        }

        let new_inner = self.buffer().to_string();
        let expected_screen = format!("{}{}", previous_inner, ch);

        if new_inner != expected_screen {
            Some(EngineEvent::Replace {
                backspaces: previous_inner.chars().count() + 1,
                insert: new_inner,
            })
        } else {
            None
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
            'à' | 'á' | 'ả' | 'ã' | 'ạ' | 'ă' | 'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ'
            | 'â' | 'ầ' | 'ấ' | 'ẩ' | 'ẫ' | 'ậ' => 'a',
            // A variants
            'À' | 'Á' | 'Ả' | 'Ã' | 'Ạ' | 'Ă' | 'Ằ' | 'Ắ' | 'Ẳ' | 'Ẵ' | 'Ặ'
            | 'Â' | 'Ầ' | 'Ấ' | 'Ẩ' | 'Ẫ' | 'Ậ' => 'A',
            // e variants
            'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' | 'ê' | 'ề' | 'ế' | 'ể' | 'ễ' | 'ệ' => 'e',
            'È' | 'É' | 'Ẻ' | 'Ẽ' | 'Ẹ' | 'Ê' | 'Ề' | 'Ế' | 'Ể' | 'Ễ' | 'Ệ' => 'E',
            // i variants
            'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị' => 'i',
            'Ì' | 'Í' | 'Ỉ' | 'Ĩ' | 'Ị' => 'I',
            // o variants
            'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' | 'ô' | 'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ'
            | 'ơ' | 'ờ' | 'ớ' | 'ở' | 'ỡ' | 'ợ' => 'o',
            'Ò' | 'Ó' | 'Ỏ' | 'Õ' | 'Ọ' | 'Ô' | 'Ồ' | 'Ố' | 'Ổ' | 'Ỗ' | 'Ộ'
            | 'Ơ' | 'Ờ' | 'Ớ' | 'Ở' | 'Ỡ' | 'Ợ' => 'O',
            // u variants
            'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' | 'ư' | 'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự' => 'u',
            'Ù' | 'Ú' | 'Ủ' | 'Ũ' | 'Ụ' | 'Ư' | 'Ừ' | 'Ứ' | 'Ử' | 'Ữ' | 'Ự' => 'U',
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
            Some(EngineEvent::UndoTones { backspaces, restored }) => {
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
        let events: Vec<_> = "ko ".chars()
            .filter_map(|ch| engine.process_key(ch))
            .collect();

        // Should contain the macro expansion
        let output: String = events.iter().filter_map(|e| match e {
            EngineEvent::Flush(s) => Some(s.as_str()),
            EngineEvent::Insert(s) => Some(s.as_str()),
            EngineEvent::Replace { insert, .. } => Some(insert.as_str()),
            _ => None,
        }).collect();

        assert!(output.contains("không"));
    }
}
