// SPDX-License-Identifier: MIT
//! Telex encoder for vietc-vk.
//!
//! The virtual keyboard can only emit ASCII keystrokes, so a Vietnamese
//! paragraph must be encoded into Telex before being typed. Telex puts the
//! tone letter at the END of the syllable and lets the engine decide which
//! vowel carries the mark — so we only need per-syllable encoding, not
//! per-vowel tone placement (which VNI would require).
//!
//! The `simulate` helper drives the real vietc-engine so the encoder can be
//! verified without root (see the `#[cfg(test)]` block).

use std::collections::HashMap;

/// The full paragraph used as the end-to-end "input test".
pub const PARAGRAPH: &str = "Ngáy xữa ngày xửa, trong khu rừng nọ có thỏ và rùa là đôi bạn rất thân thiết với nhau. Thế nhưng, vào một ngày kia, cả hai lại xảy ra trận cãi vã chỉ vì muốn biết ai là người chạy nhanh nhất. Vậy là thỏ và rùa liền tổ chức một cuộc thi chạy để quyết định ra ai là người chạy nhanh hơn.

Lúc bắt đầu cuộc đua, chú thỏ chạy rất nhanh, vượt xa chú rùa cả một đoạn đường rất dài. Thỏ ta thấy vậy liền nghĩ bụng rằng rùa còn lâu mới đuổi kịp với mình nên chú thỏ cứ vui vẻ chậm rãi đi bộ, bắt bướm, đùa vui hái hoa, từ chỗ này đến chỗ khác cho đến khi thỏ mệt quá liền tìm gốc cây lớn để ngồi nghỉ và ngủ quên lúc nào không hay.

Trong lúc đó, chú rùa vẫn chậm rãi kiên trì vượt mọi vất vả khó nhọc, để đi từng bước trên đường đua.

Sau một khoảng thời gian rất lâu, Thỏ mới chợt tỉnh dậy, nhận ra Rùa đã đi rất xa và gần tới vạch đích. Thỏ thấy vậy liền ba chân bốn cẳng chạy đuổi theo nhưng đã muộn, chú rùa đã tới vạch đích đầu tiên. Và thế là chú thỏ đành phải chịu thua trước chú rùa.

Qua câu chuyện cổ tích loài vật này, lời khuyên cho các bé là không nên coi thường người khác, không nên tự cao tự đại trong mọi công việc nhé.";

