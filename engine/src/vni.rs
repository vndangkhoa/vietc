use crate::engine::EngineEvent;

const VOWELS: &[char] = &[
    'a', 'e', 'i', 'o', 'u', 'y',
    'ă', 'â', 'ê', 'ô', 'ơ', 'ư',
];

const VOWEL_ACCENTED: &[char] = &[
    'a', 'á', 'à', 'ả', 'ã', 'ạ',
    'ă', 'ằ', 'ắ', 'ẳ', 'ẵ', 'ặ',
    'â', 'ầ', 'ấ', 'ẩ', 'ẫ', 'ậ',
    'e', 'é', 'è', 'ẻ', 'ẽ', 'ẹ',
    'ê', 'ề', 'ế', 'ể', 'ễ', 'ệ',
    'i', 'í', 'ì', 'ỉ', 'ĩ', 'ị',
    'o', 'ó', 'ò', 'ỏ', 'õ', 'ọ',
    'ô', 'ồ', 'ố', 'ổ', 'ỗ', 'ộ',
    'ơ', 'ờ', 'ớ', 'ở', 'ỡ', 'ợ',
    'u', 'ú', 'ù', 'ủ', 'ũ', 'ụ',
    'ư', 'ừ', 'ứ', 'ử', 'ữ', 'ự',
    'y', 'ý', 'ỳ', 'ỷ', 'ỹ', 'ỵ',
];

fn is_vowel(c: char) -> bool {
    VOWEL_ACCENTED.contains(&c)
}

const MAX_FLEXIBLE_BACKTRACK: usize = 3;

/// Strip tone from a Vietnamese vowel, returning (base_modified_vowel, tone_digit_or_none)
fn strip_tone_vni(c: char) -> (char, Option<char>) {
    match c {
        'a' => ('a', None), 'á' => ('a', Some('1')), 'à' => ('a', Some('2')),
        'ả' => ('a', Some('3')), 'ã' => ('a', Some('4')), 'ạ' => ('a', Some('5')),
        'ă' => ('ă', None), 'ắ' => ('ă', Some('1')), 'ằ' => ('ă', Some('2')),
        'ẳ' => ('ă', Some('3')), 'ẵ' => ('ă', Some('4')), 'ặ' => ('ă', Some('5')),
        'â' => ('â', None), 'ấ' => ('â', Some('1')), 'ầ' => ('â', Some('2')),
        'ẩ' => ('â', Some('3')), 'ẫ' => ('â', Some('4')), 'ậ' => ('â', Some('5')),
        'e' => ('e', None), 'é' => ('e', Some('1')), 'è' => ('e', Some('2')),
        'ẻ' => ('e', Some('3')), 'ẽ' => ('e', Some('4')), 'ẹ' => ('e', Some('5')),
        'ê' => ('ê', None), 'ế' => ('ê', Some('1')), 'ề' => ('ê', Some('2')),
        'ể' => ('ê', Some('3')), 'ễ' => ('ê', Some('4')), 'ệ' => ('ê', Some('5')),
        'i' => ('i', None), 'í' => ('i', Some('1')), 'ì' => ('i', Some('2')),
        'ỉ' => ('i', Some('3')), 'ĩ' => ('i', Some('4')), 'ị' => ('i', Some('5')),
        'o' => ('o', None), 'ó' => ('o', Some('1')), 'ò' => ('o', Some('2')),
        'ỏ' => ('o', Some('3')), 'õ' => ('o', Some('4')), 'ọ' => ('o', Some('5')),
        'ô' => ('ô', None), 'ố' => ('ô', Some('1')), 'ồ' => ('ô', Some('2')),
        'ổ' => ('ô', Some('3')), 'ỗ' => ('ô', Some('4')), 'ộ' => ('ô', Some('5')),
        'ơ' => ('ơ', None), 'ớ' => ('ơ', Some('1')), 'ờ' => ('ơ', Some('2')),
        'ở' => ('ơ', Some('3')), 'ỡ' => ('ơ', Some('4')), 'ợ' => ('ơ', Some('5')),
        'u' => ('u', None), 'ú' => ('u', Some('1')), 'ù' => ('u', Some('2')),
        'ủ' => ('u', Some('3')), 'ũ' => ('u', Some('4')), 'ụ' => ('u', Some('5')),
        'ư' => ('ư', None), 'ứ' => ('ư', Some('1')), 'ừ' => ('ư', Some('2')),
        'ử' => ('ư', Some('3')), 'ữ' => ('ư', Some('4')), 'ự' => ('ư', Some('5')),
        'y' => ('y', None), 'ý' => ('y', Some('1')), 'ỳ' => ('y', Some('2')),
        'ỷ' => ('y', Some('3')), 'ỹ' => ('y', Some('4')), 'ỵ' => ('y', Some('5')),
        _ => (c, None),
    }
}

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

    // Tone overriding: vowel already has a tone → strip it and apply the new one
    let (base, _) = strip_tone_vni(vowel);
    if base != vowel {
        for &(v, t, result) in table {
            if v == base && t == digit {
                return Some(result);
            }
        }
    }

    None
}

