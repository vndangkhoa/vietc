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

/// Maximum number of characters to scan backward during flexible placement.
/// Vietnamese vowel clusters are at most 3 characters; limiting the scan
/// prevents modifying vowels in a different syllable (e.g. `dang d` + `a`
/// should not change the `a` in `dang`).
const MAX_FLEXIBLE_BACKTRACK: usize = 3;

fn is_vowel(c: char) -> bool {
    VOWEL_ACCENTED.contains(&c)
}

/// Strip tone from a Vietnamese vowel, returning (base_modified_vowel, tone_char_or_none)
/// where base_modified_vowel still has its shape modifier (e.g., 'â', 'ă', 'ô', 'ơ').
fn strip_tone(c: char) -> (char, Option<char>) {
    match c {
        'a' => ('a', None), 'á' => ('a', Some('s')), 'à' => ('a', Some('f')),
        'ả' => ('a', Some('r')), 'ã' => ('a', Some('x')), 'ạ' => ('a', Some('j')),
        'ă' => ('ă', None), 'ắ' => ('ă', Some('s')), 'ằ' => ('ă', Some('f')),
        'ẳ' => ('ă', Some('r')), 'ẵ' => ('ă', Some('x')), 'ặ' => ('ă', Some('j')),
        'â' => ('â', None), 'ấ' => ('â', Some('s')), 'ầ' => ('â', Some('f')),
        'ẩ' => ('â', Some('r')), 'ẫ' => ('â', Some('x')), 'ậ' => ('â', Some('j')),
        'e' => ('e', None), 'é' => ('e', Some('s')), 'è' => ('e', Some('f')),
        'ẻ' => ('e', Some('r')), 'ẽ' => ('e', Some('x')), 'ẹ' => ('e', Some('j')),
        'ê' => ('ê', None), 'ế' => ('ê', Some('s')), 'ề' => ('ê', Some('f')),
        'ể' => ('ê', Some('r')), 'ễ' => ('ê', Some('x')), 'ệ' => ('ê', Some('j')),
        'i' => ('i', None), 'í' => ('i', Some('s')), 'ì' => ('i', Some('f')),
        'ỉ' => ('i', Some('r')), 'ĩ' => ('i', Some('x')), 'ị' => ('i', Some('j')),
        'o' => ('o', None), 'ó' => ('o', Some('s')), 'ò' => ('o', Some('f')),
        'ỏ' => ('o', Some('r')), 'õ' => ('o', Some('x')), 'ọ' => ('o', Some('j')),
        'ô' => ('ô', None), 'ố' => ('ô', Some('s')), 'ồ' => ('ô', Some('f')),
        'ổ' => ('ô', Some('r')), 'ỗ' => ('ô', Some('x')), 'ộ' => ('ô', Some('j')),
        'ơ' => ('ơ', None), 'ớ' => ('ơ', Some('s')), 'ờ' => ('ơ', Some('f')),
        'ở' => ('ơ', Some('r')), 'ỡ' => ('ơ', Some('x')), 'ợ' => ('ơ', Some('j')),
        'u' => ('u', None), 'ú' => ('u', Some('s')), 'ù' => ('u', Some('f')),
        'ủ' => ('u', Some('r')), 'ũ' => ('u', Some('x')), 'ụ' => ('u', Some('j')),
        'ư' => ('ư', None), 'ứ' => ('ư', Some('s')), 'ừ' => ('ư', Some('f')),
        'ử' => ('ư', Some('r')), 'ữ' => ('ư', Some('x')), 'ự' => ('ư', Some('j')),
        'y' => ('y', None), 'ý' => ('y', Some('s')), 'ỳ' => ('y', Some('f')),
        'ỷ' => ('y', Some('r')), 'ỹ' => ('y', Some('x')), 'ỵ' => ('y', Some('j')),
        _ => (c, None),
    }
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

    // Tone overriding: vowel already has a tone → strip it and apply the new one
    let (base, _) = strip_tone(vowel);
    if base != vowel {
        for &(v, t, result) in table {
            if v == base && t == tone {
                return Some(result);
            }
        }
    }

    None
}

