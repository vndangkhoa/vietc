//! Tests for smart English auto-restore: when Vietnamese mode is on, words that
//! are clearly English / not valid Vietnamese revert to the raw keystrokes the
//! user typed, while genuine Vietnamese is kept.

use std::collections::HashMap;
use vietc_engine::{Engine, InputMethod};

fn telex(keys: &str) -> String {
    Engine::replay_keystrokes(InputMethod::Telex, &HashMap::new(), &keys.chars().collect::<Vec<_>>()).0
}

/// Resolve what would actually be committed for a Telex keystroke sequence,
/// applying the auto-restore decision the daemon makes on word commit.
fn committed(keys: &str) -> String {
    let composed = telex(keys);
    let raw: String = keys.chars().collect();
    if Engine::should_restore_word(&composed, &raw) {
        raw
    } else {
        composed
    }
}

#[test]
fn english_words_are_restored() {
    // (telex keystrokes, expected committed word)
    let cases = [
        ("fix", "fix"),       // foreign letter f
        ("cargo", "cargo"),   // invalid onset/coda
        ("status", "status"), // invalid cluster
        ("world", "world"),   // invalid coda
        ("english", "english"),
        ("sweet", "sweet"), // invalid onset "sw"
    ];
    for (keys, want) in cases {
        assert_eq!(committed(keys), want, "expected {keys} to restore to {want}");
    }
}

#[test]
fn vietnamese_words_are_kept() {
    let cases = [
        ("tieengs", "tiếng"),
        ("vieejt", "việt"),
        ("quar", "quả"),
        ("gif", "gì"),
        ("khoong", "không"),
        ("tooi", "tôi"),
        ("banhf", "bành"),
        ("ddi", "đi"),
    ];
    for (keys, want) in cases {
        assert_eq!(committed(keys), want, "expected {keys} to stay {want}");
    }
}

#[test]
fn untransformed_english_passes_through() {
    // Words with no tone/mark letters never transform, so nothing to restore.
    for keys in ["type", "code", "hello", "the", "and"] {
        assert_eq!(committed(keys), keys);
        assert!(!Engine::should_restore_word(&telex(keys), keys));
    }
}

#[test]
fn process_key_restores_on_flush() {
    // Drive the per-keystroke engine API and confirm the flush commits English.
    let mut engine = Engine::new(InputMethod::Telex);
    engine.set_enabled(true);
    for ch in "cargo".chars() {
        engine.process_key(ch);
    }
    // Mid-word the buffer is the Vietnamese composition.
    assert_eq!(engine.buffer(), "cảgo");
    // On flush the engine should emit a Replace back to the raw English word.
    let event = engine.process_key(' ');
    match event {
        Some(vietc_engine::EngineEvent::Replace { insert, .. }) => {
            assert_eq!(insert, "cargo");
        }
        other => panic!("expected Replace to 'cargo', got {other:?}"),
    }
}

#[test]
fn auto_restore_can_be_disabled() {
    let mut engine = Engine::new(InputMethod::Telex);
    engine.set_enabled(true);
    engine.set_auto_restore(false);
    for ch in "cargo".chars() {
        engine.process_key(ch);
    }
    // With auto-restore off the Vietnamese composition is kept on screen
    // (no restore back to the raw English keystrokes).
    assert_eq!(engine.buffer(), "cảgo");
    // The composed word is already correct on screen, so flushing emits no
    // Replace — re-typing it would race with the forwarded flush char and eat
    // the spacing. (Contrast with auto-restore on, which emits Replace→"cargo".)
    let event = engine.process_key(' ');
    assert!(
        event.is_none(),
        "with auto-restore off the composed VN word stays untouched on flush, got {event:?}"
    );
}
