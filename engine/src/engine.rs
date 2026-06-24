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
            return Some(EngineEvent::Insert(ch.to_string()));
        }

        // ESC = undo tones
        if ch == '\x1b' {
            return self.process_escape();
        }

        if ch == ' ' || ch == '\t' || ch == '.' || ch == ',' || ch == '!' || ch == '?'
            || ch == ';' || ch == ':' || ch == '\n'
        {
            // Check for macro expansion before auto-restore
            let buffer = match self.input_method {
                InputMethod::Telex => self.telex.buffer(),
                InputMethod::Vni => self.vni.buffer(),
            };

            let macro_expansion = self.macros.get(buffer).cloned();

            if let Some(expansion) = macro_expansion {
                self.reset();
                let mut result = expansion;
                result.push(ch);
                return Some(EngineEvent::Flush(result));
            }

            // Try auto-restore before flushing
            if let Some(restore) = self.try_auto_restore() {
                match restore {
                    EngineEvent::AutoRestore(word) => {
                        let mut result = String::new();
                        for _ in 0..word.len() {
                            result.push('\x08');
                        }
                        result.push_str(&word);
                        result.push(ch);
                        return Some(EngineEvent::Flush(result));
                    }
                    _ => return Some(restore),
                }
            }

            // Flush buffer with trailing character
            return match self.input_method {
                InputMethod::Telex => self.telex.flush_with(ch),
                InputMethod::Vni => self.vni_flush_with(ch),
            };
        }

        match self.input_method {
            InputMethod::Telex => self.telex.process_key(ch),
            InputMethod::Vni => self.vni.process_key(ch),
        }
    }

    fn vni_flush_with(&mut self, ch: char) -> Option<EngineEvent> {
        if self.vni.buffer().is_empty() {
            return Some(EngineEvent::Insert(ch.to_string()));
        }
        let flush = self.vni.flush();
        match flush {
            Some(EngineEvent::Flush(mut text)) => {
                text.push(ch);
                Some(EngineEvent::Flush(text))
            }
            _ => Some(EngineEvent::Insert(ch.to_string())),
        }
    }

    fn try_auto_restore(&mut self) -> Option<EngineEvent> {
        let buffer = match self.input_method {
            InputMethod::Telex => self.telex.buffer(),
            InputMethod::Vni => self.vni.buffer(),
        };

        if buffer.is_empty() {
            return None;
        }

        if !buffer.chars().all(|c| c.is_ascii_alphabetic()) {
            return None;
        }

        let clean = buffer.to_lowercase();
        if self.english.should_restore(&clean) {
            let original = buffer.to_string();
            self.reset();
            return Some(EngineEvent::AutoRestore(original));
        }

        None
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
            _ => None,
        }).collect();

        assert!(output.contains("không"));
    }
}