/// Override the shape modifier on a vowel with a different one.
/// Preserves any existing tone.
/// Telex mappings: â↔ă via w/a, ô↔ơ via w/o
fn override_telex_modifier(vowel: char, key: char) -> Option<char> {
    let (base, tone) = strip_tone(vowel);
    let new_base = match (base, key) {
        ('â', 'w') => Some('ă'),
        ('ă', 'a') => Some('â'),
        ('ô', 'w') => Some('ơ'),
        ('ơ', 'o') => Some('ô'),
        _ => None,
    }?;
    match tone {
        None => Some(new_base),
        Some(t) => apply_tone_to_vowel(new_base, t),
    }
}


fn apply_w_to_vowel(vowel: char) -> Option<char> {
    // Telex: aw=ă, ow=ơ, ew=ê, uw=ư
    // (aa=â, ee=ê, oo=ô are handled by double-letter logic)
    match vowel {
        'a' => Some('ă'),
        'o' => Some('ơ'),
        'e' => Some('ê'),
        'u' => Some('ư'),
        _ => None,
    }
}

// Smart cluster helpers: detect "uo" → "ươ" and transfer tones

fn is_u_vowel(c: char) -> bool {
    matches!(c, 'u' | 'ú' | 'ù' | 'ủ' | 'ũ' | 'ụ')
}

fn is_o_vowel(c: char) -> bool {
    matches!(c, 'o' | 'ó' | 'ò' | 'ỏ' | 'õ' | 'ọ')
}

/// Determine the tone character (Telex) from a toned vowel.
/// 'u' variants → Some('tone_char'), plain vowels → None.
fn tone_of_vowel(c: char) -> Option<char> {
    match c {
        'u' | 'o' | 'a' | 'e' | 'i' | 'y' | 'ă' | 'â' | 'ê' | 'ô' | 'ơ' | 'ư' => None,
        'ù' | 'ò' | 'à' | 'è' | 'ì' | 'ỳ' | 'ằ' | 'ầ' | 'ề' | 'ồ' | 'ờ' | 'ừ' => Some('f'),
        'ú' | 'ó' | 'á' | 'é' | 'í' | 'ý' | 'ắ' | 'ấ' | 'ế' | 'ố' | 'ớ' | 'ứ' => Some('s'),
        'ủ' | 'ỏ' | 'ả' | 'ẻ' | 'ỉ' | 'ỷ' | 'ẳ' | 'ẩ' | 'ể' | 'ổ' | 'ở' | 'ử' => Some('r'),
        'ũ' | 'õ' | 'ã' | 'ẽ' | 'ĩ' | 'ỹ' | 'ẵ' | 'ẫ' | 'ễ' | 'ỗ' | 'ỡ' | 'ữ' => Some('x'),
        'ụ' | 'ọ' | 'ạ' | 'ẹ' | 'ị' | 'ỵ' | 'ặ' | 'ậ' | 'ệ' | 'ộ' | 'ợ' | 'ự' => Some('j'),
        _ => None,
    }
}

/// Apply a Telex tone to the vowel 'ơ', returning the toned variant.
fn apply_tone_to_ơ_char(tone: Option<char>) -> char {
    match tone {
        None       => 'ơ',
        Some('f')  => 'ờ',
        Some('s')  => 'ớ',
        Some('r')  => 'ở',
        Some('x')  => 'ỡ',
        Some('j')  => 'ợ',
        _          => 'ơ',
    }
}

/// Convert a "uo" cluster (with possible tones) into "ươ" with correct tone placement.
/// The tone ends up on 'ơ' (second vowel of ươ) regardless of which vowel carried it.
fn uo_to_uơ(u_char: char, o_char: char) -> (char, char) {
    let o_tone = tone_of_vowel(o_char);
    let u_tone = tone_of_vowel(u_char);
    let tone = o_tone.or(u_tone);
    ('ư', apply_tone_to_ơ_char(tone))
}