/// Override the shape modifier on a vowel with a different one.
/// Preserves any existing tone.
/// VNI mappings: â↔ă via 6↔8, ô↔ơ via 6↔7
fn override_vni_modifier(vowel: char, digit: char) -> Option<char> {
    let (base, tone) = strip_tone_vni(vowel);
    let new_base = match (base, digit) {
        ('â', '8') => Some('ă'),
        ('ă', '6') => Some('â'),
        ('ô', '7') => Some('ơ'),
        ('ơ', '6') => Some('ô'),
        _ => None,
    }?;
    match tone {
        None => Some(new_base),
        Some(t) => apply_tone_to_vowel(new_base, t),
    }
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

fn is_u_vowel(c: char) -> bool {
    matches!(c, 'u' | 'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ')
}

fn is_o_vowel(c: char) -> bool {
    matches!(c, 'o' | 'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ')
}

fn tone_of_vowel_vni(c: char) -> Option<char> {
    match c {
        'u' | 'o' | 'a' | 'e' | 'i' | 'y' | 'ă' | 'â' | 'ê' | 'ô' | 'ơ' | 'ư' => None,
        'ù' | 'ò' | 'à' | 'è' | 'ì' | 'ỳ' | 'ằ' | 'ầ' | 'ề' | 'ồ' | 'ờ' | 'ừ' => Some('2'),
        'ú' | 'ó' | 'á' | 'é' | 'í' | 'ý' | 'ắ' | 'ấ' | 'ế' | 'ố' | 'ớ' | 'ứ' => Some('1'),
        'ủ' | 'ỏ' | 'ả' | 'ẻ' | 'ỉ' | 'ỷ' | 'ẳ' | 'ẩ' | 'ể' | 'ổ' | 'ở' | 'ử' => Some('3'),
        'ũ' | 'õ' | 'ã' | 'ẽ' | 'ĩ' | 'ỹ' | 'ẵ' | 'ẫ' | 'ễ' | 'ỗ' | 'ỡ' | 'ữ' => Some('4'),
        'ụ' | 'ọ' | 'ạ' | 'ẹ' | 'ị' | 'ỵ' | 'ặ' | 'ậ' | 'ệ' | 'ộ' | 'ợ' | 'ự' => Some('5'),
        _ => None,
    }
}

fn apply_tone_to_ơ_vni(tone: Option<char>) -> char {
    match tone {
        None      => 'ơ',
        Some('2') => 'ờ',
        Some('1') => 'ớ',
        Some('3') => 'ở',
        Some('4') => 'ỡ',
        Some('5') => 'ợ',
        _         => 'ơ',
    }
}

fn uo_to_uơ_vni(u_char: char, o_char: char) -> (char, char) {
    let o_tone = tone_of_vowel_vni(o_char);
    let u_tone = tone_of_vowel_vni(u_char);
    let tone = o_tone.or(u_tone);
    ('ư', apply_tone_to_ơ_vni(tone))
}

fn is_q_before_u(chars: &[char], i: usize) -> bool {
    i > 1 && chars[i - 2] == 'q'
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
                // Smart cluster "uo" → "ươ" (digit '7')
                if digit == '7' && is_o_vowel(last_ch) {
                    let mut chars: Vec<char> = self.buffer.chars().collect();
                    if chars.len() >= 2 && is_u_vowel(chars[chars.len() - 2]) && !is_q_before_u(&chars, chars.len() - 1) {
                        let o_char = chars.pop().unwrap();
                        let u_char = chars.pop().unwrap();
                        let (new_first, new_second) = uo_to_uơ_vni(u_char, o_char);
                        self.buffer = chars.into_iter().collect::<String>();
                        self.buffer.push(new_first);
                        self.buffer.push(new_second);
                        return None;
                    }
                }
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

                // Smart cluster forward (override): "uô" + 7 → "ươ"
                if digit == '7' {
                    let strip = strip_tone_vni(last_ch);
                    if strip.0 == 'ô' {
                        let mut chars: Vec<char> = self.buffer.chars().collect();
                        if chars.len() >= 2 && is_u_vowel(chars[chars.len() - 2]) && !is_q_before_u(&chars, chars.len() - 1) {
                            let o_char = chars.pop().unwrap();
                            let u_char = chars.pop().unwrap();
                            let (new_first, new_second) = uo_to_uơ_vni(u_char, o_char);
                            self.buffer = chars.into_iter().collect::<String>();
                            self.buffer.push(new_first);
                            self.buffer.push(new_second);
                            return None;
                        }
                    }
                }
                // Smart cluster reverse (override): "ươ" + 6 → "uô"
                if digit == '6' {
                    let strip = strip_tone_vni(last_ch);
                    if strip.0 == 'ơ' {
                        let mut chars: Vec<char> = self.buffer.chars().collect();
                        if chars.len() >= 2 && chars[chars.len() - 2] == 'ư' {
                            let ơ_char = chars.pop().unwrap();
                            chars.pop().unwrap();
                            let tone = tone_of_vowel_vni(ơ_char);
                            let ô_char = match tone {
                                None => 'ô',
                                Some(t) => apply_tone_to_vowel('ô', t).unwrap_or('ô'),
                            };
                            self.buffer = chars.into_iter().collect::<String>();
                            self.buffer.push('u');
                            self.buffer.push(ô_char);
                            return None;
                        }
                    }
                }
                // VNI digit 9: 'd' → 'đ'
                if digit == '9' && last_ch == 'd' {
                    self.buffer.pop();
                    self.buffer.push('đ');
                    return None;
                }
                // Modifier override: vowel already has a different modifier
                if let Some(modified) = override_vni_modifier(last_ch, digit) {
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
                        // Smart cluster "uo" → "ươ" (digit '7', flexible)
                        if digit == '7' && is_o_vowel(chars[i]) && i > 0 && is_u_vowel(chars[i - 1]) && !is_q_before_u(&chars, i) {
                            let (new_first, new_second) = uo_to_uơ_vni(chars[i - 1], chars[i]);
                            self.buffer = chars[..i - 1].iter().collect::<String>();
                            self.buffer.push(new_first);
                            self.buffer.push(new_second);
                            for &c in &chars[i + 1..] {
                                self.buffer.push(c);
                            }
                            return None;
                        }
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
                        // Smart cluster forward (override): "uô" + 7 → "ươ" (flexible)
                        if digit == '7' {
                            let strip = strip_tone_vni(chars[i]);
                            if strip.0 == 'ô' && i > 0 && is_u_vowel(chars[i - 1]) && !is_q_before_u(&chars, i) {
                                let (new_first, new_second) = uo_to_uơ_vni(chars[i - 1], chars[i]);
                                self.buffer = chars[..i - 1].iter().collect::<String>();
                                self.buffer.push(new_first);
                                self.buffer.push(new_second);
                                for &c in &chars[i + 1..] {
                                    self.buffer.push(c);
                                }
                                return None;
                            }
                        }
                        // Smart cluster reverse (override): "ươ" + 6 → "uô" (flexible)
                        if digit == '6' {
                            let strip = strip_tone_vni(chars[i]);
                            if strip.0 == 'ơ' && i > 0 && chars[i - 1] == 'ư' {
                                let ơ_char = chars[i];
                                let tone = tone_of_vowel_vni(ơ_char);
                                let ô_char = match tone {
                                    None => 'ô',
                                    Some(t) => apply_tone_to_vowel('ô', t).unwrap_or('ô'),
                                };
                                self.buffer = chars[..i - 1].iter().collect::<String>();
                                self.buffer.push('u');
                                self.buffer.push(ô_char);
                                for &c in &chars[i + 1..] {
                                    self.buffer.push(c);
                                }
                                return None;
                            }
                        }
                        // VNI digit 9: 'd' → 'đ' (flexible)
                        if digit == '9' && chars[i] == 'd' {
                            self.buffer = chars[..i].iter().collect::<String>();
                            self.buffer.push('đ');
                            for &c in &chars[i + 1..] {
                                self.buffer.push(c);
                            }
                            return None;
                        }
                        // Modifier override: vowel already has a different modifier
                        if let Some(modified) = override_vni_modifier(chars[i], digit) {
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

        // Digit '9' in flexible context: scan backwards for 'd' → 'đ'
        if digit == '9' {
            let chars: Vec<char> = self.buffer.chars().collect();
            for i in (0..chars.len()).rev() {
                if chars[i] == 'd' {
                    self.buffer = chars[..i].iter().collect::<String>();
                    self.buffer.push('đ');
                    for &c in &chars[i + 1..] {
                        self.buffer.push(c);
                    }
                    return None;
                }
            }
        }

        // Digit not applicable - just append
        self.buffer.push(digit);
        None
    }

    fn apply_pending(&mut self) {
        if let Some(modifier) = self.pending_modifier.take() {
            let chars: Vec<char> = self.buffer.chars().collect();
            if chars.is_empty() {
                return;
            }
            // Try last char first, then scan backwards (flexible backtrack)
            let start = chars.len().saturating_sub(MAX_FLEXIBLE_BACKTRACK);
            for i in (start..chars.len()).rev() {
                if is_vowel(chars[i]) {
                    if let Some(modified) = apply_digit_to_vowel(chars[i], modifier) {
                        self.buffer = chars[..i].iter().collect::<String>();
                        self.buffer.push(modified);
                        for &c in &chars[i + 1..] {
                            self.buffer.push(c);
                        }
                    }
                    return;
                }
            }
        }
    }
}
