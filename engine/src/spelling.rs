// SPDX-License-Identifier: MIT
const FIRST_CONSONANT_SEQS: &[&str] = &[
    "b d đ g gh m n nh p ph r s t tr v z",
    "c h k kh qu th",
    "ch gi l ng ngh x",
    "đ l",
    "h",
];

const VOWEL_SEQS: &[&str] = &[
    "ê i ua uê uy y",
    "a iê oa uyê yê",
    "â ă e o oo ô ơ oe u ư uâ uô ươ",
    "oă",
    "uơ",
    "ai ao au âu ay ây eo êu ia iêu iu oai oao oay oeo oi ôi ơi ưa uây ui ưi uôi ươi ươu ưu uya uyu yêu",
    "ă",
    "i",
];

const LAST_CONSONANT_SEQS: &[&str] = &["ch nh", "c ng", "m n p t", "k", "c"];

const CV_MATRIX: &[&[usize]] = &[
    &[0, 1, 2, 5],
    &[0, 1, 2, 3, 4, 5],
    &[0, 1, 2, 3, 5],
    &[6],
    &[7],
];

const VC_MATRIX: &[&[usize]] = &[&[0, 2], &[0, 1, 2], &[1, 2], &[1, 2], &[], &[], &[3], &[4]];

fn strip_tone(c: char) -> char {
    match c {
        'à' | 'á' | 'ả' | 'ã' | 'ạ' => 'a',
        'ằ' | 'ắ' | 'ẳ' | 'ẵ' | 'ặ' => 'ă',
        'ầ' | 'ấ' | 'ẩ' | 'ẫ' | 'ậ' => 'â',
        'è' | 'é' | 'ẻ' | 'ẽ' | 'ẹ' => 'e',
        'ề' | 'ế' | 'ể' | 'ễ' | 'ệ' => 'ê',
        'ì' | 'í' | 'ỉ' | 'ĩ' | 'ị' => 'i',
        'ò' | 'ó' | 'ỏ' | 'õ' | 'ọ' => 'o',
        'ồ' | 'ố' | 'ổ' | 'ỗ' | 'ộ' => 'ô',
        'ờ' | 'ớ' | 'ở' | 'ỡ' | 'ợ' => 'ơ',
        'ù' | 'ú' | 'ủ' | 'ũ' | 'ụ' => 'u',
        'ừ' | 'ứ' | 'ử' | 'ữ' | 'ự' => 'ư',
        'ỳ' | 'ý' | 'ỷ' | 'ỹ' | 'ỵ' => 'y',
        _ => c,
    }
}

fn is_vowel(c: char) -> bool {
    matches!(
        c,
        'a' | 'à'
            | 'á'
            | 'ả'
            | 'ã'
            | 'ạ'
            | 'ă'
            | 'ằ'
            | 'ắ'
            | 'ẳ'
            | 'ẵ'
            | 'ặ'
            | 'â'
            | 'ầ'
            | 'ấ'
            | 'ẩ'
            | 'ẫ'
            | 'ậ'
            | 'e'
            | 'è'
            | 'é'
            | 'ẻ'
            | 'ẽ'
            | 'ẹ'
            | 'ê'
            | 'ề'
            | 'ế'
            | 'ể'
            | 'ễ'
            | 'ệ'
            | 'i'
            | 'ì'
            | 'í'
            | 'ỉ'
            | 'ĩ'
            | 'ị'
            | 'o'
            | 'ò'
            | 'ó'
            | 'ỏ'
            | 'õ'
            | 'ọ'
            | 'ô'
            | 'ồ'
            | 'ố'
            | 'ổ'
            | 'ỗ'
            | 'ộ'
            | 'ơ'
            | 'ờ'
            | 'ớ'
            | 'ở'
            | 'ỡ'
            | 'ợ'
            | 'u'
            | 'ù'
            | 'ú'
            | 'ủ'
            | 'ũ'
            | 'ụ'
            | 'ư'
            | 'ừ'
            | 'ứ'
            | 'ử'
            | 'ữ'
            | 'ự'
            | 'y'
            | 'ý'
            | 'ỳ'
            | 'ỷ'
            | 'ỹ'
            | 'ỵ'
    )
}

/// Partition a word into (first_consonant, vowel_cluster, last_consonant)
pub fn partition(word: &str) -> (String, String, String) {
    let chars: Vec<char> = word.chars().collect();
    let n = chars.len();
    if n == 0 {
        return (String::new(), String::new(), String::new());
    }

    // 1. Find the first vowel index
    let mut first_vowel_idx = None;
    for i in 0..n {
        if is_vowel(chars[i]) {
            first_vowel_idx = Some(i);
            break;
        }
    }

    let first_vowel = match first_vowel_idx {
        Some(idx) => idx,
        None => {
            return (word.to_string(), String::new(), String::new());
        }
    };

    let mut fc_end = first_vowel;

    // Adjust fc_end for "qu" or "gi" acting as onset
    if first_vowel == 1 && chars[0] == 'q' && chars[1] == 'u' && n > 2 && is_vowel(chars[2]) {
        fc_end = 2;
    }
    if first_vowel == 1 && chars[0] == 'g' && chars[1] == 'i' && n > 2 && is_vowel(chars[2]) {
        fc_end = 2;
    }

    // 2. Find the end of the vowel cluster
    let mut vo_end = fc_end;
    while vo_end < n && is_vowel(chars[vo_end]) {
        vo_end += 1;
    }

    let fc: String = chars[..fc_end].iter().collect();
    let vo: String = chars[fc_end..vo_end].iter().collect();
    let lc: String = chars[vo_end..].iter().collect();

    (fc, vo, lc)
}