/// Check whether a position `i` (pointing at 'o' in a potential "uo" cluster) is
/// preceded by 'q' (making it a "qu" consonant cluster, not a vowel pair).
fn is_q_before_u(chars: &[char], i: usize) -> bool {
    i > 1 && chars[i - 2] == 'q'
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

    pub fn pop(&mut self) {
        self.buffer.pop();
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
                        // For oa, oe, uâ, uê, uơ, uy, iê, yê → tone on second vowel
                        let tone_on_second = matches!(
                            (first, second),
                            ('o', 'a') | ('o', 'e')
                            | ('u', 'â') | ('u', 'ê') | ('u', 'ơ') | ('u', 'y')
                            | ('ư', 'ơ')
                            | ('i', 'ê') | ('y', 'ê')
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

        // Check for double-letter pattern (last char matches)
        if let Some(last_ch) = self.buffer.chars().last() {
            if last_ch == ch {
                let replacement = match ch {
                    'a' => Some('â'),
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
            // Smart cluster reverse: "ươ" + o → "uô"
            if ch == 'o' && is_vowel(last_ch) {
                let strip = strip_tone(last_ch);
                if strip.0 == 'ơ' {
                    let mut chars: Vec<char> = self.buffer.chars().collect();
                    if chars.len() >= 2 && chars[chars.len() - 2] == 'ư' {
                        let ơ_char = chars.pop().unwrap();
                        chars.pop().unwrap();
                        let tone = tone_of_vowel(ơ_char);
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
            // Modifier override: if last vowel has a different modifier that can
            // be replaced by this key (e.g., ă+a→â, ơ+o→ô)
            if is_vowel(last_ch) && ch != last_ch {
                if let Some(modified) = override_telex_modifier(last_ch, ch) {
                    self.buffer.pop();
                    self.buffer.push(modified);
                    return None;
                }
            }
        }

        // Flexible placement: if last char is not a vowel, scan the last
        // N chars for a matching vowel to form a double-vowel pair, or for
        // a modified vowel that can be overridden by this key.
        // Limited backtrack prevents modifying vowels in a different syllable.
        if matches!(ch, 'a' | 'e' | 'o') {
            if let Some(last_ch) = self.buffer.chars().last() {
                if !is_vowel(last_ch) {
                    let chars: Vec<char> = self.buffer.chars().collect();
                    let start = chars.len().saturating_sub(MAX_FLEXIBLE_BACKTRACK);
                    for i in (start..chars.len()).rev() {
                        if is_vowel(chars[i]) {
                            if chars[i] == ch {
                                let replacement = match ch {
                                    'a' => 'â',
                                    'e' => 'ê',
                                    'o' => 'ô',
                                    _ => unreachable!(),
                                };
                                self.buffer = chars[..i].iter().collect::<String>();
                                self.buffer.push(replacement);
                                for &c in &chars[i + 1..] {
                                    self.buffer.push(c);
                                }
                                return None;
                            }
                            // Smart cluster reverse: "ươ" + o → "uô" (flexible)
                            if ch == 'o' {
                                let strip = strip_tone(chars[i]);
                                if strip.0 == 'ơ' && i > 0 && chars[i - 1] == 'ư' {
                                    let ơ_char = chars[i];
                                    let tone = tone_of_vowel(ơ_char);
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
                            // Modifier override for flexible path
                            if let Some(modified) = override_telex_modifier(chars[i], ch) {
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
        }

        self.buffer.push(ch);
        None
    }

    fn process_w(&mut self) -> Option<EngineEvent> {
        self.apply_pending_to_last_vowel();

        // Direct: last char is a vowel
        if let Some(last_ch) = self.buffer.chars().last() {
            if is_o_vowel(last_ch) {
                // Smart cluster "uo" → "ươ"
                let mut chars: Vec<char> = self.buffer.chars().collect();
                if chars.len() >= 2 && is_u_vowel(chars[chars.len() - 2]) && !is_q_before_u(&chars, chars.len() - 1) {
                    let o_char = chars.pop().unwrap();
                    let u_char = chars.pop().unwrap();
                    let (new_first, new_second) = uo_to_uơ(u_char, o_char);
                    self.buffer = chars.into_iter().collect::<String>();
                    self.buffer.push(new_first);
                    self.buffer.push(new_second);
                    return None;
                }
            }
            if is_vowel(last_ch) {
                if let Some(modified) = apply_w_to_vowel(last_ch) {
                    self.buffer.pop();
                    self.buffer.push(modified);
                    return None;
                }
                // Smart cluster override: "uô" + w → "ươ"
                let strip = strip_tone(last_ch);
                if strip.0 == 'ô' || strip.0 == 'ơ' {
                    let mut chars: Vec<char> = self.buffer.chars().collect();
                    if chars.len() >= 2 && is_u_vowel(chars[chars.len() - 2]) && !is_q_before_u(&chars, chars.len() - 1) {
                        let o_char = chars.pop().unwrap();
                        let u_char = chars.pop().unwrap();
                        let (new_first, new_second) = uo_to_uơ(u_char, o_char);
                        self.buffer = chars.into_iter().collect::<String>();
                        self.buffer.push(new_first);
                        self.buffer.push(new_second);
                        return None;
                    }
                }
                // Modifier override: if vowel already has a different modifier
                if let Some(modified) = override_telex_modifier(last_ch, 'w') {
                    self.buffer.pop();
                    self.buffer.push(modified);
                    return None;
                }
            }
        }

        // Flexible placement: if last char is not a vowel, scan the last
        // N chars for a vowel to apply the w modifier.
        if let Some(last_ch) = self.buffer.chars().last() {
            if !is_vowel(last_ch) {
                let chars: Vec<char> = self.buffer.chars().collect();
                let start = chars.len().saturating_sub(MAX_FLEXIBLE_BACKTRACK);
                for i in (start..chars.len()).rev() {
                    if is_vowel(chars[i]) {
                        // Smart cluster "uo" → "ươ" (flexible)
                        if is_o_vowel(chars[i]) && i > 0 && is_u_vowel(chars[i - 1]) && !is_q_before_u(&chars, i) {
                            let (new_first, new_second) = uo_to_uơ(chars[i - 1], chars[i]);
                            self.buffer = chars[..i - 1].iter().collect::<String>();
                            self.buffer.push(new_first);
                            self.buffer.push(new_second);
                            for &c in &chars[i + 1..] {
                                self.buffer.push(c);
                            }
                            return None;
                        }
                        if let Some(modified) = apply_w_to_vowel(chars[i]) {
                            self.buffer = chars[..i].iter().collect::<String>();
                            self.buffer.push(modified);
                            for &c in &chars[i + 1..] {
                                self.buffer.push(c);
                            }
                            return None;
                        }
                        // Smart cluster override: "uô" + w → "ươ" (flexible)
                        if i > 0 && is_u_vowel(chars[i - 1]) && !is_q_before_u(&chars, i) {
                            let strip = strip_tone(chars[i]);
                            if strip.0 == 'ô' || strip.0 == 'ơ' {
                                let (new_first, new_second) = uo_to_uơ(chars[i - 1], chars[i]);
                                self.buffer = chars[..i - 1].iter().collect::<String>();
                                self.buffer.push(new_first);
                                self.buffer.push(new_second);
                                for &c in &chars[i + 1..] {
                                    self.buffer.push(c);
                                }
                                return None;
                            }
                        }
                        // Modifier override: vowel already has a different modifier
                        if let Some(modified) = override_telex_modifier(chars[i], 'w') {
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

