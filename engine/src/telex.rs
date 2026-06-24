use crate::engine::EngineEvent;

const VOWELS: &[char] = &['a', 'e', 'i', 'o', 'u', 'y', 'ă', 'â', 'ê', 'ô', 'ơ', 'ư'];

fn is_vowel(c: char) -> bool {
    VOWELS.contains(&c)
}

fn apply_tone_to_vowel(vowel: char, tone: char) -> Option<char> {
    // Standard Telex: f=huyền, s=sắc, r=hỏi, x=ngã, j=nặng
    let table: &[(char, char, char)] = &[
        ('a', 'f', 'à'), ('a', 's', 'á'), ('a', 'r', 'ả'), ('a', 'x', 'ã'), ('a', 'j', 'ạ'),
        ('ă', 'f', 'ằ'), ('ă', 's', 'ắ'), ('ă', 'r', 'ẳ'), ('ă', 'x', 'ẵ'), ('ă', 'j', 'ặ'),
        ('â', 'f', 'ầ'), ('â', 's', 'ấ'), ('â', 'r', 'ẩ'), ('â', 'x', 'ẫ'), ('â', 'j', 'ậ'),
        ('e', 'f', 'è'), ('e', 's', 'é'), ('e', 'r', 'ẻ'), ('e', 'x', 'ẽ'), ('e', 'j', 'ẹ'),
        ('ê', 'f', 'ề'), ('ê', 's', 'ế'), ('ê', 'r', 'ể'), ('ê', 'x', 'ễ'), ('ê', 'j', 'ệ'),
        ('i', 'f', 'ì'), ('i', 's', 'í'), ('i', 'r', 'ỉ'), ('i', 'x', 'ĩ'), ('i', 'j', 'ị'),
        ('o', 'f', 'ò'), ('o', 's', 'ó'), ('o', 'r', 'ỏ'), ('o', 'x', 'õ'), ('o', 'j', 'ọ'),
        ('ô', 'f', 'ồ'), ('ô', 's', 'ố'), ('ô', 'r', 'ổ'), ('ô', 'x', 'ỗ'), ('ô', 'j', 'ộ'),
        ('ơ', 'f', 'ờ'), ('ơ', 's', 'ớ'), ('ơ', 'r', 'ở'), ('ơ', 'x', 'ỡ'), ('ơ', 'j', 'ợ'),
        ('u', 'f', 'ù'), ('u', 's', 'ú'), ('u', 'r', 'ủ'), ('u', 'x', 'ũ'), ('u', 'j', 'ụ'),
        ('ư', 'f', 'ừ'), ('ư', 's', 'ứ'), ('ư', 'r', 'ử'), ('ư', 'x', 'ữ'), ('ư', 'j', 'ự'),
        ('y', 'f', 'ỳ'), ('y', 's', 'ý'), ('y', 'r', 'ỷ'), ('y', 'x', 'ỹ'), ('y', 'j', 'ỵ'),
    ];

    for &(v, t, result) in table {
        if v == vowel && t == tone {
            return Some(result);
        }
    }
    None
}

fn apply_w_to_vowel(vowel: char) -> Option<char> {
    // Telex: aw=â, ow=ô, ew=ê, uw=ư
    // (aa=ă, ee=ê, oo=ô are handled by double-letter logic)
    match vowel {
        'a' => Some('â'),
        'o' => Some('ô'),
        'e' => Some('ê'),
        'u' => Some('ư'),
        _ => None,
    }
}


pub struct TelexEngine {
    buffer: String,
    pending_modifier: Option<char>,
}

impl TelexEngine {
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

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn flush(&mut self) -> Option<EngineEvent> {
        if self.buffer.is_empty() && self.pending_modifier.is_none() {
            return None;
        }

        self.apply_pending_to_last_vowel();

        let result = self.buffer.clone();
        self.buffer.clear();
        self.pending_modifier = None;

        Some(EngineEvent::Flush(result))
    }

    /// Flush buffer and append a trailing character (e.g., space, punctuation)
    pub fn flush_with(&mut self, trailing: char) -> Option<EngineEvent> {
        if self.buffer.is_empty() && self.pending_modifier.is_none() {
            return Some(EngineEvent::Insert(trailing.to_string()));
        }

        self.apply_pending_to_last_vowel();

        let mut result = self.buffer.clone();
        result.push(trailing);
        self.buffer.clear();
        self.pending_modifier = None;

        Some(EngineEvent::Flush(result))
    }

