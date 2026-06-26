use crate::bamboo::BambooEngine;
use crate::input_method::InputMethod;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub enum EngineEvent {
    Replace { backspaces: usize, insert: String },
    Insert(String),
    Flush(String),
    AutoRestore(String),
    UndoTones { backspaces: usize, restored: String },
    Paste(String),
}

pub struct Engine {
    bamboo: BambooEngine,
    macros: HashMap<String, String>,
    raw_buffer: String,
    paste_mode: bool,
}

impl Engine {
    pub fn new(method: InputMethod) -> Self {
        Self {
            bamboo: BambooEngine::new(method),
            macros: HashMap::new(),
            raw_buffer: String::new(),
            paste_mode: false,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.bamboo.set_enabled(enabled);
        if !enabled {
            self.reset();
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.bamboo.is_enabled()
    }

    pub fn set_method(&mut self, method: InputMethod) {
        self.bamboo.set_method(method);
        self.reset();
    }

    pub fn enter_paste_mode(&mut self) {
        self.paste_mode = true;
    }

    pub fn exit_paste_mode(&mut self) {
        self.paste_mode = false;
    }

    pub fn paste(&mut self, text: &str) -> EngineEvent {
        self.raw_buffer.clear();
        let event = EngineEvent::Paste(text.to_string());
        self.raw_buffer.push_str(text);
        event
    }

    pub fn replay_keystrokes(
        method: InputMethod,
        macros: &HashMap<String, String>,
        keystrokes: &[char],
    ) -> (String, bool) {
        let mut engine = Engine::new(method);
        for (shortcut, expansion) in macros {
            engine.add_macro(shortcut.clone(), expansion.clone());
        }

        let mut last_output = String::new();
        let mut composing = String::new();

        for &ch in keystrokes {
            if ch == '\x08' {
                let _ = engine.bamboo.pop_last();
                composing = engine.bamboo.get_output();
                last_output = composing.clone();
                continue;
            }

            if is_flush_char(ch) {
                if !composing.is_empty() {
                    last_output = composing.clone();
                }
                composing.clear();
                engine.bamboo.reset();
                continue;
            }

            if let Some(out) = engine.bamboo.process_key(ch) {
                composing = out.clone();
                last_output = out;
            } else {
                composing = engine.bamboo.get_output();
                last_output = composing.clone();
            }
        }

        let output = engine.bamboo.get_output();
        if !output.is_empty() {
            last_output = output.clone();
        }

        let did_flush = output.is_empty() && composing.is_empty();
        (if did_flush { String::new() } else { last_output }, did_flush)
    }

    pub fn update_with_pasted_text(&mut self, text: &str) {
        self.raw_buffer.clear();
        self.raw_buffer.push_str(text);
    }

    pub fn reset(&mut self) {
        self.bamboo.reset();
        self.raw_buffer.clear();
    }

    pub fn flush(&mut self) -> Option<EngineEvent> {
        if self.paste_mode && !self.raw_buffer.is_empty() {
            let has_unicode = self.raw_buffer.chars().any(|c| !c.is_ascii());
            if has_unicode {
                let word = self.raw_buffer.clone();
                self.raw_buffer.clear();
                self.paste_mode = false;
                return Some(EngineEvent::Flush(word));
            }
        }

        None
    }

    pub fn add_macro(&mut self, shortcut: String, expansion: String) {
        self.macros.insert(shortcut.clone(), expansion.clone());
        self.bamboo.add_macro(shortcut, expansion);
    }

    pub fn clear_macros(&mut self) {
        self.macros.clear();
        self.bamboo.clear_macros();
    }

    pub fn process_key(&mut self, ch: char) -> Option<EngineEvent> {
        if !self.bamboo.is_enabled() {
            return Some(EngineEvent::Insert(ch.to_string()));
        }

        if ch == '\x08' {
            self.bamboo.pop_last();
            let _ = self.raw_buffer.pop();
            return None;
        }

        if is_flush_char(ch) {
            if self.raw_buffer.is_empty() {
                return None;
            }

            let previous = self.bamboo.get_output();
            let prev_len = previous.chars().count();

            // Check for macro
            let macro_expansion = self.macros.get(&self.raw_buffer.to_lowercase()).cloned();
            if let Some(expansion) = macro_expansion {
                self.reset();
                return Some(EngineEvent::Replace {
                    backspaces: prev_len,
                    insert: expansion,
                });
            }

            self.reset();
            // The composed word is already correctly on screen — re-typing it
            // here would trigger a redundant backspace + clipboard-paste cycle
            // that races against the separately-forwarded flush char, eating
            // spaces and merging words. Just finalize and let the flush char
            // through untouched.
            return None;
        }

        let previous = self.bamboo.get_output();
        let prev_len = previous.chars().count();
        self.raw_buffer.push(ch);

        if let Some(new_output) = self.bamboo.process_key(ch) {
            // Only emit Replace when Vietnamese processing CHANGED the output
            // (tone/mark keys). For simple appends, let the raw key go through.
            let expected = format!("{}{}", previous, ch);
            if new_output != expected && new_output != previous {
                let cased = match_casing(&self.raw_buffer, &new_output);
                return Some(EngineEvent::Replace {
                    backspaces: prev_len,
                    insert: cased,
                });
            }
        }

        None
    }

    pub fn buffer(&self) -> String {
        self.bamboo.get_output()
    }
}

fn is_flush_char(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '.' | ',' | '!' | '?' | ';' | ':' | '\n')
}

fn match_casing(raw: &str, processed: &str) -> String {
    if raw.is_empty() || processed.is_empty() {
        return processed.to_string();
    }

    let alpha: Vec<char> = raw.chars().filter(|c| c.is_alphabetic()).collect();
    if alpha.is_empty() {
        return processed.to_string();
    }

    let all_upper = alpha.iter().all(|c| c.is_uppercase());
    if all_upper {
        processed.to_uppercase()
    } else if alpha[0].is_uppercase() {
        let mut chars = processed.chars();
        match chars.next() {
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            None => processed.to_string(),
        }
    } else {
        processed.to_string()
    }
}
