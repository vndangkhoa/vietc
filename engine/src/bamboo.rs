// SPDX-License-Identifier: MIT
use crate::input_method::{InputMethod, InputMethodRules, get_rules};
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct Transformation {
    base_char: char,
    mark_applied: Option<char>,
    tone_applied: Option<char>,
    is_upper: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Vietnamese,
    English,
}

impl Mode {
    fn is_vn(self) -> bool { matches!(self, Mode::Vietnamese) }
}

pub struct BambooEngine {
    composition: Vec<Transformation>,
    rules: InputMethodRules,
    mode: Mode,
    macros: HashMap<String, String>,
    macro_buf: String,
}

impl BambooEngine {
    pub fn new(method: InputMethod) -> Self {
        Self {
            composition: Vec::new(),
            rules: get_rules(method),
            mode: Mode::Vietnamese,
            macros: HashMap::new(),
            macro_buf: String::new(),
        }
    }

    pub fn set_method(&mut self, method: InputMethod) {
        self.rules = get_rules(method);
        self.reset();
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.mode = if enabled { Mode::Vietnamese } else { Mode::English };
        if !enabled { self.reset(); }
    }

    pub fn is_enabled(&self) -> bool {
        self.mode.is_vn()
    }

    pub fn add_macro(&mut self, shortcut: String, expansion: String) {
        self.macros.insert(shortcut, expansion);
    }

    pub fn clear_macros(&mut self) {
        self.macros.clear();
    }

    pub fn reset(&mut self) {
        self.composition.clear();
        self.macro_buf.clear();
    }

    pub fn process_key(&mut self, ch: char) -> Option<String> {
        if !self.mode.is_vn() {
            return Some(ch.to_string());
        }

        let lower = ch.to_ascii_lowercase();

        // Check macros
        self.macro_buf.push(lower);
        for (shortcut, expansion) in &self.macros.clone() {
            if self.macro_buf.ends_with(shortcut) {
                self.macro_buf.clear();
                self.reset();
                return Some(expansion.clone());
            }
        }
        if self.macro_buf.len() > 50 {
            self.macro_buf.clear();
        }

        // Check tone keys — only apply if composition has a vowel, else treat as normal char
        if let Some(&(tone_char, _tone_name)) = self.rules.tone_keys.get(&lower) {
            let has_vowel = self.composition.iter().any(|t| {
                is_vowel(t.mark_applied.unwrap_or(t.base_char))
            });
            if has_vowel {
                return self.apply_tone(tone_char);
            }
            // Fall through: append as normal character
        }

        // Smart "uo" → "ươ" shortcut with flexible backtrack":
        // Scan backward through consonants to find the "uo" pair
        if self.rules.method == InputMethod::Telex && lower == 'w'
            || self.rules.method == InputMethod::Vni && lower == '7'
        {
            if self.composition.len() >= 2 {
                for offset in 0..5usize.min(self.composition.len() - 1) {
                    let o_idx = self.composition.len() - 1 - offset;
                    let o_ch = self.composition[o_idx].base_char.to_ascii_lowercase();
                    if o_ch == 'o' && o_idx > 0 {
                        let u_ch = self.composition[o_idx - 1].base_char.to_ascii_lowercase();
                        if u_ch == 'u' {
                            // Found "uo" pair, replace with "ươ"
                            let u_idx = o_idx - 1;
                            let old_tone_o = self.composition[o_idx].tone_applied;
                            let was_upper = self.composition[u_idx].is_upper;
                            self.composition.drain(u_idx..=o_idx);
                            self.composition.insert(u_idx, Transformation { base_char: 'ư', mark_applied: Some('ư'), tone_applied: old_tone_o, is_upper: was_upper });
                            self.composition.insert(u_idx + 1, Transformation { base_char: 'ơ', mark_applied: Some('ơ'), tone_applied: None, is_upper: false });
                            return Some(self.flatten());
                        }
                    }
                    if o_ch == 'u' || is_vowel(o_ch) {
                        break; // Stop at vowel boundary
                    }
                }
            }

            // Smart "ua" → "ưa": the horn goes on the u (xưa, chưa, mưa, lửa),
            // not the breve on the a ("xuă" is not a valid syllable). Skip the
            // "qu" glide case, where the u belongs to the initial consonant and
            // the a takes the breve instead (quă → quăng).
            if self.composition.len() >= 2 {
                let a_idx = self.composition.len() - 1;
                let u_idx = a_idx - 1;
                let a_ch = self.composition[a_idx].base_char.to_ascii_lowercase();
                let u_ch = self.composition[u_idx].base_char.to_ascii_lowercase();
                let preceded_by_q = u_idx > 0
                    && self.composition[u_idx - 1]
                        .base_char
                        .eq_ignore_ascii_case(&'q');
                if a_ch == 'a'
                    && u_ch == 'u'
                    && self.composition[u_idx].mark_applied.is_none()
                    && !preceded_by_q
                {
                    self.composition[u_idx].base_char = 'ư';
                    self.composition[u_idx].mark_applied = Some('ư');
                    return Some(self.flatten());
                }
            }
        }

        // Try mark rules with flexible backtrack" (scan up to 3 chars backward)
        let mark_match = self.find_mark_backtrack(lower);

        if let Some((idx, pattern, result)) = mark_match {
            self.apply_mark_at(idx, &pattern, &result);
            return Some(self.flatten());
        }

        // Normal character — append
        self.append_char(ch);
        self.macro_buf.clear();
        Some(self.flatten())
    }