/// `(accented_char, base_ascii, diacritic_doubling, tone_letter)`
/// `tone_letter` is `'\0'` when the character carries no tone.
const TELEX_MAP: &[(char, char, &str, char)] = &[
    // a-group
    ('á', 'a', "", 's'), ('à', 'a', "", 'f'), ('ả', 'a', "", 'r'), ('ã', 'a', "", 'x'), ('ạ', 'a', "", 'j'),
    ('â', 'a', "a", '\0'), ('ấ', 'a', "a", 's'), ('ầ', 'a', "a", 'f'), ('ẩ', 'a', "a", 'r'), ('ẫ', 'a', "a", 'x'), ('ậ', 'a', "a", 'j'),
    ('ă', 'a', "w", '\0'), ('ắ', 'a', "w", 's'), ('ằ', 'a', "w", 'f'), ('ẳ', 'a', "w", 'r'), ('ẵ', 'a', "w", 'x'), ('ặ', 'a', "w", 'j'),
    // e-group
    ('é', 'e', "", 's'), ('è', 'e', "", 'f'), ('ẻ', 'e', "", 'r'), ('ẽ', 'e', "", 'x'), ('ẹ', 'e', "", 'j'),
    ('ê', 'e', "e", '\0'), ('ế', 'e', "e", 's'), ('ề', 'e', "e", 'f'), ('ể', 'e', "e", 'r'), ('ễ', 'e', "e", 'x'), ('ệ', 'e', "e", 'j'),
    // i-group
    ('í', 'i', "", 's'), ('ì', 'i', "", 'f'), ('ỉ', 'i', "", 'r'), ('ĩ', 'i', "", 'x'), ('ị', 'i', "", 'j'),
    // o-group
    ('ó', 'o', "", 's'), ('ò', 'o', "", 'f'), ('ỏ', 'o', "", 'r'), ('õ', 'o', "", 'x'), ('ọ', 'o', "", 'j'),
    ('ô', 'o', "o", '\0'), ('ố', 'o', "o", 's'), ('ồ', 'o', "o", 'f'), ('ổ', 'o', "o", 'r'), ('ỗ', 'o', "o", 'x'), ('ộ', 'o', "o", 'j'),
    ('ơ', 'o', "w", '\0'), ('ớ', 'o', "w", 's'), ('ờ', 'o', "w", 'f'), ('ở', 'o', "w", 'r'), ('ỡ', 'o', "w", 'x'), ('ợ', 'o', "w", 'j'),
    // u-group
    ('ú', 'u', "", 's'), ('ù', 'u', "", 'f'), ('ủ', 'u', "", 'r'), ('ũ', 'u', "", 'x'), ('ụ', 'u', "", 'j'),
    ('ư', 'u', "w", '\0'), ('ứ', 'u', "w", 's'), ('ừ', 'u', "w", 'f'), ('ử', 'u', "w", 'r'), ('ữ', 'u', "w", 'x'), ('ự', 'u', "w", 'j'),
    // y-group
    ('ý', 'y', "", 's'), ('ỳ', 'y', "", 'f'), ('ỷ', 'y', "", 'r'), ('ỹ', 'y', "", 'x'), ('ỵ', 'y', "", 'j'),
    // uppercase a-group
    ('Á', 'A', "", 's'), ('À', 'A', "", 'f'), ('Ả', 'A', "", 'r'), ('Ã', 'A', "", 'x'), ('Ạ', 'A', "", 'j'),
    ('Â', 'A', "AA", '\0'), ('Ấ', 'A', "AA", 's'), ('Ầ', 'A', "AA", 'f'), ('Ẩ', 'A', "AA", 'r'), ('Ẫ', 'A', "AA", 'x'), ('Ậ', 'A', "AA", 'j'),
    ('Ă', 'A', "AW", '\0'), ('Ắ', 'A', "AW", 's'), ('Ằ', 'A', "AW", 'f'), ('Ẳ', 'A', "AW", 'r'), ('Ẵ', 'A', "AW", 'x'), ('Ặ', 'A', "AW", 'j'),
    // uppercase e-group
    ('É', 'E', "", 's'), ('È', 'E', "", 'f'), ('Ẻ', 'E', "", 'r'), ('Ẽ', 'E', "", 'x'), ('Ẹ', 'E', "", 'j'),
    ('Ê', 'E', "EE", '\0'), ('Ế', 'E', "EE", 's'), ('Ề', 'E', "EE", 'f'), ('Ể', 'E', "EE", 'r'), ('Ễ', 'E', "EE", 'x'), ('Ệ', 'E', "EE", 'j'),
    // uppercase i-group
    ('Í', 'I', "", 's'), ('Ì', 'I', "", 'f'), ('Ỉ', 'I', "", 'r'), ('Ĩ', 'I', "", 'x'), ('Ị', 'I', "", 'j'),
    // uppercase o-group
    ('Ó', 'O', "", 's'), ('Ò', 'O', "", 'f'), ('Ỏ', 'O', "", 'r'), ('Õ', 'O', "", 'x'), ('Ọ', 'O', "", 'j'),
    ('Ô', 'O', "OO", '\0'), ('Ố', 'O', "OO", 's'), ('Ồ', 'O', "OO", 'f'), ('Ổ', 'O', "OO", 'r'), ('Ỗ', 'O', "OO", 'x'), ('Ộ', 'O', "OO", 'j'),
    ('Ơ', 'O', "OW", '\0'), ('Ớ', 'O', "OW", 's'), ('Ờ', 'O', "OW", 'f'), ('Ở', 'O', "OW", 'r'), ('Ỡ', 'O', "OW", 'x'), ('Ợ', 'O', "OW", 'j'),
    // uppercase u-group
    ('Ú', 'U', "", 's'), ('Ù', 'U', "", 'f'), ('Ủ', 'U', "", 'r'), ('Ũ', 'U', "", 'x'), ('Ụ', 'U', "", 'j'),
    ('Ư', 'U', "UW", '\0'), ('Ứ', 'U', "UW", 's'), ('Ừ', 'U', "UW", 'f'), ('Ử', 'U', "UW", 'r'), ('Ữ', 'U', "UW", 'x'), ('Ự', 'U', "UW", 'j'),
    // uppercase y-group
    ('Ý', 'Y', "", 's'), ('Ỳ', 'Y', "", 'f'), ('Ỷ', 'Y', "", 'r'), ('Ỹ', 'Y', "", 'x'), ('Ỵ', 'Y', "", 'j'),
];

