#[cfg(test)]
mod tests {
    use crate::{Engine, EngineEvent, InputMethod};

    fn process_input(engine: &mut Engine, input: &str) -> Vec<EngineEvent> {
        let mut events = Vec::new();
        for ch in input.chars() {
            if let Some(event) = engine.process_key(ch) {
                let is_replace = matches!(&event, EngineEvent::Replace { .. });
                let fl = is_flush_char(ch);
                events.push(event);
                if is_replace && fl {
                    events.push(EngineEvent::Insert(ch.to_string()));
                }
            } else if engine.is_enabled() {
                events.push(EngineEvent::Insert(ch.to_string()));
            }
        }
        events
    }

    fn is_flush_char(ch: char) -> bool {
        matches!(ch, ' ' | '\t' | '.' | ',' | '!' | '?' | ';' | ':' | '\n')
    }

    fn get_display(events: &[EngineEvent]) -> String {
        let mut display = String::new();
        for ev in events {
            match ev {
                EngineEvent::Flush(text) | EngineEvent::Paste(text) => {
                    display.push_str(text);
                }
                EngineEvent::Insert(text) => {
                    display.push_str(text);
                }
                EngineEvent::Replace { backspaces, insert } => {
                    for _ in 0..*backspaces {
                        display.pop();
                    }
                    display.push_str(insert);
                }
                EngineEvent::AutoRestore(word) => {
                    for _ in 0..word.len() {
                        display.pop();
                    }
                    display.push_str(word);
                }
                EngineEvent::UndoTones { backspaces, restored } => {
                    for _ in 0..*backspaces {
                        display.pop();
                    }
                    display.push_str(restored);
                }
            }
        }
        display
    }

    // ================================================================
    // Telex: Vowel combinations
    // ================================================================