    fn find_mark_backtrack(&self, lower: char) -> Option<(usize, String, String)> {
        let scan_limit = 5usize.min(self.composition.len());
        for offset in 0..scan_limit {
            let idx = self.composition.len() - 1 - offset;
            let ch = self.composition[idx].base_char.to_ascii_lowercase();
            let seq = format!("{}{}", ch, lower);
            if let Some((p, r)) = self.rules.mark_rules.iter().find(|(p, _)| seq == *p) {
                return Some((idx, p.clone(), r.clone()));
            }
        }
        None
    }

    fn apply_mark_at(&mut self, idx: usize, _pattern: &str, result: &str) {
        let result_chars: Vec<char> = result.chars().collect();
        let was_upper = self.composition[idx].is_upper;
        let old_tone = self.composition[idx].tone_applied;

        // Replace the char at idx with result chars
        self.composition.remove(idx);
        for (i, &ch) in result_chars.iter().enumerate() {
            self.composition.insert(idx + i, Transformation {
                base_char: ch,
                mark_applied: Some(ch),
                tone_applied: old_tone,
                is_upper: was_upper && i == 0,
            });
        }
    }

    #[allow(dead_code)]
    pub fn debug_composition(&self) -> Vec<(char, Option<char>, Option<char>)> {
        self.composition.iter().map(|t| (t.base_char, t.mark_applied, t.tone_applied)).collect()
    }

    pub fn get_output(&self) -> String {
        self.flatten()
    }

    pub fn pop_last(&mut self) -> Option<String> {
        if self.composition.pop().is_some() {
            Some(self.flatten())
        } else {
            None
        }
    }

    fn append_char(&mut self, ch: char) {
        self.composition.push(Transformation {
            base_char: ch,
            mark_applied: None,
            tone_applied: None,
            is_upper: ch.is_uppercase(),
        });
    }

    fn apply_tone(&mut self, tone_char: char) -> Option<String> {
        if self.composition.is_empty() {
            return Some(tone_char.to_string());
        }

        // Find the last syllable
        let last_syllable = self.last_syllable_range();
        let tone_pos = self.find_tone_position(last_syllable);

        if let Some(t) = self.composition.get_mut(tone_pos) {
            t.tone_applied = Some(tone_char);
            return Some(self.flatten());
        }

        Some(self.flatten())
    }

    fn last_syllable_range(&self) -> std::ops::Range<usize> {
        let mut start = 0usize;
        for (i, t) in self.composition.iter().enumerate().rev() {
            let ch = t.mark_applied.unwrap_or(t.base_char);
            if ch.is_whitespace() || ch == '.' || ch == ',' || ch == '!' || ch == '?' || ch == ';' || ch == ':' {
                start = i + 1;
                break;
            }
        }
        start..self.composition.len()
    }