fn telex_map() -> &'static HashMap<char, (char, &'static str, char)> {
    use std::sync::OnceLock;
    static M: OnceLock<HashMap<char, (char, &'static str, char)>> = OnceLock::new();
    M.get_or_init(|| {
        let mut m = HashMap::new();
        for &(acc, base, dia, tone) in TELEX_MAP {
            m.insert(acc, (base, dia, tone));
        }
        m
    })
}

/// Encode a single Vietnamese character into its Telex keystroke sequence.
/// Returns `None` for characters with no Telex representation (plain ASCII
/// letters fall through to the caller, which emits them verbatim).
///
/// The returned string is only the base letter plus any shape modifier
/// doubling (e.g. `â` → `"aa"`, `ư` → `"uw"`); the tone letter, if any, is
/// returned separately so the caller can place it once at the end of the
/// syllable (Telex applies the trailing tone to the correct vowel).
fn encode_char(c: char) -> Option<(String, Option<char>)> {
    if c == 'đ' {
        return Some(("dd".into(), None));
    }
    if c == 'Đ' {
        return Some(("DD".into(), None));
    }
    let m = telex_map();
    let (base, dia, tone) = m.get(&c)?;
    let mut s = String::new();
    s.push(*base);
    if !dia.is_empty() {
        s.push_str(dia);
    }
    let tone = if *tone != '\0' { Some(*tone) } else { None };
    Some((s, tone))
}

/// Encode one syllable (a run of non-flush characters) into Telex.
/// The tone letter is appended at the end of the syllable so the engine can
/// place it on the correct vowel.
fn encode_syllable(syl: &str) -> String {
    let mut out = String::new();
    let mut syllable_tone: Option<char> = None;
    for ch in syl.chars() {
        if let Some((s, tone)) = encode_char(ch) {
            out.push_str(&s);
            if let Some(t) = tone {
                syllable_tone = Some(t);
            }
        } else {
            out.push(ch);
        }
    }
    if let Some(t) = syllable_tone {
        out.push(t);
    }
    out
}

/// Encode a full Vietnamese paragraph into a Telex keystroke string.
/// Spaces and punctuation are preserved verbatim; alphabetic syllables are
/// encoded.
pub fn to_telex(text: &str) -> String {
    let mut out = String::new();
    let mut syllable = String::new();
    for ch in text.chars() {
        if is_flush_char(ch) {
            if !syllable.is_empty() {
                out.push_str(&encode_syllable(&syllable));
                syllable.clear();
            }
            out.push(ch);
        } else {
            syllable.push(ch);
        }
    }
    if !syllable.is_empty() {
        out.push_str(&encode_syllable(&syllable));
    }
    out
}

/// VNI shape modifier digit for a (base, dia) pair, or `None` if unmodified.
/// vietc VNI: 6 = mũ (â/ê/ô), 7 = móc (ơ/ư), 8 = ă.
fn vni_mark_digit(base: char, dia: &str) -> Option<char> {
    match (base, dia) {
        ('a', "a") => Some('6'), // â
        ('a', "w") => Some('8'), // ă
        ('e', "e") => Some('6'), // ê
        ('o', "o") => Some('6'), // ô
        ('o', "w") => Some('7'), // ơ
        ('u', "w") => Some('7'), // ư
        _ => None,
    }
}

/// VNI tone digit for a Telex tone letter (vietc dialect: 1=sắc, 2=huyền,
/// 3=hỏi, 4=ngã, 5=nặng), or `None` if untoned.
fn vni_tone_digit(tone: char) -> Option<char> {
    match tone {
        's' => Some('1'),
        'f' => Some('2'),
        'r' => Some('3'),
        'x' => Some('4'),
        'j' => Some('5'),
        _ => None,
    }
}

/// Encode a single Vietnamese character into VNI keystrokes.
/// VNI carries the mark and tone inline per character (base + mark + tone), so
/// no end-of-syllable tone placement is needed.
fn vni_char(c: char) -> Option<String> {
    if c == 'đ' {
        return Some("d9".into());
    }
    if c == 'Đ' {
        return Some("D9".into());
    }
    let m = telex_map();
    let (base, dia, tone) = m.get(&c)?;
    let mut s = String::new();
    s.push(*base);
    if let Some(mk) = vni_mark_digit(*base, dia) {
        s.push(mk);
    }
    if let Some(td) = vni_tone_digit(*tone) {
        s.push(td);
    }
    Some(s)
}

/// Encode a full paragraph into VNI keystrokes.
pub fn to_vni(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if let Some(s) = vni_char(ch) {
            out.push_str(&s);
        } else {
            out.push(ch);
        }
    }
    out
}