    #[test]
    fn telex_double_a() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "aa")), "â");
    }

    #[test]
    fn telex_double_e() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "ee")), "ê");
    }

    #[test]
    fn telex_double_o() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "oo")), "ô");
    }

    #[test]
    fn telex_aw() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "aw")), "ă");
    }

    #[test]
    fn telex_ow() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "ow")), "ơ");
    }

    #[test]
    fn telex_uw() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "uw")), "ư");
    }

    // ================================================================
    // Telex: Tones on all vowels
    // ================================================================

    #[test]
    fn telex_tone_a_sac() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "as")), "á");
    }

    #[test]
    fn telex_tone_a_huyen() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "af")), "à");
    }

    #[test]
    fn telex_tone_a_hoi() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "ar")), "ả");
    }

    #[test]
    fn telex_tone_a_nga() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "ax")), "ã");
    }

    #[test]
    fn telex_tone_a_nang() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "aj")), "ạ");
    }

    #[test]
    fn telex_tone_e_sac() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "es")), "é");
    }

    #[test]
    fn telex_tone_i_sac() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "is")), "í");
    }

    // ================================================================
    // Telex: Tones on modified vowels
    // ================================================================

    #[test]
    fn telex_tone_aa_sac() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "aas")), "ấ");
    }

    #[test]
    fn telex_tone_aw_sac() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "aws")), "ắ");
    }

    #[test]
    fn telex_tone_ee_sac() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "ees")), "ế");
    }

    #[test]
    fn telex_tone_ow_sac() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "ows")), "ớ");
    }

    #[test]
    fn telex_tone_uw_sac() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "uws")), "ứ");
    }

    // ================================================================
    // Telex: Compound vowels with tones
    // ================================================================

    #[test]
    fn telex_oa_tone() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "oas")), "oá");
    }

    #[test]
    fn telex_uy_tone() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "uys")), "uý");
    }

    // ================================================================
    // Telex: Full Vietnamese words
    // ================================================================

    #[test]
    fn telex_word_chao() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "chafo")), "chào");
    }

    #[test]
    fn telex_word_duong() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "dduwowngf")), "đường");
    }

    #[test]
    fn telex_word_cam_on() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "cams own")), "cám ơn");
    }

    // ================================================================
    // VNI: Tones
    // ================================================================

    #[test]
    fn vni_a_sac() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "a1")), "á");
    }

    #[test]
    fn vni_a_huyen() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "a2")), "à");
    }

    #[test]
    fn vni_a_hoi() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "a3")), "ả");
    }

    #[test]
    fn vni_a_nga() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "a4")), "ã");
    }

    #[test]
    fn vni_a_nang() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "a5")), "ạ");
    }

    // ================================================================
    // VNI: Vowel modifications
    // ================================================================

    #[test]
    fn vni_a6_aa() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "a6")), "â");
    }

    #[test]
    fn vni_a8_aw() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "a8")), "ă");
    }

    #[test]
    fn vni_e6_ee() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "e6")), "ê");
    }

    #[test]
    fn vni_o6_oo() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "o6")), "ô");
    }

    #[test]
    fn vni_o7_ow() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "o7")), "ơ");
    }

    #[test]
    fn vni_u7_uw() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "u7")), "ư");
    }

    // ================================================================
    // VNI: Tone on modified vowel
    // ================================================================

    #[test]
    fn vni_aa_sac() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "a61")), "ấ");
    }

    #[test]
    fn vni_aw_sac() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "a81")), "ắ");
    }

    // ================================================================
    // VNI: Full Vietnamese words
    // ================================================================

    #[test]
    fn vni_word_tieng() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "tie6ng1")), "tiếng");
    }

    #[test]
    fn vni_word_duong() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "d9u7o7ng2")), "đường");
    }

    // ================================================================
    // Telex: dd
    // ================================================================

    #[test]
    fn telex_dd() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "dd")), "đ");
    }

    #[test]
    fn vni_d9() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "d9")), "đ");
    }

    // ================================================================
    // Uppercase preservation
    // ================================================================

    #[test]
    fn telex_uppercase_tieng() {
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "Tieengs");
        let display = get_display(&events);
        assert_eq!(display, "Tiếng");
    }

    // ================================================================
    // Macros
    // ================================================================

    #[test]
    fn macro_ko() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("ko".into(), "không".into());
        let events = process_input(&mut e, "ko ");
        assert_eq!(get_display(&events), "không ");
    }

    #[test]
    fn macro_clear() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("ko".into(), "không".into());
        e.clear_macros();
        let events = process_input(&mut e, "ko ");
        assert_eq!(get_display(&events), "ko ");
    }

    // ================================================================
    // Toggle enabled/disabled
    // ================================================================

    #[test]
    fn toggle_disabled() {
        let mut e = Engine::new(InputMethod::Telex);
        e.set_enabled(false);
        // When disabled, chars pass through as Insert events
        let events = process_input(&mut e, "aas");
        // a,a,s → "aas" via Insert events
        assert_eq!(get_display(&events), "aas");
    }

    #[test]
    fn toggle_reenabled() {
        let mut e = Engine::new(InputMethod::Telex);
        e.set_enabled(false);
        e.set_enabled(true);
        assert_eq!(get_display(&process_input(&mut e, "aas")), "ấ");
    }

    // ================================================================
    // Replay keystrokes
    // ================================================================

    #[test]
    fn replay_telex_chao() {
        let macros = std::collections::HashMap::new();
        let (output, _) = Engine::replay_keystrokes(InputMethod::Telex, &macros, &['c', 'h', 'a', 'o', 'f']);
        assert_eq!(output, "chào");
    }

    #[test]
    fn replay_vni_tieng() {
        let macros = std::collections::HashMap::new();
        let (output, _) = Engine::replay_keystrokes(
            InputMethod::Vni, &macros,
            &['t', 'i', 'e', '6', 'n', 'g', '1'],
        );
        assert_eq!(output, "tiếng");
    }

    // ================================================================
    // Edge cases
    // ================================================================

    #[test]
    fn empty_input() {
        let mut e = Engine::new(InputMethod::Telex);
        assert!(process_input(&mut e, "").is_empty());
    }

    #[test]
    fn only_consonants() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "bcd")), "bcd");
    }

    #[test]
    fn numbers_passthrough() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "123")), "123");
    }

    #[test]
    fn tone_key_standalone() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "s")), "s");
    }

    #[test]
    fn reset_clears() {
        let mut e = Engine::new(InputMethod::Telex);
        e.process_key('a');
        e.process_key('a');
        e.reset();
        assert_eq!(e.buffer(), "");
    }

    #[test]
    fn method_switch() {
        let mut e = Engine::new(InputMethod::Telex);
        e.set_method(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "a1")), "á");
    }
}