    fn find_tone_position(&self, range: std::ops::Range<usize>) -> usize {
        let start = range.start;
        let mut vowels: Vec<usize> = Vec::new();

        for i in range {
            let ch = self.composition[i].mark_applied.unwrap_or(self.composition[i].base_char);
            if is_vowel(ch) {
                vowels.push(i);
            }
        }

        if vowels.is_empty() {
            return self.composition.len().saturating_sub(1);
        }

        // Exclude onset glides: in "qu…" the u and in "gi…" the i belong to the
        // initial consonant, not the vowel nucleus — so they must never carry the
        // tone (e.g. "quả" not "qủa", "giờ" not "gìơ"). Only strip the glide when
        // another vowel follows it; bare "gì"/"qu" keep the letter as the nucleus.
        if vowels.len() >= 2 && vowels[0] == start + 1 {
            let onset = self.composition[start].base_char.to_ascii_lowercase();
            let glide = self.composition[start + 1].base_char.to_ascii_lowercase();
            if (onset == 'q' && glide == 'u') || (onset == 'g' && glide == 'i') {
                vowels.remove(0);
            }
        }

        if vowels.len() == 1 {
            return vowels[0];
        }

        // Check the last two vowels with their actual characters (including marks applied)
        let cv1 = self.composition[vowels[vowels.len()-2]].mark_applied
            .unwrap_or(self.composition[vowels[vowels.len()-2]].base_char)
            .to_ascii_lowercase();
        let cv2 = self.composition[vowels[vowels.len()-1]].mark_applied
            .unwrap_or(self.composition[vowels[vowels.len()-1]].base_char)
            .to_ascii_lowercase();

        // Clusters where tone goes on the SECOND vowel:
        // oa/oe: hoá, khoẻ
        // uy: tuý
        // iê/yê: tiếng, biết, nguyễn
        // uô: muốn, buồn
        // ươ: tướng, đường
        let tone_on_second = matches!((cv1, cv2),
            ('o', 'a') | ('o', 'e') | ('u', 'y') |
            ('i', 'ê') | ('y', 'ê') | ('u', 'ô') | ('ư', 'ơ') |
            ('i', 'o') | ('u', 'â') | ('u', 'ê') | ('u', 'ơ')
        );

        if tone_on_second {
            return vowels[vowels.len()-1];
        }

        // Three+ vowels: tone on the middle one
        if vowels.len() >= 3 {
            return vowels[1];
        }

        // Default: tone on first vowel
        vowels[0]
    }

    fn flatten(&self) -> String {
        let mut output = String::new();

        for t in &self.composition {
            let base = t.mark_applied.unwrap_or(t.base_char);
            let mut ch = if let Some(tone) = t.tone_applied {
                apply_tone_to_char(base, tone)
            } else {
                base
            };

            if t.is_upper && !ch.is_uppercase() {
                ch = ch.to_ascii_uppercase();
            }

            output.push(ch);
        }

        output
    }
}

fn is_vowel(ch: char) -> bool {
    matches!(ch.to_ascii_lowercase(),
        'a' | 'e' | 'i' | 'o' | 'u' | 'y' |
        'ă' | 'â' | 'ê' | 'ô' | 'ơ' | 'ư'
    )
}