    fn apply_pending_to_last_vowel(&mut self) {
        if let Some(modifier) = self.pending_modifier.take() {
            if let Some(last_ch) = self.buffer.pop() {
                if is_vowel(last_ch) {
                    if let Some(modified) = match modifier {
                        'f' | 's' | 'r' | 'x' | 'j' => apply_tone_to_vowel(last_ch, modifier),
                        'w' => apply_w_to_vowel(last_ch),
                        _ => None,
                    } {
                        self.buffer.push(modified);
                    } else {
                        self.buffer.push(last_ch);
                        self.pending_modifier = Some(modifier);
                    }
                } else {
                    self.buffer.push(last_ch);
                    self.pending_modifier = Some(modifier);
                }
            }
        }
    }

    pub fn process_key(&mut self, ch: char) -> Option<EngineEvent> {
        match ch {
            ' ' | '\t' => self.flush_with(ch),
            '.' | ',' | '!' | '?' | ';' | ':' | '\n' => self.flush_with(ch),
            'f' | 's' | 'r' | 'x' | 'j' => self.process_tone(ch),
            'a' | 'e' | 'o' => self.process_vowel_or_double(ch),
            'w' => self.process_w(),
            _ => self.process_other(ch),
        }
    }

    fn process_tone(&mut self, tone: char) -> Option<EngineEvent> {
        self.apply_pending_to_last_vowel();

        // Find the vowel to apply tone to.
        // For compound vowels, tone goes on the first vowel of the cluster
        // (except when preceded by o/u in certain combinations).
        // Simplified: apply to the first vowel found scanning backward.
        if !self.buffer.is_empty() {
            let chars: Vec<char> = self.buffer.chars().collect();
            // Scan backward to find the last vowel
            for i in (0..chars.len()).rev() {
                if is_vowel(chars[i]) {
                    // Check if there's a vowel before this one (compound vowel)
                    // For compound vowels starting with o/u, tone goes on the second vowel
                    if i > 0 && is_vowel(chars[i - 1]) {
                        let first = chars[i - 1];
                        let second = chars[i];
                        // For oa, oe, uy → tone on second vowel (already at position i)
                        // For others → tone on first vowel
                        let tone_on_second = matches!(
                            (first, second),
                            ('o', 'a') | ('o', 'e') | ('u', 'y')
                        );
                        if !tone_on_second {
                            // Apply tone to first vowel
                            if let Some(modified) = apply_tone_to_vowel(chars[i - 1], tone) {
                                self.buffer = chars[..i - 1].iter().collect::<String>();
                                self.buffer.push(modified);
                                // Re-add chars after i-1
                                for &c in &chars[i..] {
                                    self.buffer.push(c);
                                }
                                return None;
                            }
                        }
                    }

                    // Apply tone to this vowel (default: last vowel)
                    if let Some(modified) = apply_tone_to_vowel(chars[i], tone) {
                        self.buffer = chars[..i].iter().collect::<String>();
                        self.buffer.push(modified);
                        for &c in &chars[i + 1..] {
                            self.buffer.push(c);
                        }
                        return None;
                    }
                    break;
                }
            }
        }

        // No vowel found - append tone key (might be English)
        self.buffer.push(tone);
        None
    }

    fn process_vowel_or_double(&mut self, ch: char) -> Option<EngineEvent> {
        self.apply_pending_to_last_vowel();

        // Check for double-letter pattern
        if let Some(last_ch) = self.buffer.chars().last() {
            if last_ch == ch {
                let replacement = match ch {
                    'a' => Some('ă'),
                    'e' => Some('ê'),
                    'o' => Some('ô'),
                    _ => None,
                };

                if let Some(rep) = replacement {
                    self.buffer.pop();
                    self.buffer.push(rep);
                    return None;
                }
            }
        }

        self.buffer.push(ch);
        None
    }

    fn process_w(&mut self) -> Option<EngineEvent> {
        self.apply_pending_to_last_vowel();

        if let Some(last_ch) = self.buffer.chars().last() {
            if is_vowel(last_ch) {
                if let Some(modified) = apply_w_to_vowel(last_ch) {
                    self.buffer.pop();
                    self.buffer.push(modified);
                    return None;
                }
            }
        }

        // w after consonant or at start - pending modifier
        self.pending_modifier = Some('w');
        None
    }

    fn process_other(&mut self, ch: char) -> Option<EngineEvent> {
        // dd → đ digraph
        if ch == 'd' {
            if let Some(last_ch) = self.buffer.chars().last() {
                if last_ch == 'd' {
                    let chars: Vec<char> = self.buffer.chars().collect();
                    if chars.len() == 1 {
                        self.buffer.pop();
                        self.buffer.push('đ');
                        return None;
                    } else if chars.len() >= 2 {
                        let prev = chars[chars.len() - 2];
                        if !is_vowel(prev) {
                            self.buffer.pop();
                            self.buffer.push('đ');
                            return None;
                        }
                    }
                }
            }
        }

        if self.pending_modifier.is_some() {
            self.apply_pending_to_last_vowel();
        }

        self.buffer.push(ch);
        None
    }
}