fn lookup(seqs: &[&str], input: &str) -> Vec<usize> {
    let mut matching_indices = Vec::new();
    if input.is_empty() {
        return matching_indices;
    }

    for (index, row) in seqs.iter().enumerate() {
        for word in row.split_whitespace() {
            if word == input {
                matching_indices.push(index);
                break;
            }
        }
    }
    matching_indices
}

/// Check if a word is a valid Vietnamese syllable according to phonology rules
pub fn is_valid_vietnamese_syllable(word: &str) -> bool {
    let lowercase_word = word.to_lowercase();

    // Quick reject if it has foreign letters 'f', 'j', 'w', 'z'
    if lowercase_word
        .chars()
        .any(|c| matches!(c, 'f' | 'j' | 'w' | 'z'))
    {
        return false;
    }

    // Clean tones from the word to validate spelling structure
    let cleaned_word: String = lowercase_word.chars().map(strip_tone).collect();

    let (fc, vo, lc) = partition(&cleaned_word);

    // If there is no vowel, it must be a valid standalone consonant (like "d", "đ", etc.)
    // but typically a full syllable must have a vowel. Let's allow empty vowel only if it's
    // a valid first consonant of length 1 or 2 (e.g. for initials/abbreviations).
    if vo.is_empty() {
        return !fc.is_empty() && !lookup(FIRST_CONSONANT_SEQS, &fc).is_empty();
    }

    let fc_indices = if !fc.is_empty() {
        let indices = lookup(FIRST_CONSONANT_SEQS, &fc);
        if indices.is_empty() {
            return false; // Invalid onset consonant
        }
        Some(indices)
    } else {
        None
    };

    let vo_indices = lookup(VOWEL_SEQS, &vo);
    if vo_indices.is_empty() {
        return false; // Invalid vowel cluster
    }

    let lc_indices = if !lc.is_empty() {
        let indices = lookup(LAST_CONSONANT_SEQS, &lc);
        if indices.is_empty() {
            return false; // Invalid coda consonant
        }
        Some(indices)
    } else {
        None
    };

    // If we have an onset, check CV compatibility
    if let Some(ref fcs) = fc_indices {
        let mut cv_valid = false;
        for &fc_idx in fcs {
            if let Some(allowed_vos) = CV_MATRIX.get(fc_idx) {
                for &allowed_vo in *allowed_vos {
                    if vo_indices.contains(&allowed_vo) {
                        cv_valid = true;
                        break;
                    }
                }
            }
            if cv_valid {
                break;
            }
        }
        if !cv_valid {
            return false;
        }
    }

    // If we have a coda, check VC compatibility
    if let Some(ref lcs) = lc_indices {
        let mut vc_valid = false;
        for &vo_idx in &vo_indices {
            if let Some(allowed_lcs) = VC_MATRIX.get(vo_idx) {
                for &allowed_lc in *allowed_lcs {
                    if lcs.contains(&allowed_lc) {
                        vc_valid = true;
                        break;
                    }
                }
            }
            if vc_valid {
                break;
            }
        }
        if !vc_valid {
            return false;
        }
    } else {
        // If there's no coda, we must verify that the vowel allows having no coda
        // (all vowel sequences allow no coda, except some specific ones in matrix, but let's see:
        // vowel groups 4, 5 have no allowed last consonants in matrix, which is correct).
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_vietnamese_syllables() {
        assert!(is_valid_vietnamese_syllable("chuyên"));
        assert!(is_valid_vietnamese_syllable("tiếng"));
        assert!(is_valid_vietnamese_syllable("việt"));
        assert!(is_valid_vietnamese_syllable("quang"));
        assert!(is_valid_vietnamese_syllable("giá"));
        assert!(is_valid_vietnamese_syllable("oanh"));
        assert!(is_valid_vietnamese_syllable("anh"));
        assert!(is_valid_vietnamese_syllable("thuở"));
        assert!(is_valid_vietnamese_syllable("gì"));
    }

    #[test]
    fn test_invalid_vietnamese_syllables() {
        assert!(!is_valid_vietnamese_syllable("fast"));
        assert!(!is_valid_vietnamese_syllable("box"));
        assert!(!is_valid_vietnamese_syllable("study"));
        assert!(!is_valid_vietnamese_syllable("fát"));
        assert!(!is_valid_vietnamese_syllable("făst"));
        assert!(!is_valid_vietnamese_syllable("cargo"));
        assert!(!is_valid_vietnamese_syllable("rust"));
        assert!(!is_valid_vietnamese_syllable("status"));
    }
}
