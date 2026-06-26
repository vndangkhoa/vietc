//! Regression tests for tone placement on syllables whose onset contains a
//! glide letter ("qu", "gi") and on the "uê"/"uơ" vowel clusters.

use std::collections::HashMap;
use vietc_engine::{Engine, InputMethod};

fn telex(keys: &str) -> String {
    Engine::replay_keystrokes(InputMethod::Telex, &HashMap::new(), &keys.chars().collect::<Vec<_>>()).0
}

fn vni(keys: &str) -> String {
    Engine::replay_keystrokes(InputMethod::Vni, &HashMap::new(), &keys.chars().collect::<Vec<_>>()).0
}

/// (telex keystrokes, vni keystrokes, expected word)
const CASES: &[(&str, &str, &str)] = &[
    // "qu" onset: the u is part of the consonant, tone stays on the nucleus.
    ("quar", "qua3", "quả"),
    ("quaf", "qua2", "quà"),
    ("quas", "qua1", "quá"),
    ("quaj", "qua5", "quạ"),
    // "gi" onset: the i is part of the consonant, tone stays on the nucleus.
    ("gias", "gia1", "giá"),
    ("giof", "gio2", "giò"),
    ("giowf", "gio72", "giờ"),
    ("giups", "giu1p", "giúp"),
    ("gieets", "gie61t", "giết"),
    ("giuwowngf", "giuo7ng2", "giường"),
    // "uê"/"uơ" clusters: tone belongs on the second vowel.
    ("thuees", "thue61", "thuế"),
    ("hueej", "hue65", "huệ"),
    // Controls that must keep working: bare "gì", "uy", "uâ", "uô".
    ("gif", "gi2", "gì"),
    ("quys", "quy1", "quý"),
    ("quaanf", "qua62n", "quần"),
    ("muoons", "muo61n", "muốn"),
];

#[test]
fn onset_glide_and_cluster_tone_placement() {
    let mut fails = Vec::new();
    for &(tk, vk, want) in CASES {
        let gt = telex(tk);
        if gt != want {
            fails.push(format!("TELEX {tk:>10} -> {gt:>8}  want {want}"));
        }
        let gv = vni(vk);
        if gv != want {
            fails.push(format!("VNI   {vk:>10} -> {gv:>8}  want {want}"));
        }
    }
    assert!(fails.is_empty(), "tone placement mismatches:\n{}", fails.join("\n"));
}
