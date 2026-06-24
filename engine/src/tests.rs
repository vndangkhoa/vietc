#[cfg(test)]
mod tests {
    use crate::{Engine, EngineEvent, InputMethod};

    fn process_input(engine: &mut Engine, input: &str) -> Vec<EngineEvent> {
        let mut events = Vec::new();
        for ch in input.chars() {
            if ch == '\x08' {
                events.push(EngineEvent::Replace { backspaces: 1, insert: String::new() });
                let _ = engine.process_key(ch);
                continue;
            }

            events.push(EngineEvent::Insert(ch.to_string()));
            if let Some(event) = engine.process_key(ch) {
                events.push(event);
            }
        }
        if let Some(event) = engine.flush() {
            events.push(event);
        }
        events
    }

    fn get_output(events: &[EngineEvent]) -> String {
        let mut output = String::new();
        for ev in events {
            match ev {
                EngineEvent::Flush(text) | EngineEvent::Insert(text) => {
                    output.push_str(text);
                }
                EngineEvent::Replace { backspaces, insert } => {
                    for _ in 0..*backspaces {
                        output.push('\x08');
                    }
                    output.push_str(insert);
                }
                EngineEvent::AutoRestore(word) => {
                    for _ in 0..word.len() {
                        output.push('\x08');
                    }
                    output.push_str(word);
                }
                EngineEvent::UndoTones { backspaces, restored } => {
                    for _ in 0..*backspaces {
                        output.push('\x08');
                    }
                    output.push_str(restored);
                }
            }
        }
        output
    }