/// Encode a paragraph using the given method (`"telex"` or `"vni"`).
pub fn to_viet(method: &str, text: &str) -> String {
    if method == "vni" {
        to_vni(text)
    } else {
        to_telex(text)
    }
}

fn is_flush_char(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\n' | '.' | ',' | '!' | '?' | ';' | ':' | '(' | ')')
}

#[cfg(test)]
mod tests {
    use super::*;
    use vietc_engine::{Engine, EngineEvent, InputMethod};

    /// Drive the real vietc-engine through a keystroke string and return the
    /// on-screen text, mirroring how the daemon applies engine output.
    fn simulate(seq: &str, method: InputMethod) -> String {
        let mut engine = Engine::new(method);
        engine.set_enabled(true);
        let mut screen = String::new();
        let trunc = |screen: &mut String, n: usize| {
            let keep: usize = screen.chars().count().saturating_sub(n);
            *screen = screen.chars().take(keep).collect();
        };
        for ch in seq.chars() {
            match engine.process_key(ch) {
                Some(EngineEvent::Replace { backspaces, insert }) => {
                    trunc(&mut screen, backspaces);
                    screen.push_str(&insert);
                }
                Some(EngineEvent::Insert(s)) => screen.push_str(&s),
                Some(EngineEvent::Flush(s)) => screen.push_str(&s),
                Some(EngineEvent::AutoRestore(w)) => screen.push_str(&w),
                Some(EngineEvent::UndoTones { backspaces, restored }) => {
                    trunc(&mut screen, backspaces);
                    screen.push_str(&restored);
                }
                Some(EngineEvent::Paste(s)) => screen.push_str(&s),
                None => screen.push(ch),
            }
        }
        screen
    }

    #[test]
    fn telex_encoder_roundtrip_paragraph() {
        let seq = to_telex(PARAGRAPH);
        let got = simulate(&seq, InputMethod::Telex);
        assert_eq!(got, PARAGRAPH, "\nencoded: {}\n", seq);
    }

    #[test]
    fn vni_encoder_roundtrip_paragraph() {
        let seq = to_vni(PARAGRAPH);
        let got = simulate(&seq, InputMethod::Vni);
        assert_eq!(got, PARAGRAPH, "\nencoded: {}\n", seq);
    }

    #[test]
    fn telex_encoder_basic() {
        assert_eq!(to_telex("Ngáy"), "Ngays");
        assert_eq!(to_telex("xừa"), "xuwaf");
        assert_eq!(to_telex("rừng"), "ruwngf");
        assert_eq!(to_telex("thỏ"), "thor");
        assert_eq!(to_telex("đôi"), "ddooi");
        assert_eq!(to_telex("quyết"), "quyeets");
        assert_eq!(to_telex("cuộc"), "cuoocj");
        assert_eq!(to_telex("chạy"), "chayj");
        // simulate a couple to ensure tone placement is correct
        assert_eq!(simulate(&to_telex("nào"), InputMethod::Telex), "nào");
        assert_eq!(simulate(&to_telex("nước"), InputMethod::Telex), "nước");
        assert_eq!(simulate(&to_telex("thiết"), InputMethod::Telex), "thiết");
    }

    #[test]
    fn vni_encoder_basic() {
        assert_eq!(to_vni("Ngáy"), "Nga1y");
        assert_eq!(to_vni("rừng"), "ru72ng");
        assert_eq!(to_vni("thỏ"), "tho3");
        assert_eq!(to_vni("đôi"), "d9o6i");
        assert_eq!(to_vni("quyết"), "quye61t");
        assert_eq!(to_vni("cuộc"), "cuo65c");
        assert_eq!(to_vni("chạy"), "cha5y");
        assert_eq!(simulate(&to_vni("nào"), InputMethod::Vni), "nào");
        assert_eq!(simulate(&to_vni("nước"), InputMethod::Vni), "nước");
        assert_eq!(simulate(&to_vni("thiết"), InputMethod::Vni), "thiết");
    }
}