fn apply_tone_to_char(ch: char, tone: char) -> char {
    match (ch.to_ascii_lowercase(), tone) {
        // sắc
        ('a', 's') | ('a', '1') => 'á',
        ('ă', 's') | ('ă', '1') => 'ắ',
        ('â', 's') | ('â', '1') => 'ấ',
        ('e', 's') | ('e', '1') => 'é',
        ('ê', 's') | ('ê', '1') => 'ế',
        ('i', 's') | ('i', '1') => 'í',
        ('o', 's') | ('o', '1') => 'ó',
        ('ô', 's') | ('ô', '1') => 'ố',
        ('ơ', 's') | ('ơ', '1') => 'ớ',
        ('u', 's') | ('u', '1') => 'ú',
        ('ư', 's') | ('ư', '1') => 'ứ',
        ('y', 's') | ('y', '1') => 'ý',

        // huyền
        ('a', 'f') | ('a', '2') => 'à',
        ('ă', 'f') | ('ă', '2') => 'ằ',
        ('â', 'f') | ('â', '2') => 'ầ',
        ('e', 'f') | ('e', '2') => 'è',
        ('ê', 'f') | ('ê', '2') => 'ề',
        ('i', 'f') | ('i', '2') => 'ì',
        ('o', 'f') | ('o', '2') => 'ò',
        ('ô', 'f') | ('ô', '2') => 'ồ',
        ('ơ', 'f') | ('ơ', '2') => 'ờ',
        ('u', 'f') | ('u', '2') => 'ù',
        ('ư', 'f') | ('ư', '2') => 'ừ',
        ('y', 'f') | ('y', '2') => 'ỳ',

        // hỏi
        ('a', 'r') | ('a', '3') => 'ả',
        ('ă', 'r') | ('ă', '3') => 'ẳ',
        ('â', 'r') | ('â', '3') => 'ẩ',
        ('e', 'r') | ('e', '3') => 'ẻ',
        ('ê', 'r') | ('ê', '3') => 'ể',
        ('i', 'r') | ('i', '3') => 'ỉ',
        ('o', 'r') | ('o', '3') => 'ỏ',
        ('ô', 'r') | ('ô', '3') => 'ổ',
        ('ơ', 'r') | ('ơ', '3') => 'ở',
        ('u', 'r') | ('u', '3') => 'ủ',
        ('ư', 'r') | ('ư', '3') => 'ử',
        ('y', 'r') | ('y', '3') => 'ỷ',

        // ngã
        ('a', 'x') | ('a', '4') => 'ã',
        ('ă', 'x') | ('ă', '4') => 'ẵ',
        ('â', 'x') | ('â', '4') => 'ẫ',
        ('e', 'x') | ('e', '4') => 'ẽ',
        ('ê', 'x') | ('ê', '4') => 'ễ',
        ('i', 'x') | ('i', '4') => 'ĩ',
        ('o', 'x') | ('o', '4') => 'õ',
        ('ô', 'x') | ('ô', '4') => 'ỗ',
        ('ơ', 'x') | ('ơ', '4') => 'ỡ',
        ('u', 'x') | ('u', '4') => 'ũ',
        ('ư', 'x') | ('ư', '4') => 'ữ',
        ('y', 'x') | ('y', '4') => 'ỹ',

        // nặng
        ('a', 'j') | ('a', '5') => 'ạ',
        ('ă', 'j') | ('ă', '5') => 'ặ',
        ('â', 'j') | ('â', '5') => 'ậ',
        ('e', 'j') | ('e', '5') => 'ẹ',
        ('ê', 'j') | ('ê', '5') => 'ệ',
        ('i', 'j') | ('i', '5') => 'ị',
        ('o', 'j') | ('o', '5') => 'ọ',
        ('ô', 'j') | ('ô', '5') => 'ộ',
        ('ơ', 'j') | ('ơ', '5') => 'ợ',
        ('u', 'j') | ('u', '5') => 'ụ',
        ('ư', 'j') | ('ư', '5') => 'ự',
        ('y', 'j') | ('y', '5') => 'ỵ',

        // unknown — return unchanged
        _ => ch,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn process(method: InputMethod, input: &str) -> String {
        let mut engine = BambooEngine::new(method);
        let mut output = String::new();
        for ch in input.chars() {
            if let Some(o) = engine.process_key(ch) {
                output = o;
            }
        }
        output
    }

    #[test]
    fn test_telex_tone() {
        assert_eq!(process(InputMethod::Telex, "tieengs"), "tiếng");
        assert_eq!(process(InputMethod::Telex, "dduwowngf"), "đường");
        assert_eq!(process(InputMethod::Telex, "thuw"), "thư");
    }

    #[test]
    fn test_telex_marks() {
        assert_eq!(process(InputMethod::Telex, "aa"), "â");
        assert_eq!(process(InputMethod::Telex, "ee"), "ê");
        assert_eq!(process(InputMethod::Telex, "oo"), "ô");
        assert_eq!(process(InputMethod::Telex, "aw"), "ă");
        assert_eq!(process(InputMethod::Telex, "ow"), "ơ");
        assert_eq!(process(InputMethod::Telex, "uw"), "ư");
        assert_eq!(process(InputMethod::Telex, "dd"), "đ");
    }

    #[test]
    fn test_vni_tone() {
        assert_eq!(process(InputMethod::Vni, "d9"), "đ");
        assert_eq!(process(InputMethod::Vni, "u7"), "ư");
        assert_eq!(process(InputMethod::Vni, "o7"), "ơ");
        assert_eq!(process(InputMethod::Vni, "d9u7o7ng2"), "đường");
        assert_eq!(process(InputMethod::Vni, "tie6ng1"), "tiếng");
        assert_eq!(process(InputMethod::Vni, "thu3"), "thủ");
        assert_eq!(process(InputMethod::Vni, "xa4"), "xã");
        assert_eq!(process(InputMethod::Vni, "na85ng5"), "nặng");
    }

    #[test]
    fn test_vni_marks() {
        assert_eq!(process(InputMethod::Vni, "a6"), "â");
        assert_eq!(process(InputMethod::Vni, "e6"), "ê");
        assert_eq!(process(InputMethod::Vni, "o6"), "ô");
        assert_eq!(process(InputMethod::Vni, "o7"), "ơ");
        assert_eq!(process(InputMethod::Vni, "u7"), "ư");
        assert_eq!(process(InputMethod::Vni, "a8"), "ă");
        assert_eq!(process(InputMethod::Vni, "d9"), "đ");
    }

    #[test]
    fn test_tone_placement() {
        // oa cluster: tone on second vowel → hoá (standard Vietnamese IME convention)
        assert_eq!(process(InputMethod::Telex, "hoas"), "hoá");
        // thuố = th + uô + sắc → tone on ô (uô cluster → tone on second)
        assert_eq!(process(InputMethod::Telex, "thuoos"), "thuố");
    }

    #[test]
    fn test_reset() {
        let mut engine = BambooEngine::new(InputMethod::Telex);
        engine.process_key('t');
        engine.reset();
        assert!(engine.get_output().is_empty());
    }

    #[test]
    fn test_uppercase_preservation() {
        let mut engine = BambooEngine::new(InputMethod::Telex);
        engine.process_key('T');
        engine.process_key('i');
        engine.process_key('e');
        engine.process_key('e');
        engine.process_key('n');
        engine.process_key('g');
        engine.process_key('s');
        assert_eq!(engine.get_output(), "Tiếng");
    }

    #[test]
    fn test_simple_words() {
        assert_eq!(process(InputMethod::Telex, "chafo"), "chào");
        assert_eq!(process(InputMethod::Vni, "chao2"), "chào");
    }

#[test]
fn test_telex_tuaan() {
    let mut e = crate::bamboo::BambooEngine::new(crate::input_method::InputMethod::Telex);
    let mut out = String::new();
    for ch in "Tuaans".chars() {
        if let Some(o) = e.process_key(ch) { out = o; }
    }
    assert_eq!(out, "Tuấn", "Expected Tuấn, got {}", out);
}

#[test]
fn test_telex_nguyeenx() {
    let mut e = crate::bamboo::BambooEngine::new(crate::input_method::InputMethod::Telex);
    let mut out = String::new();
    for ch in "nguyeenx".chars() {
        if let Some(o) = e.process_key(ch) { out = o; }
    }
    assert_eq!(out, "nguyễn", "Expected nguyễn, got {}", out);
}

#[test]
fn test_telex_gios() {
    let mut e = crate::bamboo::BambooEngine::new(crate::input_method::InputMethod::Telex);
    let mut out = String::new();
    for ch in "gios".chars() {
        if let Some(o) = e.process_key(ch) { out = o; }
    }
    assert_eq!(out, "gió", "Expected gió, got {}", out);
}



    #[test]
    fn test_telex_ua_horn() {
        // "w" after a "ua" cluster puts the horn on the u (ưa), it must not
        // put the breve on the a ("xuă" is not a valid Vietnamese syllable).
        assert_eq!(process(InputMethod::Telex, "xuaw"), "xưa");
        assert_eq!(process(InputMethod::Telex, "chuaw"), "chưa");
        assert_eq!(process(InputMethod::Telex, "muaw"), "mưa");
        assert_eq!(process(InputMethod::Telex, "Xuaw"), "Xưa");
        // With a following tone the horn target still carries the tone.
        assert_eq!(process(InputMethod::Telex, "luawr"), "lửa");
        // "qu" glide exception: the u belongs to the initial, a takes the breve.
        assert_eq!(process(InputMethod::Telex, "quawng"), "quăng");
        // VNI parity.
        assert_eq!(process(InputMethod::Vni, "xua7"), "xưa");
        assert_eq!(process(InputMethod::Vni, "qua8ng"), "quăng");
    }

    #[test]
    fn test_telex_r_as_normal_char() {
        let mut e = BambooEngine::new(InputMethod::Telex);
        let mut out = String::new();
        for ch in "tr".chars() {
            if let Some(o) = e.process_key(ch) { out = o; }
        }
        assert_eq!(out, "tr");
        out.clear(); e.reset();
        for ch in "traf".chars() {
            if let Some(o) = e.process_key(ch) { out = o; }
        }
        assert_eq!(out, "trà");
        out.clear(); e.reset();
        for ch in "tar".chars() {
            if let Some(o) = e.process_key(ch) { out = o; }
        }
        assert_eq!(out, "tả");
        out.clear(); e.reset();
        for ch in "tramr".chars() {
            if let Some(o) = e.process_key(ch) { out = o; }
        }
        assert_eq!(out, "trảm");
    }


}