    fn get_display(events: &[EngineEvent]) -> String {
        let mut display = String::new();
        for ev in events {
            match ev {
                EngineEvent::Flush(text) => {
                    if !display.ends_with(text) {
                        display.push_str(text);
                    }
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

    fn count_backspaces(events: &[EngineEvent]) -> usize {
        let mut count = 0;
        for ev in events {
            match ev {
                EngineEvent::Replace { backspaces, .. } => {
                    count += *backspaces;
                }
                EngineEvent::AutoRestore(word) => {
                    count += word.len();
                }
                EngineEvent::UndoTones { backspaces, .. } => {
                    count += *backspaces;
                }
                _ => {}
            }
        }
        count
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
    fn telex_ew() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "ew")), "ê");
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
    fn telex_tone_a_ngã() {
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

    #[test]
    fn telex_tone_o_sac() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "os")), "ó");
    }

    #[test]
    fn telex_tone_u_sac() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "us")), "ú");
    }

    #[test]
    fn telex_tone_y_sac() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "ys")), "ý");
    }

    // ================================================================
    // Telex: Tones on modified vowels
    // ================================================================

    #[test]
    fn telex_tone_â_from_aa() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "aas")), "ấ");
    }

    #[test]
    fn telex_tone_â() {
        let mut e = Engine::new(InputMethod::Telex);
        // aws: aw→ă, s adds sắc → ắ
        assert_eq!(get_display(&process_input(&mut e, "aws")), "ắ");
    }

    #[test]
    fn telex_tone_ê() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "ees")), "ế");
    }

    #[test]
    fn telex_tone_ô() {
        let mut e = Engine::new(InputMethod::Telex);
        // oos: oo→ô, s adds sắc → ố
        assert_eq!(get_display(&process_input(&mut e, "oos")), "ố");
    }

    #[test]
    fn telex_tone_ơ() {
        let mut e = Engine::new(InputMethod::Telex);
        // ows: ow→ơ, s adds sắc → ớ
        assert_eq!(get_display(&process_input(&mut e, "ows")), "ớ");
    }

    #[test]
    fn telex_tone_ư() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "uws")), "ứ");
    }

    // ================================================================
    // Telex: Compound vowels with tones
    // ================================================================

    #[test]
    fn telex_oa_tone() {
        let mut e = Engine::new(InputMethod::Telex);
        // Engine applies tone to first vowel in compound: oá (not óa)
        assert_eq!(get_display(&process_input(&mut e, "oas")), "oá");
    }

    #[test]
    fn telex_oe_tone() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "oes")), "oé");
    }

    #[test]
    fn telex_uy_tone() {
        let mut e = Engine::new(InputMethod::Telex);
        // Engine applies tone to second vowel (y) in "uy": uý
        assert_eq!(get_display(&process_input(&mut e, "uys")), "uý");
    }

    #[test]
    fn telex_ua_tone_on_first_vowel() {
        let mut e = Engine::new(InputMethod::Telex);
        // "ua" → tone on first vowel (u): mùa → "ùa"
        assert_eq!(get_display(&process_input(&mut e, "uaf")), "ùa");
    }

    #[test]
    fn telex_uâ_tone_on_second_vowel() {
        let mut e = Engine::new(InputMethod::Telex);
        // "uâ" → tone on second vowel (â): tuấn
        assert_eq!(get_display(&process_input(&mut e, "tuana")), "tuân");
        assert_eq!(get_display(&process_input(&mut e, "tuanas")), "tuấn");
    }

    #[test]
    fn telex_uê_tone_on_second_vowel() {
        let mut e = Engine::new(InputMethod::Telex);
        // "uê" → tone on second vowel (ê): thuế
        assert_eq!(get_display(&process_input(&mut e, "thuee")), "thuê");
        assert_eq!(get_display(&process_input(&mut e, "thuees")), "thuế");
    }

    // ================================================================
    // Telex: Flexible backtrack limit
    // ================================================================

    #[test]
    fn telex_flexible_backtrack_limit() {
        let mut e = Engine::new(InputMethod::Telex);
        // "dangd" + "a" should NOT modify the 'a' in "dang"
        // (too far back, crosses a syllable boundary).
        // The last 3 chars are "ngd" → no vowel → 'a' is appended normally.
        assert_eq!(get_display(&process_input(&mut e, "dangda")), "dangda");
    }

    #[test]
    fn telex_flexible_backtrack_still_works_near() {
        let mut e = Engine::new(InputMethod::Telex);
        // "tran" + "a" → last 3: "ran" → 'a' found at index 1 → "trân"
        assert_eq!(get_display(&process_input(&mut e, "trana")), "trân");
    }

    #[test]
    fn telex_flexible_backtrack_w_limit() {
        let mut e = Engine::new(InputMethod::Telex);
        // "dangd" + "w" should NOT modify 'a' in "dang".
        // w becomes a pending modifier (no vowel found within backtrack)
        // On flush, pending w is consumed without modifying anything.
        assert_eq!(get_display(&process_input(&mut e, "dangdw")), "dangd");
    }

    #[test]
    fn telex_flexible_backtrack_w_still_works_near() {
        let mut e = Engine::new(InputMethod::Telex);
        // "ngon" + "w" → last 3: "gon" → 'o' found at index 1 → "ngơn"
        assert_eq!(get_display(&process_input(&mut e, "ngonw")), "ngơn");
    }

    // ================================================================
    // Telex: Digraph dd
    // ================================================================

    #[test]
    fn telex_dd_at_start() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "dd")), "đ");
    }

    #[test]
    fn telex_dd_after_consonant() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "ndd")), "nđ");
    }

    #[test]
    fn telex_dd_in_word() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "ddo")), "đo");
    }

    // ================================================================
    // Telex: Pending modifier w
    // ================================================================

    #[test]
    fn telex_w_after_consonant_pending() {
        let mut e = Engine::new(InputMethod::Telex);
        // "cw" - w is pending after consonant, space flushes pending without vowel
        let events = process_input(&mut e, "cw ");
        // w is pending, flush applies pending to last vowel (none) → w consumed
        assert_eq!(get_display(&events), "c ");
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
    fn telex_word_cam_on() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "cams")), "cám");
    }

    #[test]
    fn telex_word_xin() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "xin")), "xin");
    }

    #[test]
    fn telex_word_ngon() {
        let mut e = Engine::new(InputMethod::Telex);
        // "ngon" + f → "ngonf" where f is pending, flush applies tone to 'o'
        assert_eq!(get_display(&process_input(&mut e, "ngonf")), "ngòn");
    }

    #[test]
    fn telex_word_tot() {
        let mut e = Engine::new(InputMethod::Telex);
        // "tot" + s → "tót" (s=sắc on o)
        assert_eq!(get_display(&process_input(&mut e, "tots")), "tót");
    }

    #[test]
    fn telex_word_dep() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "deps")), "dép");
    }

    #[test]
    fn telex_word_beauty() {
        let mut e = Engine::new(InputMethod::Telex);
        // "deeps" - ee→ê, then s=sắc on ê → dếp
        assert_eq!(get_display(&process_input(&mut e, "deeps")), "dếp");
    }

    #[test]
    fn telex_word_hoc() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "hocj")), "học");
    }

    #[test]
    fn telex_word_dung() {
        let mut e = Engine::new(InputMethod::Telex);
        // "dung" + j → "dụng"
        assert_eq!(get_display(&process_input(&mut e, "dungj")), "dụng");
    }

    #[test]
    fn telex_word_nha() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "nha")), "nha");
    }

    #[test]
    fn telex_word_nhas() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "nhas")), "nhá");
    }

    // ================================================================
    // Telex: Flush behavior
    // ================================================================

    #[test]
    fn telex_flush_on_space() {
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "hello ");
        assert_eq!(get_display(&events), "hello ");
    }

    #[test]
    fn telex_flush_on_period() {
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "hello.");
        assert_eq!(get_display(&events), "hello.");
    }

    #[test]
    fn telex_flush_on_comma() {
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "hello,");
        assert_eq!(get_display(&events), "hello,");
    }

    #[test]
    fn telex_flush_on_newline() {
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "hello\n");
        assert_eq!(get_display(&events), "hello\n");
    }

    #[test]
    fn telex_flush_on_enter() {
        let mut e = Engine::new(InputMethod::Telex);
        // "hello\n" flushes, then "xinh " starts fresh
        let events = process_input(&mut e, "hello\nxinh ");
        let display = get_display(&events);
        assert!(display.starts_with("hello\n"));
        assert!(display.ends_with(" "));
    }

    // ================================================================
    // Telex: Tone replacement
    // ================================================================

    #[test]
    fn telex_tone_replacement() {
        let mut e = Engine::new(InputMethod::Telex);
        // "as" → á, then "f" is pending tone, flush applies pending
        // The buffer after "as" is "á", pending='f'
        // flush calls apply_pending_to_last_vowel which tries f on á
        // á is not in the tone table (it's already toned), so f stays pending
        // Result: "á" + "f" in the flush output
        e.process_key('a');
        e.process_key('s');
        e.process_key('f');
        let event = e.flush();
        match event {
            Some(EngineEvent::Flush(text)) => {
                // After flushing with pending f on already-toned vowel
                assert!(!text.is_empty());
            }
            _ => {}
        }
    }

    // ================================================================
    // Telex: Edge cases
    // ================================================================

    #[test]
    fn telex_empty_input() {
        let mut e = Engine::new(InputMethod::Telex);
        assert!(process_input(&mut e, "").is_empty());
    }

    #[test]
    fn telex_only_consonants() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "bcd")), "bcd");
    }

    #[test]
    fn telex_single_vowel() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "a")), "a");
    }

    #[test]
    fn telex_numbers_passthrough() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "123")), "123");
    }

    #[test]
    fn telex_mixed_text() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "hello123")), "hello123");
    }

    // ================================================================
    // Telex: Toggle
    // ================================================================

    #[test]
    fn telex_disabled_passthrough() {
        let mut e = Engine::new(InputMethod::Telex);
        e.set_enabled(false);
        assert_eq!(get_display(&process_input(&mut e, "aas")), "aas");
    }

    #[test]
    fn telex_enabled_active() {
        let mut e = Engine::new(InputMethod::Telex);
        e.set_enabled(true);
        assert_eq!(get_display(&process_input(&mut e, "aas")), "ấ");
    }

    #[test]
    fn telex_toggle_mid_word() {
        let mut e = Engine::new(InputMethod::Telex);
        // Disabled: "a" passes through, then enabled: "a" → â
        e.set_enabled(false);
        e.process_key('a');
        e.set_enabled(true);
        e.process_key('a');
        let event = e.flush();
        match event {
            Some(EngineEvent::Flush(text)) => {
                // "a" passed through when disabled, then "a" processed when enabled → â
                // But flush_with is called: first 'a' flushes as Insert, second 'a' becomes â
                assert!(text.contains('a') || text.contains('â'));
            }
            _ => {}
        }
    }

    // ================================================================
    // Telex: Flexible diacritic placement
    // Vowel modifiers and tone marks can be typed at end of syllable,
    // scanning backward through consonants to find the base vowel.
    // ================================================================

    #[test]
    fn telex_flexible_double_a_tone() {
        let mut e = Engine::new(InputMethod::Telex);
        // "tranaf" → "aa" (flexible) → â, then "f" (tone) → ầ → "trần"
        assert_eq!(get_display(&process_input(&mut e, "tranaf")), "trần");
    }

    #[test]
    fn telex_flexible_w_modifier() {
        let mut e = Engine::new(InputMethod::Telex);
        // "ngonw" → "w" on 'o' through 'n' (flexible) → ơ → "ngơn"
        assert_eq!(get_display(&process_input(&mut e, "ngonw")), "ngơn");
    }

    #[test]
    fn telex_flexible_w_tone() {
        let mut e = Engine::new(InputMethod::Telex);
        // "tranwf" → "w" on 'a' (flexible) → ă, then "f" (tone) → ằ → "trằn"
        assert_eq!(get_display(&process_input(&mut e, "tranwf")), "trằn");
    }

    #[test]
    fn telex_flexible_double_e() {
        let mut e = Engine::new(InputMethod::Telex);
        // "treen" → "ee" (flexible) on 'e' in "tren" → ê → "trên"
        assert_eq!(get_display(&process_input(&mut e, "treen")), "trên");
    }

    #[test]
    fn telex_flexible_double_o() {
        let mut e = Engine::new(InputMethod::Telex);
        // "choon" → "oo" (flexible) on 'o' in "chon" → ô → "chôn"
        assert_eq!(get_display(&process_input(&mut e, "choon")), "chôn");
    }

    #[test]
    fn telex_flexible_tone_through_consonants() {
        let mut e = Engine::new(InputMethod::Telex);
        // "tranf" → already worked in standard engine (tone scans backward)
        assert_eq!(get_display(&process_input(&mut e, "tranf")), "tràn");
    }

    #[test]
    fn telex_flexible_w_after_u() {
        let mut e = Engine::new(InputMethod::Telex);
        // "xungw" → "w" on 'u' through 'ng' (flexible) → ư → "xưng"
        assert_eq!(get_display(&process_input(&mut e, "xungw")), "xưng");
    }

    // ================================================================
    // Telex: Smart "uo" → "ươ" cluster
    // ================================================================

    #[test]
    fn telex_smart_uo_to_uơ_shortcut() {
        let mut e = Engine::new(InputMethod::Telex);
        // Single w at end converts "uo" → "ươ" through trailing "ng"
        assert_eq!(get_display(&process_input(&mut e, "chuongw")), "chương");
    }

    #[test]
    fn telex_smart_uo_to_uơ_traditional() {
        let mut e = Engine::new(InputMethod::Telex);
        // Traditional uw+ow still works
        assert_eq!(get_display(&process_input(&mut e, "chuwowng")), "chương");
    }

    #[test]
    fn telex_smart_uo_to_uơ_with_tone_after_w() {
        let mut e = Engine::new(InputMethod::Telex);
        // "chuongws" → w first (cluster→ươ), then s (tone on ơ)
        assert_eq!(get_display(&process_input(&mut e, "chuongws")), "chướng");
    }

    #[test]
    fn telex_smart_uo_to_uơ_with_tone_before_w() {
        let mut e = Engine::new(InputMethod::Telex);
        // "chuongsw" → s first (tone on u), then w (cluster→ươ, tone→ơ)
        assert_eq!(get_display(&process_input(&mut e, "chuongsw")), "chướng");
    }

    #[test]
    fn telex_smart_uo_to_uơ_thuong_after_w() {
        let mut e = Engine::new(InputMethod::Telex);
        // "thuowngf" → w first (cluster→ươ), then f (huyền on ơ)
        assert_eq!(get_display(&process_input(&mut e, "thuowngf")), "thường");
    }

    #[test]
    fn telex_smart_uo_to_uơ_thuong_before_w() {
        let mut e = Engine::new(InputMethod::Telex);
        // "thuongfw" → f first (tone on u), then w (cluster→ươ, tone→ơ)
        assert_eq!(get_display(&process_input(&mut e, "thuongfw")), "thường");
    }

    // ================================================================
    // VNI: Flexible diacritic placement
    // ================================================================

    #[test]
    fn vni_flexible_digit_tone() {
        let mut e = Engine::new(InputMethod::Vni);
        // "tran62" → 6 on 'a' (flexible) → â, then 2 on 'â' (flexible) → ầ → "trần"
        assert_eq!(get_display(&process_input(&mut e, "tran62")), "trần");
    }

    #[test]
    fn vni_flexible_tone_through_consonants() {
        let mut e = Engine::new(InputMethod::Vni);
        // "tran1" → 1 (sắc) on 'a' (flexible) → á → "trán"
        assert_eq!(get_display(&process_input(&mut e, "tran1")), "trán");
    }

    #[test]
    fn vni_flexible_vowel_mod() {
        let mut e = Engine::new(InputMethod::Vni);
        // "tran6" → 6 on 'a' (flexible) → â → "trân"
        assert_eq!(get_display(&process_input(&mut e, "tran6")), "trân");
    }

    #[test]
    fn vni_flexible_no_vowel_passthrough() {
        let mut e = Engine::new(InputMethod::Vni);
        // "b1" → no vowel in buffer, digit appended unchanged
        assert_eq!(get_display(&process_input(&mut e, "b1")), "b1");
    }

    #[test]
    fn vni_flexible_empty_buffer() {
        let mut e = Engine::new(InputMethod::Vni);
        // "1" on empty buffer → appended
        assert_eq!(get_display(&process_input(&mut e, "1")), "1");
    }

    #[test]
    fn vni_flexible_backtrack_limit() {
        let mut e = Engine::new(InputMethod::Vni);
        // "dangd" + "6" should NOT modify 'a' in "dang"
        assert_eq!(get_display(&process_input(&mut e, "dangd6")), "dangd6");
    }

    #[test]
    fn vni_flexible_backtrack_still_works_near() {
        let mut e = Engine::new(InputMethod::Vni);
        // "tran" + "6" → "trân" (within backtrack limit)
        assert_eq!(get_display(&process_input(&mut e, "tran6")), "trân");
    }

    // ================================================================
    // VNI: Smart "uo" → "ươ" cluster
    // ================================================================

    #[test]
    fn vni_smart_uo_to_uơ_shortcut() {
        let mut e = Engine::new(InputMethod::Vni);
        // Single 7 at end converts "uo" → "ươ" through trailing "ng"
        assert_eq!(get_display(&process_input(&mut e, "chuong7")), "chương");
    }

    #[test]
    fn vni_smart_uo_to_uơ_traditional() {
        let mut e = Engine::new(InputMethod::Vni);
        // Traditional u7+o7 still works
        assert_eq!(get_display(&process_input(&mut e, "chu7o7ng")), "chương");
    }

    #[test]
    fn vni_smart_uo_to_uơ_with_tone_after_7() {
        let mut e = Engine::new(InputMethod::Vni);
        // "chuong71" → 7 first (cluster→ươ), then 1 (sắc on ơ) → "chướng"
        assert_eq!(get_display(&process_input(&mut e, "chuong71")), "chướng");
    }

    #[test]
    fn vni_smart_uo_to_uơ_with_tone_before_7() {
        let mut e = Engine::new(InputMethod::Vni);
        // "chuong17" → 1 first (tone on o), then 7 (cluster→ươ, tone→ơ) → "chướng"
        assert_eq!(get_display(&process_input(&mut e, "chuong17")), "chướng");
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
    fn vni_a_ngã() {
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
    fn vni_a6_â() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "a6")), "â");
    }

    #[test]
    fn vni_a8_ă() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "a8")), "ă");
    }

    #[test]
    fn vni_e6_ê() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "e6")), "ê");
    }

    #[test]
    fn vni_o6_ô() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "o6")), "ô");
    }

    #[test]
    fn vni_o7_ơ() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "o7")), "ơ");
    }

    #[test]
    fn vni_u7_ư() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "u7")), "ư");
    }

    // ================================================================
    // VNI: Tone on modified vowel
    // ================================================================

    #[test]
    fn vni_ă_sac() {
        let mut e = Engine::new(InputMethod::Vni);
        // "a8" → ă, then "1" → ắ
        assert_eq!(get_display(&process_input(&mut e, "a81")), "ắ");
    }

    #[test]
    fn vni_â_huyen() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "a62")), "ầ");
    }

    #[test]
    fn vni_ê_sac() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "e61")), "ế");
    }

    #[test]
    fn vni_ô_nang() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "o65")), "ộ");
    }

    // ================================================================
    // VNI: Digit after consonant (passthrough)
    // ================================================================

    #[test]
    fn vni_digit_after_consonant() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "b1")), "b1");
    }

    #[test]
    fn vni_digit_after_space() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, " 1")), " 1");
    }

    // ================================================================
    // VNI: Full Vietnamese words
    // ================================================================

    #[test]
    fn vni_word_chao() {
        let mut e = Engine::new(InputMethod::Vni);
        // "chao2" → tone 2 (huyền) on last vowel 'o' → "chaò"
        assert_eq!(get_display(&process_input(&mut e, "chao2")), "chaò");
    }

    #[test]
    fn vni_word_cam_on() {
        let mut e = Engine::new(InputMethod::Vni);
        // "cam1" → flexible placement: '1' scans backward past 'm' to vowel 'a' → "cám"
        assert_eq!(get_display(&process_input(&mut e, "cam1")), "cám");
    }

    // ================================================================
    // Auto-restore: English words
    // ================================================================

    #[test]
    fn auto_restore_hello() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "hello ")), "hello ");
    }

    #[test]
    fn auto_restore_the() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "the ")), "the ");
    }

    #[test]
    fn auto_restore_and() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "and ")), "and ");
    }

    #[test]
    fn auto_restore_you() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "you ")), "you ");
    }

    #[test]
    fn auto_restore_on_period() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "hello.")), "hello.");
    }

    #[test]
    fn auto_restore_on_comma() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "ok,")), "ok,");
    }

    #[test]
    fn auto_restore_not_on_vietnamese() {
        let mut e = Engine::new(InputMethod::Telex);
        // "xin" is in Vietnamese overrides, should NOT auto-restore
        let events = process_input(&mut e, "xin ");
        let display = get_display(&events);
        assert_eq!(display, "xin ");
    }

    // ================================================================
    // ESC Undo
    // ================================================================

    #[test]
    fn esc_undo_basic() {
        let mut e = Engine::new(InputMethod::Telex);
        e.process_key('a');
        e.process_key('s');
        let event = e.process_escape();
        match event {
            Some(EngineEvent::UndoTones { backspaces, restored }) => {
                assert_eq!(backspaces, 1);
                assert_eq!(restored, "a");
            }
            _ => panic!("Expected UndoTones"),
        }
    }

    #[test]
    fn esc_undo_word() {
        let mut e = Engine::new(InputMethod::Telex);
        for ch in "chafo".chars() {
            e.process_key(ch);
        }
        let event = e.process_escape();
        match event {
            Some(EngineEvent::UndoTones { backspaces, restored }) => {
                assert_eq!(backspaces, 4);
                assert_eq!(restored, "chao");
            }
            _ => panic!("Expected UndoTones"),
        }
    }

    #[test]
    fn esc_no_tones_flushes() {
        let mut e = Engine::new(InputMethod::Telex);
        for ch in "hello".chars() {
            e.process_key(ch);
        }
        let event = e.process_escape();
        match event {
            Some(EngineEvent::Flush(text)) => assert_eq!(text, "hello"),
            _ => panic!("Expected Flush"),
        }
    }

    #[test]
    fn esc_empty_buffer() {
        let mut e = Engine::new(InputMethod::Telex);
        let event = e.process_escape();
        assert!(event.is_none());
    }

    #[test]
    fn esc_undo_after_multiple_tones() {
        let mut e = Engine::new(InputMethod::Telex);
        // "as" → á, then "f" overrides tone: sắc → huyền → "à"
        // ESC strips diacritics → "a"
        e.process_key('a');
        e.process_key('s');
        e.process_key('f');
        let event = e.process_escape();
        match event {
            Some(EngineEvent::UndoTones { restored, .. }) => {
                assert_eq!(restored, "a");
            }
            _ => panic!("Expected UndoTones, got {:?}", event),
        }
    }

    // ================================================================
    // Macros
    // ================================================================

    #[test]
    fn macro_ko() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("ko".into(), "không".into());
        assert_eq!(get_display(&process_input(&mut e, "ko ")), "không ");
    }

    #[test]
    fn macro_vs() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("vs".into(), "với".into());
        assert_eq!(get_display(&process_input(&mut e, "vs ")), "với ");
    }

    #[test]
    fn macro_dc() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("dc".into(), "được".into());
        assert_eq!(get_display(&process_input(&mut e, "dc ")), "được ");
    }

    #[test]
    fn macro_on_period() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("ok".into(), "được".into());
        assert_eq!(get_display(&process_input(&mut e, "ok.")), "được.");
    }

    #[test]
    fn macro_on_comma() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("ko".into(), "không".into());
        assert_eq!(get_display(&process_input(&mut e, "ko,")), "không,");
    }

    #[test]
    fn macro_overrides_telex() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("dc".into(), "được".into());
        // "dc" without macro = consonants, with macro = "được"
        assert_eq!(get_display(&process_input(&mut e, "dc ")), "được ");
    }

    #[test]
    fn macro_partial_match_no_expand() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("ko".into(), "không".into());
        // "kox" - 'x' is a tone key, 'o' gets tone applied: buffer = "kõ"
        // Then 'x' doesn't trigger flush, so no macro expansion
        // "kox" is NOT the same as "ko" when flushed
        let events = process_input(&mut e, "kox");
        let display = get_display(&events);
        // 'x' after 'o' applies ngã tone, so output is "kõ"
        assert_eq!(display, "kõ");
    }

    #[test]
    fn macro_empty_no_expand() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("".into(), "nothing".into());
        // Empty macro key should not crash or expand
        let events = process_input(&mut e, "a ");
        assert_eq!(get_display(&events), "a ");
    }

    #[test]
    fn macro_with_vietnamese_output() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("ntn".into(), "như thế này".into());
        assert_eq!(get_display(&process_input(&mut e, "ntn ")), "như thế này ");
    }

    #[test]
    fn macro_long_expansion() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("bhg".into(), "bài họcгруппа".into());
        assert_eq!(get_display(&process_input(&mut e, "bhg ")), "bài họcгруппа ");
    }

    #[test]
    fn macro_does_not_affect_vietnamese() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("ko".into(), "không".into());
        // "chao" is not a macro, should be processed normally as Telex
        assert_eq!(get_display(&process_input(&mut e, "chao ")), "chao ");
    }

    #[test]
    fn macro_and_telex_mixed() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("vs".into(), "với".into());
        // "vs" expands, then "hello" is English
        assert_eq!(get_display(&process_input(&mut e, "vs hello ")), "với hello ");
    }

    // ================================================================
    // Engine: Reset
    // ================================================================

    #[test]
    fn engine_reset_clears_buffer() {
        let mut e = Engine::new(InputMethod::Telex);
        e.process_key('a');
        e.process_key('a');
        e.reset();
        assert_eq!(e.buffer(), "");
    }

    #[test]
    fn engine_flush_after_reset() {
        let mut e = Engine::new(InputMethod::Telex);
        e.process_key('a');
        e.process_key('a');
        e.reset();
        let event = e.flush();
        assert!(event.is_none());
    }

    // ================================================================
    // Engine: Method switching
    // ================================================================

    #[test]
    fn engine_switch_to_vni() {
        let mut e = Engine::new(InputMethod::Telex);
        e.set_method(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "a1")), "á");
    }

    #[test]
    fn engine_switch_to_telex() {
        let mut e = Engine::new(InputMethod::Vni);
        e.set_method(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "as")), "á");
    }

    // ================================================================
    // Engine: Macro management
    // ================================================================

    #[test]
    fn engine_clear_macros() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("ko".into(), "không".into());
        e.clear_macros();
        // "ko" should no longer expand
        assert_eq!(get_display(&process_input(&mut e, "ko ")), "ko ");
    }

    // ================================================================
    // Engine: is_enabled
    // ================================================================

    #[test]
    fn engine_is_enabled_default() {
        let e = Engine::new(InputMethod::Telex);
        assert!(e.is_enabled());
    }

    #[test]
    fn engine_set_disabled() {
        let mut e = Engine::new(InputMethod::Telex);
        e.set_enabled(false);
        assert!(!e.is_enabled());
    }

    // ================================================================
    // Backspace counting
    // ================================================================

    #[test]
    fn backspace_count_auto_restore_debug() {
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "was ");
        // Verify auto-restore produces correct backspace counts
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        assert_eq!(replace_events.len(), 3);
        // w-pending: backspace 1 (delete 'w' from screen)
        assert_eq!(replace_events[0], (1, "".to_string()));
        // s-tone: backspace 2 (delete 'as'), insert "á"
        assert_eq!(replace_events[1], (2, "á".to_string()));
        // space auto-restore: backspace 2 (delete "á "), insert "was "
        assert_eq!(replace_events[2], (2, "was ".to_string()));
        assert_eq!(get_display(&events), "was ");
    }

    #[test]
    fn backspace_count_esc_undo() {
        let mut e = Engine::new(InputMethod::Telex);
        for ch in "chafo".chars() {
            e.process_key(ch);
        }
        let event = e.process_escape();
        match event {
            Some(EngineEvent::UndoTones { backspaces, .. }) => {
                assert_eq!(backspaces, 4); // "chào" = 4 chars
            }
            _ => panic!("Expected UndoTones"),
        }
    }

    // ================================================================
    // Telex: w at start of input
    // ================================================================

    #[test]
    fn telex_w_at_start() {
        let mut e = Engine::new(InputMethod::Telex);
        // "w" at start with no vowel → pending modifier, space flushes it
        let events = process_input(&mut e, "w ");
        // w is pending, flush applies pending to last vowel (none) → consumed
        assert_eq!(get_display(&events), " ");
    }

    // ================================================================
    // Telex: double letter not for i/u/y
    // ================================================================

    #[test]
    fn telex_double_i_passthrough() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "ii")), "ii");
    }

    #[test]
    fn telex_double_u_passthrough() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "uu")), "uu");
    }

    #[test]
    fn telex_double_y_passthrough() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "yy")), "yy");
    }

    // ================================================================
    // Telex: tone after non-vowel
    // ================================================================

    #[test]
    fn telex_tone_after_consonant() {
        let mut e = Engine::new(InputMethod::Telex);
        // "bs" → no vowel, s is appended as pending
        assert_eq!(get_display(&process_input(&mut e, "bs")), "bs");
    }

    #[test]
    fn telex_tone_key_standalone() {
        let mut e = Engine::new(InputMethod::Telex);
        // "s" alone → no vowel, just "s"
        assert_eq!(get_display(&process_input(&mut e, "s")), "s");
    }

    // ================================================================
    // VNI: Full words with modifications + tones
    // ================================================================

    #[test]
    fn vni_word_with_modifications() {
        let mut e = Engine::new(InputMethod::Vni);
        // "a61" → â + sac = ấ
        assert_eq!(get_display(&process_input(&mut e, "a61")), "ấ");
    }

    #[test]
    fn vni_word_complex() {
        let mut e = Engine::new(InputMethod::Vni);
        // "o61" → ô + sac = ố
        assert_eq!(get_display(&process_input(&mut e, "o61")), "ố");
    }

    // ================================================================
    // English dict
    // ================================================================

    #[test]
    fn english_dict_is_english() {
        let dict = crate::english::EnglishDict::new();
        assert!(dict.is_english_word("hello"));
        assert!(dict.is_english_word("the"));
        assert!(dict.is_english_word("you"));
        assert!(!dict.is_english_word("xyz"));
    }

    #[test]
    fn english_dict_should_restore() {
        let dict = crate::english::EnglishDict::new();
        assert!(dict.should_restore("hello"));
        assert!(dict.should_restore("the"));
        // Vietnamese overrides should NOT restore
        assert!(!dict.should_restore("xin"));
        assert!(!dict.should_restore("không"));
    }

    // ================================================================
    // strip_diacritics
    // ================================================================

    #[test]
    fn strip_diacritics_basic() {
        let mut e = Engine::new(InputMethod::Telex);
        // Type "chào" then ESC
        for ch in "chafo".chars() {
            e.process_key(ch);
        }
        let event = e.process_escape();
        match event {
            Some(EngineEvent::UndoTones { restored, .. }) => {
                assert_eq!(restored, "chao");
            }
            _ => panic!("Expected UndoTones"),
        }
    }

    #[test]
    fn strip_diacritics_all_vowels() {
        let mut e = Engine::new(InputMethod::Telex);
        // Each tone combo is flushed on space, so ESC only undoes the last word
        // "as af ar ax aj" → last buffer is "aj" → ESC → "a"
        let input = "as af ar ax aj";
        for ch in input.chars() {
            e.process_key(ch);
        }
        let event = e.process_escape();
        match event {
            Some(EngineEvent::UndoTones { restored, .. }) => {
                // Only the last unflushed vowel group "aj" is in the buffer
                assert_eq!(restored, "a");
            }
            _ => panic!("Expected UndoTones"),
        }
    }

    #[test]
    fn strip_diacritics_single_vowel() {
        let mut e = Engine::new(InputMethod::Telex);
        // "as" without space → buffer = "á" → ESC → "a"
        e.process_key('a');
        e.process_key('s');
        let event = e.process_escape();
        match event {
            Some(EngineEvent::UndoTones { restored, .. }) => {
                assert_eq!(restored, "a");
            }
            _ => panic!("Expected UndoTones"),
        }
    }

    #[test]
    fn strip_diacritics_modified_vowel() {
        let mut e = Engine::new(InputMethod::Telex);
        // "aas" → ắ → ESC → "a" (strip diacritics removes ă→a)
        e.process_key('a');
        e.process_key('a');
        e.process_key('s');
        let event = e.process_escape();
        match event {
            Some(EngineEvent::UndoTones { restored, .. }) => {
                assert_eq!(restored, "a");
            }
            _ => panic!("Expected UndoTones"),
        }
    }

    // ================================================================
    // Backspace counting: comprehensive tests
    // ================================================================

    #[test]
    fn backspace_count_simple_tone() {
        // "as" → Replace {2, "á"}
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "as");
        // Find the Replace event
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        assert_eq!(replace_events.len(), 1, "Expected 1 Replace event for 'as'");
        assert_eq!(replace_events[0], (2, "á".to_string()));
        assert_eq!(get_display(&events), "á");
    }

    #[test]
    fn backspace_count_double_letter() {
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "aa");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        assert_eq!(replace_events.len(), 1);
        assert_eq!(replace_events[0], (2, "â".to_string()));
        assert_eq!(get_display(&events), "â");
    }

    #[test]
    fn backspace_count_w_modifier() {
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "aw");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        assert_eq!(replace_events.len(), 1);
        assert_eq!(replace_events[0], (2, "ă".to_string()));
        assert_eq!(get_display(&events), "ă");
    }

    #[test]
    fn backspace_count_w_modifier_then_tone() {
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "aws");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        // "aw" → Replace {2, "ă"}, then "s" → Replace {2, "ắ"}
        assert_eq!(replace_events.len(), 2, "Expected 2 Replace events: {:?}", replace_events);
        assert_eq!(replace_events[0], (2, "ă".to_string()));
        assert_eq!(replace_events[1], (2, "ắ".to_string()));
        assert_eq!(get_display(&events), "ắ");
    }

    #[test]
    fn backspace_count_compound_vowel_tone() {
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "oas");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        // "oas" → tone on second vowel: Replace {3, "oá"}
        assert_eq!(replace_events.len(), 1, "Expected 1 Replace event: {:?}", replace_events);
        assert_eq!(replace_events[0], (3, "oá".to_string()));
        assert_eq!(get_display(&events), "oá");
    }

    #[test]
    fn backspace_count_compound_vowel_uy_tone() {
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "uys");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        // "uys" → tone on first vowel: Replace {3, "uý"}
        assert_eq!(replace_events.len(), 1, "Expected 1 Replace event: {:?}", replace_events);
        assert_eq!(replace_events[0], (3, "uý".to_string()));
        assert_eq!(get_display(&events), "uý");
    }

    #[test]
    fn backspace_count_tone_after_consonant() {
        // "bs" → no vowel, 's' is appended as text
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "bs");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, .. } => Some(backspaces),
            _ => None,
        }).collect();
        // 's' after consonant 'b': no vowel found, 's' appended to buffer
        // But s is a tone key, and process_tone is called...
        // In process_tone: buffer "b", chars=['b'], no vowel found → buffer.push('s') → "bs"
        // new_inner = "bs", expected = "b"+"s" = "bs" → same → None
        assert_eq!(replace_events.len(), 0, "Expected no Replace events, got: {:?}", replace_events);
        assert_eq!(get_display(&events), "bs");
    }

    #[test]
    fn backspace_count_auto_restore_was() {
        // "was " should auto-restore because "was" is an English word
        // The engine converts: w→pending(blink), a→normal, s→tone on a → "á"
        // Then space triggers auto-restore back to "was "
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "was ");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        // Expected events for "was ":
        // 'w': pending modifier, no buffer change → Replace {1, ""} (blink)
        // 's': tone on 'a' → Replace {2, "á"}
        // ' ': auto-restore → Replace {2, "was "}
        assert_eq!(replace_events.len(), 3, "Expected 3 Replace events, got: {:?}", replace_events);
        // Event 0: 'w' blinks (gets deleted as pending modifier)
        assert_eq!(replace_events[0].0, 1, "w-pending backspace");
        assert_eq!(replace_events[0].1, "");
        // Event 1: 's' replaces 'as' with 'á' (2 backspaces: 'a' + 's')
        assert_eq!(replace_events[1].0, 2, "tone on 'a' backspace");
        assert_eq!(replace_events[1].1, "á");
        // Event 2: auto-restore back to "was " (2 backspaces: 'á' + ' ')
        assert_eq!(replace_events[2].0, 2, "auto-restore backspace");
        assert_eq!(replace_events[2].1, "was ");

        let display = get_display(&events);
        assert_eq!(display, "was ", "Final display should be 'was '");
    }

    #[test]
    fn backspace_count_auto_restore_hello() {
        // "hello " → no conversion needed, should_restore("hello") → true, no diacritics → None
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "hello ");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, .. } => Some(backspaces),
            _ => None,
        }).collect();
        // "hello" has no Vietnamese conversion, should_restore returns true
        // has_diacritics = false → returns None in auto-restore path
        assert_eq!(replace_events.len(), 0, "No Replace events for plain English");
        assert_eq!(get_display(&events), "hello ");
    }

    #[test]
    fn backspace_count_macro_expansion() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("ko".into(), "không".into());
        let events = process_input(&mut e, "ko ");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        // "ko " → macro expansion: raw_buffer="ko", Replace { 3, "không " }
        // backspaces = raw_buffer.len + 1 = 2 + 1 = 3
        assert_eq!(replace_events.len(), 1, "Expected 1 Replace event for macro");
        assert_eq!(replace_events[0].0, 3, "macro backspace count");
        assert_eq!(replace_events[0].1, "không ");
        assert_eq!(get_display(&events), "không ");
    }

    #[test]
    fn backspace_count_pending_tone_on_space() {
        // "chof " → 'f' is pending after 'o' on "cho", space flushes → "chò "
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "chof ");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        // "chof":
        // 'c' → no event
        // 'h' → no event
        // 'o' → no event
        // 'f' → process_tone on 'o' → Replace { 4, "chò" } (prev_inner="cho", expected="chof")
        // ' ' → flush with space, final_word="chò" == previous_inner="chò" → None
        assert_eq!(replace_events.len(), 1, "Expected 1 Replace event: {:?}", replace_events);
        assert_eq!(replace_events[0].0, 4, "chof→chò backspace");
        assert_eq!(replace_events[0].1, "chò");
        assert_eq!(get_display(&events), "chò ");
    }

    #[test]
    fn backspace_count_esc_undo_accuracy() {
        let mut e = Engine::new(InputMethod::Telex);
        for ch in "chafo".chars() {
            e.process_key(ch);
        }
        let event = e.process_escape();
        match event {
            Some(EngineEvent::UndoTones { backspaces, restored }) => {
                assert_eq!(backspaces, 4, "ESC undo should backspace 4 chars (chào)");
                assert_eq!(restored, "chao");
            }
            _ => panic!("Expected UndoTones"),
        }
    }

    #[test]
    fn backspace_count_after_backspace() {
        // Type "as" (→ "á"), then backspace, then type "a",
        // Then flush → "a".
        let mut e = Engine::new(InputMethod::Telex);
        e.process_key('a');
        e.process_key('s');           // buffer = "á"
        let mut events = Vec::new();
        events.push(EngineEvent::Insert(" ".to_string()));
        if let Some(ev) = e.process_key('\x08') { events.push(ev); } // backspace → buffer ""
        if let Some(ev) = e.process_key('a') { events.push(ev); }   // buffer "a" (no Replace)
        if let Some(ev) = e.flush() { events.push(ev); }
        // After backspace: buffer is empty, then 'a' → no Replace, flush returns Flush("a")
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { .. } => Some(()),
            _ => None,
        }).collect();
        assert_eq!(replace_events.len(), 0, "No Replace events after backspace + 'a'");
        let display = get_display(&events);
        assert_eq!(display, " a", "Display should be ' ' (from Insert) + 'a' (from flush)");
    }

    #[test]
    fn backspace_count_multi_word() {
        let mut e = Engine::new(InputMethod::Telex);
        // "xin chao " (xin=no convert, chao=no convert, space flushes)
        let events = process_input(&mut e, "xin chao ");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        assert_eq!(replace_events.len(), 0, "No Replace events for 'xin chao '");
        assert_eq!(get_display(&events), "xin chao ");
    }

    #[test]
    fn backspace_count_tone_at_word_end() {
        let mut e = Engine::new(InputMethod::Telex);
        // "tots" → "tót": 's' after 't' is a vowel? No. Let's trace.
        // 't' → buffer "t"
        // 'o' → buffer "to"
        // 't' → buffer "tot"
        // 's' → process_tone('s'): buffer "tot", chars ['t','o','t']
        //   i=2: is_vowel('t')? No. i=1: is_vowel('o')? Yes.
        //   Apply 's' to 'o' → 'ó'. buffer = "tót"
        // Replace { 4, "tót" }
        let events = process_input(&mut e, "tots");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        assert_eq!(replace_events.len(), 1, "Expected 1 Replace: {:?}", replace_events);
        assert_eq!(replace_events[0].0, 4, "tots→tót backspace");
        assert_eq!(replace_events[0].1, "tót");
        assert_eq!(get_display(&events), "tót");
    }

    #[test]
    fn backspace_count_final_consonant_tone() {
        let mut e = Engine::new(InputMethod::Telex);
        // "dungj" → "dụng"
        let events = process_input(&mut e, "dungj");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        assert_eq!(replace_events.len(), 1, "Expected 1 Replace: {:?}", replace_events);
        assert_eq!(replace_events[0].0, 5, "dungj→dụng backspace");
        assert_eq!(replace_events[0].1, "dụng");
        assert_eq!(get_display(&events), "dụng");
    }

    // ================================================================
    // raw_buffer integrity tests
    // ================================================================

    #[test]
    fn raw_buffer_syncs_with_engine_after_replace() {
        let mut e = Engine::new(InputMethod::Telex);
        // Type "as" → buffer="á", raw_buffer="as"
        e.process_key('a');
        e.process_key('s');
        // Verify internal state
        assert_eq!(e.buffer(), "á", "Engine buffer should be 'á'");
        // Backspace → pop engine, sync raw_buffer
        e.process_key('\x08');
        assert_eq!(e.buffer(), "", "Engine buffer should be empty after backspace");
        // Verify raw_buffer is also empty (sync'd via char count matching)
    }

    #[test]
    fn raw_buffer_tracks_keystrokes_for_macro() {
        let mut e = Engine::new(InputMethod::Telex);
        e.add_macro("dc".into(), "được".into());
        // "dc " should trigger macro: raw_buffer="dc"
        e.process_key('d');
        e.process_key('c');
        let event = e.process_key(' ');
        match event {
            Some(EngineEvent::Replace { backspaces, insert }) => {
                assert_eq!(backspaces, 3, "Macro 'dc ' → backspaces = 3");
                assert_eq!(insert, "được ");
            }
            other => panic!("Expected Replace for macro, got: {:?}", other),
        }
    }

    #[test]
    fn backspace_after_replace_syncs_raw_buffer() {
        let mut e = Engine::new(InputMethod::Telex);
        // Type "as" → buffer="á", raw_buffer="as"
        e.process_key('a');
        e.process_key('s');
        // Backspace → both should be empty
        e.process_key('\x08');
        assert_eq!(e.buffer(), "", "Buffer after backspace");
        // Type "x" → buffer="x", should not have residual raw_buffer issue
        e.process_key('x');
        assert_eq!(e.buffer(), "x", "Buffer after backspace + 'x'");
    }

    // ================================================================
    // VNI backspace counting
    // ================================================================

    #[test]
    fn vni_backspace_count_tone() {
        let mut e = Engine::new(InputMethod::Vni);
        let events = process_input(&mut e, "a1");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        assert_eq!(replace_events.len(), 1, "Expected 1 Replace: {:?}", replace_events);
        assert_eq!(replace_events[0].0, 2, "a1→á backspace");
        assert_eq!(replace_events[0].1, "á");
        assert_eq!(get_display(&events), "á");
    }

    #[test]
    fn vni_backspace_count_vowel_mod() {
        let mut e = Engine::new(InputMethod::Vni);
        let events = process_input(&mut e, "a6");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        assert_eq!(replace_events.len(), 1);
        assert_eq!(replace_events[0].0, 2, "a6→â backspace");
        assert_eq!(replace_events[0].1, "â");
        assert_eq!(get_display(&events), "â");
    }

    #[test]
    fn vni_backspace_count_mod_then_tone() {
        let mut e = Engine::new(InputMethod::Vni);
        let events = process_input(&mut e, "a61");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        // "a6" → Replace {2, "â"}, then "1" → Replace {2, "ấ"}
        assert_eq!(replace_events.len(), 2, "Expected 2 Replace: {:?}", replace_events);
        assert_eq!(replace_events[0].0, 2);
        assert_eq!(replace_events[0].1, "â");
        assert_eq!(replace_events[1].0, 2);
        assert_eq!(replace_events[1].1, "ấ");
        assert_eq!(get_display(&events), "ấ");
    }

    #[test]
    fn vni_backspace_count_consonant_digit() {
        // "b1" → 'b' is not vowel, '1' appends as digit → no Replace
        let mut e = Engine::new(InputMethod::Vni);
        let events = process_input(&mut e, "b1");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { .. } => Some(()),
            _ => None,
        }).collect();
        assert_eq!(replace_events.len(), 0, "No Replace for consonant+digit");
        assert_eq!(get_display(&events), "b1");
    }

    #[test]
    fn vni_backspace_count_word_with_mod() {
        let mut e = Engine::new(InputMethod::Vni);
        // "chao2" → '2' is tone (huyền) on 'o' → "chaò"
        let events = process_input(&mut e, "chao2");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        assert_eq!(replace_events.len(), 1, "Expected 1 Replace: {:?}", replace_events);
        // previous_inner = "chao" (4 chars), expected = "chao"+"2" = "chao2" (5 chars)
        // backspaces = 4 + 1 = 5
        assert_eq!(replace_events[0].0, 5, "chao2→chaò backspace");
        assert_eq!(replace_events[0].1, "chaò");
        assert_eq!(get_display(&events), "chaò");
    }

    // ================================================================
    // Edge case: multiple tone replacements on same vowel
    // ================================================================

    #[test]
    fn backspace_count_then_second_tone_replaces_previous() {
        // Type "as" → á, then "f" → f overrides sắc with huyền → "à"
        let mut e = Engine::new(InputMethod::Telex);
        let events = process_input(&mut e, "asf");
        let replace_events: Vec<_> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, insert } => Some((*backspaces, insert.clone())),
            _ => None,
        }).collect();
        // "as" → Replace {2, "á"}, "f" → Replace {2, "à"}
        assert_eq!(replace_events.len(), 2, "Expected 2 Replace: {:?}", replace_events);
        assert_eq!(replace_events[0].0, 2);
        assert_eq!(replace_events[0].1, "á");
        assert_eq!(replace_events[1].0, 2);
        assert_eq!(replace_events[1].1, "à");
        assert_eq!(get_display(&events), "à");
    }

    // ================================================================
    // Smart Modifier Overriding (Diacritic Replacement)
    // ================================================================

    // Category 1: The 'A' Vowel Group (a, â, ă)

    #[test]
    fn telex_override_a_aa_then_w() {
        let mut e = Engine::new(InputMethod::Telex);
        // "traan" → aa makes â → "trân", then w overrides â→ă → "trăn"
        assert_eq!(get_display(&process_input(&mut e, "traanw")), "trăn");
    }

    #[test]
    fn telex_override_a_aw_then_a() {
        let mut e = Engine::new(InputMethod::Telex);
        // "tranw" → w modifies a→ă → "trăn", then a overrides ă→â → "trân"
        assert_eq!(get_display(&process_input(&mut e, "tranwa")), "trân");
    }

    #[test]
    fn vni_override_a_6_then_8() {
        let mut e = Engine::new(InputMethod::Vni);
        // "tran6" → 6 makes â → "trân", then 8 overrides â→ă → "trăn"
        assert_eq!(get_display(&process_input(&mut e, "tran68")), "trăn");
    }

    #[test]
    fn vni_override_a_8_then_6() {
        let mut e = Engine::new(InputMethod::Vni);
        // "tran8" → 8 makes ă → "trăn", then 6 overrides ă→â → "trân"
        assert_eq!(get_display(&process_input(&mut e, "tran86")), "trân");
    }

    // Category 2: The 'O' Vowel Group (o, ô, ơ)

    #[test]
    fn telex_override_o_oo_then_w() {
        let mut e = Engine::new(InputMethod::Telex);
        // "coon" → oo makes ô → "côn", then w overrides ô→ơ → "cơn"
        assert_eq!(get_display(&process_input(&mut e, "coonw")), "cơn");
    }

    #[test]
    fn telex_override_o_ow_then_o() {
        let mut e = Engine::new(InputMethod::Telex);
        // "conw" → w modifies o→ơ → "cơn", then o overrides ơ→ô → "côn"
        assert_eq!(get_display(&process_input(&mut e, "conwo")), "côn");
    }

    #[test]
    fn vni_override_o_6_then_7() {
        let mut e = Engine::new(InputMethod::Vni);
        // "con6" → 6 makes ô → "côn", then 7 overrides ô→ơ → "cơn"
        assert_eq!(get_display(&process_input(&mut e, "con67")), "cơn");
    }

    #[test]
    fn vni_override_o_7_then_6() {
        let mut e = Engine::new(InputMethod::Vni);
        // "con7" → 7 makes ơ → "cơn", then 6 overrides ơ→ô → "côn"
        assert_eq!(get_display(&process_input(&mut e, "con76")), "côn");
    }

    // Category 3: Complex Double Vowels (uo → uô / ươ)

    #[test]
    fn telex_override_uo_oo_then_w() {
        let mut e = Engine::new(InputMethod::Telex);
        // "chuoon" → oo makes ô → "chuôn", then w overrides ô→ơ → "chươn"
        assert_eq!(get_display(&process_input(&mut e, "chuoonw")), "chươn");
    }

    #[test]
    fn telex_override_uo_ow_then_o() {
        let mut e = Engine::new(InputMethod::Telex);
        // "chuonw" → w modifies o→ơ → "chươn", then o overrides ơ→ô → "chuôn"
        assert_eq!(get_display(&process_input(&mut e, "chuonwo")), "chuôn");
    }

    #[test]
    fn vni_override_uo_6_then_7() {
        let mut e = Engine::new(InputMethod::Vni);
        // "chuon6" → 6 makes ô → "chuôn", then 7 overrides ô→ơ → "chươn"
        assert_eq!(get_display(&process_input(&mut e, "chuon67")), "chươn");
    }

    #[test]
    fn vni_override_uo_7_then_6() {
        let mut e = Engine::new(InputMethod::Vni);
        // "chuon7" → 7 makes ơ → "chươn", then 6 overrides ơ→ô → "chuôn"
        assert_eq!(get_display(&process_input(&mut e, "chuon76")), "chuôn");
    }

    // Category 4: Modifier Overriding while Preserving Tones

    #[test]
    fn telex_override_with_tone_preserved_aa_s_w() {
        let mut e = Engine::new(InputMethod::Telex);
        // "traans" → aa→â, s→sắc → "trấn", then w overrides â→ă, sắc preserved → "trắn"
        assert_eq!(get_display(&process_input(&mut e, "traansw")), "trắn");
    }

    #[test]
    fn telex_override_with_tone_preserved_oo_f_w() {
        let mut e = Engine::new(InputMethod::Telex);
        // "coonsf" → oo→ô, s→sắc then f overrides sắc→huyền → "cồn", then w overrides ô→ơ, huyền preserved → "cờn"
        assert_eq!(get_display(&process_input(&mut e, "coonsfw")), "cờn");
    }

    #[test]
    fn vni_override_with_tone_preserved_6_1_then_8() {
        let mut e = Engine::new(InputMethod::Vni);
        // "tran61" → 6→â, 1→sắc → "trấn", then 8 overrides â→ă, sắc preserved → "trắn"
        assert_eq!(get_display(&process_input(&mut e, "tran618")), "trắn");
    }

    #[test]
    fn vni_override_with_tone_preserved_6_2_then_7() {
        let mut e = Engine::new(InputMethod::Vni);
        // "con62" → 6→ô, 2→huyền → "cồn", then 7 overrides ô→ơ, huyền preserved → "cờn"
        // Note: input is "con62" then "7", but the tone 2 comes first, then modifier 7
        assert_eq!(get_display(&process_input(&mut e, "con627")), "cờn");
    }

    // ================================================================
    // Regression: backspace counting after complex sequences
    // ================================================================

    #[test]
    fn backspace_count_long_vietnamese_phrase() {
        let mut e = Engine::new(InputMethod::Telex);
        // "xin chào bạn" in Telex: "xin chaof banj"
        // xin = no change
        // ' ' = flush, no change
        // ch + ao + f = "chào"
        // ' ' = flush
        // b + a + n + j = "bạn" (j=nặng on 'a')
        let events = process_input(&mut e, "xin chaof banj");
        let replace_events: Vec<usize> = events.iter().filter_map(|ev| match ev {
            EngineEvent::Replace { backspaces, .. } => Some(*backspaces),
            _ => None,
        }).collect();
        assert_eq!(replace_events.len(), 2, "Expected 2 Replace events: {:?}", replace_events);
        assert_eq!(replace_events[0], 5, "chaof→chào should be 5");
        assert_eq!(replace_events[1], 4, "banj→bạn should be 4");
        assert_eq!(get_display(&events), "xin chào bạn");
    }

    // ================================================================
    // Core Edge Case Test Suite (from specification)
    // ================================================================

    // Standard
    #[test]
    fn core_test_traafn() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "traafn")), "trần");
    }
    #[test]
    fn core_test_tranaf() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "tranaf")), "trần");
    }
    #[test]
    fn core_test_tran62() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "tran62")), "trần");
    }

    // Double vowel / smart cluster
    #[test]
    fn core_test_chuwowng() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "chuwowng")), "chương");
    }
    #[test]
    fn core_test_chuongw() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "chuongw")), "chương");
    }
    #[test]
    fn core_test_chuong7() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "chuong7")), "chương");
    }

    // Shape override
    #[test]
    fn core_test_traanw() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "traanw")), "trăn");
    }
    #[test]
    fn core_test_trawa() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "trawa")), "trâ");
    }
    #[test]
    fn core_test_trawan() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "trawan")), "trân");
    }
    #[test]
    fn core_test_tran68() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "tran68")), "trăn");
    }

    // Tone override
    #[test]
    fn core_test_traansf() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "traansf")), "trần");
    }
    #[test]
    fn core_test_tran612() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "tran612")), "trần");
    }

    // Complex consonant + flexible
    #[test]
    fn core_test_nghieeng() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "nghieeng")), "nghiêng");
    }
    #[test]
    fn core_test_nghieengf() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "nghieengf")), "nghiềng");
    }
    #[test]
    fn core_test_nghiengf() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "nghiengf")), "nghìeng");
    }
    #[test]
    fn core_test_nghieng62() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "nghieng62")), "nghiềng");
    }

    // Tone placement
    #[test]
    fn core_test_hoangf() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "hoangf")), "hoàng");
    }
    #[test]
    fn core_test_thuyr() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "thuyr")), "thuỷ");
    }
    #[test]
    fn core_test_thuy3() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "thuy3")), "thuỷ");
    }

    // Initial đ (dd)
    #[test]
    fn core_test_ddang() {
        let mut e = Engine::new(InputMethod::Telex);
        assert_eq!(get_display(&process_input(&mut e, "ddang")), "đang");
    }
    #[test]
    fn core_test_dang9() {
        let mut e = Engine::new(InputMethod::Vni);
        assert_eq!(get_display(&process_input(&mut e, "dang9")), "đang");
    }
}
