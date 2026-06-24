use crate::engine::EngineEvent;

const VOWELS: &[char] = &['a', 'e', 'i', 'o', 'u', 'y', 'ă', 'â', 'ê', 'ô', 'ơ', 'ư'];

fn is_vowel(c: char) -> bool {
    VOWELS.contains(&c)
}

const MAX_FLEXIBLE_BACKTRACK: usize = 3;

fn apply_tone_to_vowel(vowel: char, digit: char) -> Option<char> {
    // VNI: 1=sắc, 2=huyền, 3=hỏi, 4=ngã, 5=nặng
    let table: &[(char, char, char)] = &[
        ('a', '1', 'á'), ('a', '2', 'à'), ('a', '3', 'ả'), ('a', '4', 'ã'), ('a', '5', 'ạ'),
        ('ă', '1', 'ắ'), ('ă', '2', 'ằ'), ('ă', '3', 'ẳ'), ('ă', '4', 'ẵ'), ('ă', '5', 'ặ'),
        ('â', '1', 'ấ'), ('â', '2', 'ầ'), ('â', '3', 'ẩ'), ('â', '4', 'ẫ'), ('â', '5', 'ậ'),
        ('e', '1', 'é'), ('e', '2', 'è'), ('e', '3', 'ẻ'), ('e', '4', 'ẽ'), ('e', '5', 'ẹ'),
        ('ê', '1', 'ế'), ('ê', '2', 'ề'), ('ê', '3', 'ể'), ('ê', '4', 'ễ'), ('ê', '5', 'ệ'),
        ('i', '1', 'í'), ('i', '2', 'ì'), ('i', '3', 'ỉ'), ('i', '4', 'ĩ'), ('i', '5', 'ị'),
        ('o', '1', 'ó'), ('o', '2', 'ò'), ('o', '3', 'ỏ'), ('o', '4', 'õ'), ('o', '5', 'ọ'),
        ('ô', '1', 'ố'), ('ô', '2', 'ồ'), ('ô', '3', 'ổ'), ('ô', '4', 'ỗ'), ('ô', '5', 'ộ'),
        ('ơ', '1', 'ớ'), ('ơ', '2', 'ờ'), ('ơ', '3', 'ở'), ('ơ', '4', 'ỡ'), ('ơ', '5', 'ợ'),
        ('u', '1', 'ú'), ('u', '2', 'ù'), ('u', '3', 'ủ'), ('u', '4', 'ũ'), ('u', '5', 'ụ'),
        ('ư', '1', 'ứ'), ('ư', '2', 'ừ'), ('ư', '3', 'ử'), ('ư', '4', 'ữ'), ('ư', '5', 'ự'),
        ('y', '1', 'ý'), ('y', '2', 'ỳ'), ('y', '3', 'ỷ'), ('y', '4', 'ỹ'), ('y', '5', 'ỵ'),
    ];

    for &(v, t, result) in table {
        if v == vowel && t == digit {
            return Some(result);
        }
    }
    None
}

fn apply_digit_to_vowel(vowel: char, digit: char) -> Option<char> {
    // VNI: 6=â, 7=ơ+ư, 8=ă+ê, 9=ô, 0=ơ+ư
    // Standard VNI: a6=â, a8=ă, e6=ê, o6=ô, o7=ơ, u7=ư
    match digit {
        '6' => match vowel {
            'a' => Some('â'),
            'e' => Some('ê'),
            'o' => Some('ô'),
            _ => None,
        },
        '7' => match vowel {
            'o' => Some('ơ'),
            'u' => Some('ư'),
            _ => None,
        },
        '8' => match vowel {
            'a' => Some('ă'),
            _ => None,
        },
        _ => None,
    }
}

pub struct VniEngine {
    buffer: String,
    pending_modifier: Option<char>,
}

impl VniEngine {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            pending_modifier: None,
        }
    }

    pub fn reset(&mut self) {
        self.buffer.clear();
        self.pending_modifier = None;
    }

    pub fn pop(&mut self) {
        self.buffer.pop();
        self.pending_modifier = None;
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn flush(&mut self) -> Option<EngineEvent> {
        if self.buffer.is_empty() {
            return None;
        }

        let result = self.buffer.clone();
        self.buffer.clear();
        self.pending_modifier = None;

        Some(EngineEvent::Flush(result))
    }

    pub fn process_key(&mut self, ch: char) -> Option<EngineEvent> {
        match ch {
            '0'..='9' => self.process_digit(ch),
            _ => {
                // Non-digit: apply pending modifier if any
                if self.pending_modifier.is_some() {
                    self.apply_pending();
                }
                self.buffer.push(ch);
                None
            }
        }
    }

    fn process_digit(&mut self, digit: char) -> Option<EngineEvent> {
        // Apply any pending modifier first
        if self.pending_modifier.is_some() {
            self.apply_pending();
        }

        // Find last vowel (standard behavior)
        if let Some(last_ch) = self.buffer.chars().last() {
            if is_vowel(last_ch) {
                // Try tone first (1-5)
                if let Some(modified) = apply_tone_to_vowel(last_ch, digit) {
                    self.buffer.pop();
                    self.buffer.push(modified);
                    return None;
                }

                // Try vowel modification (6-9, 0)
                if let Some(modified) = apply_digit_to_vowel(last_ch, digit) {
                    self.buffer.pop();
                    self.buffer.push(modified);
                    return None;
                }
            }
        }

        // Flexible placement: last char not a vowel, scan the last N chars
        if let Some(last_ch) = self.buffer.chars().last() {
            if !is_vowel(last_ch) {
                let chars: Vec<char> = self.buffer.chars().collect();
                let start = chars.len().saturating_sub(MAX_FLEXIBLE_BACKTRACK);
                for i in (start..chars.len()).rev() {
                    if is_vowel(chars[i]) {
                        // Try tone first (1-5)
                        if let Some(modified) = apply_tone_to_vowel(chars[i], digit) {
                            self.buffer = chars[..i].iter().collect::<String>();
                            self.buffer.push(modified);
                            for &c in &chars[i + 1..] {
                                self.buffer.push(c);
                            }
                            return None;
                        }
                        // Try vowel modification (6-9, 0)
                        if let Some(modified) = apply_digit_to_vowel(chars[i], digit) {
                            self.buffer = chars[..i].iter().collect::<String>();
                            self.buffer.push(modified);
                            for &c in &chars[i + 1..] {
                                self.buffer.push(c);
                            }
                            return None;
                        }
                    }
                }
            }
        }

        // Digit not applicable - just append
        self.buffer.push(digit);
        None
    }

    fn apply_pending(&mut self) {
        if let Some(modifier) = self.pending_modifier.take() {
            if let Some(last_ch) = self.buffer.chars().last() {
                if is_vowel(last_ch) {
                    if let Some(modified) = apply_digit_to_vowel(last_ch, modifier) {
                        self.buffer.pop();
                        self.buffer.push(modified);
                    }
                }
            }
        }
    }
}
