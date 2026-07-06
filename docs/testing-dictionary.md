# Viet+ Testing Dictionary

This document defines every test scenario, its purpose, setup, expected behavior, and verification checks. Tests are grouped by suite and identified by a unique `TEST-NNN` number.

## Conventions

- **TEST-NNN**: Unique numeric identifier, never reused.
- **SCENARIO**: Human-readable one-liner describing the user action.
- **SETUP**: Preconditions (daemon mode, engine state, key state, buffer contents).
- **INPUT**: The event(s) fed into the pipeline.
- **EXPECTED**: Side effects and final state after processing.
- **CHECKS**: Specific assertions that pass or fail the test.

## Test Suites

| Suite | Prefix | Requires | Purpose |
|-------|--------|----------|---------|
| `engine` | `E-` | Nothing | VNI/Telex composition logic (pure engine) |
| `event` | `EV-` | Nothing | Daemon event routing decisions |
| `daemon` | `D-` | Root (`/dev/uinput`) | Full pipeline: evdev grab → engine → clipboard paste |
| `config` | `C-` | Nothing | Config parsing and validation |
| `regression` | `R-` | Varies | Reproducers for every past bug |

---

## Engine Suite

Test the `vietc-engine` crate's composition logic directly. No daemon, no evdev, no I/O.

### E-001: `vni_basic_vowels`
| Field | Value |
|-------|-------|
| SCENARIO | User types "a" in VNI mode |
| SETUP | engine=VNI, buffer="" |
| INPUT | key='a' |
| EXPECTED | engine.buffer = "a" |
| CHECKS | no output event generated |

### E-002: `vni_tone_2_grave`
| Field | Value |
|-------|-------|
| SCENARIO | User types "a2" (grave tone) in VNI mode |
| SETUP | engine=VNI, buffer="a" |
| INPUT | key='2' |
| EXPECTED | engine.buffer = "à", event=Replace { backspaces:1, insert:"à" } |
| CHECKS | backspace count == 1, insert == "à" |

### E-003: `vni_tone_1_acute`
| Field | Value |
|-------|-------|
| SCENARIO | User types "a1" (acute tone) in VNI mode |
| SETUP | engine=VNI, buffer="a" |
| INPUT | key='1' |
| EXPECTED | event=Replace { backspaces:1, insert:"á" } |

### E-004: `vni_tone_3_hook`
| Field | Value |
|-------|-------|
| SCENARIO | User types "a3" (hook above) in VNI |
| SETUP | engine=VNI, buffer="a" |
| INPUT | key='3' |
| EXPECTED | event=Replace { backspaces:1, insert:"ả" } |

### E-005: `vni_tone_4_tilde`
| Field | Value |
|-------|-------|
| SCENARIO | User types "a4" (tilde) in VNI |
| SETUP | engine=VNI, buffer="a" |
| INPUT | key='4' |
| EXPECTED | event=Replace { backspaces:1, insert:"ã" } |

### E-006: `vni_tone_5_dot`
| Field | Value |
|-------|-------|
| SCENARIO | User types "a5" (dot below) in VNI |
| SETUP | engine=VNI, buffer="a" |
| INPUT | key='5' |
| EXPECTED | event=Replace { backspaces:1, insert:"ạ" } |

### E-007: `vni_tone_6_grave_hook`
| Field | Value |
|-------|-------|
| SCENARIO | User types "a6" (grave+hook) in VNI |
| SETUP | engine=VNI, buffer="a" |
| INPUT | key='6' |
| EXPECTED | event=Replace { backspaces:1, insert:"ằ"} (or appropriate double-tone) |

### E-008: `telex_s_acute`
| Field | Value |
|-------|-------|
| SCENARIO | User types "as" (acute) in Telex |
| SETUP | engine=Telex, buffer="a" |
| INPUT | key='s' |
| EXPECTED | event=Replace { backspaces:1, insert:"á" } |

### E-009: `telex_f_grave`
| Field | Value |
|-------|-------|
| SCENARIO | User types "af" (grave) in Telex |
| SETUP | engine=Telex, buffer="a" |
| INPUT | key='f' |
| EXPECTED | event=Replace { backspaces:1, insert:"à" } |

### E-010: `telex_r_hook`
| Field | Value |
|-------|-------|
| SCENARIO | User types "ar" (hook) in Telex |
| SETUP | engine=Telex, buffer="a" |
| INPUT | key='r' |
| EXPECTED | event=Replace { backspaces:1, insert:"ả" } |

### E-011: `telex_x_tilde`
| Field | Value |
|-------|-------|
| SCENARIO | User types "ax" (tilde) in Telex |
| SETUP | engine=Telex, buffer="a" |
| INPUT | key='x' |
| EXPECTED | event=Replace { backspaces:1, insert:"ã" } |

### E-012: `telex_j_dot`
| Field | Value |
|-------|-------|
| SCENARIO | User types "aj" (dot below) in Telex |
| SETUP | engine=Telex, buffer="a" |
| INPUT | key='j' |
| EXPECTED | event=Replace { backspaces:1, insert:"ạ" } |

### E-013: `compound_vowel_uoi`
| Field | Value |
|-------|-------|
| SCENARIO | User types "uoi2" in VNI |
| SETUP | engine=VNI |
| INPUT | keys="u","o","i","2" |
| EXPECTED | screen = "uòi" |
| CHECKS | each key produces correct intermediate state |

### E-014: `compound_vowel_yeu`
| Field | Value |
|-------|-------|
| SCENARIO | User types "yeeus" in Telex |
| SETUP | engine=Telex |
| INPUT | keys="y","e","e","u","s" |
| EXPECTED | screen = "yếu" |

### E-015: `flushing_space`
| Field | Value |
|-------|-------|
| SCENARIO | User presses Space after a composed word |
| SETUP | engine=Telex, buffer="ngày" |
| INPUT | key=' ' |
| EXPECTED | event=Flush("ngày") |
| CHECKS | buffer is empty after flush |

### E-016: `flushing_enter`
| Field | Value |
|-------|-------|
| SCENARIO | User presses Enter after a composed word |
| SETUP | engine=VNI, buffer="thời" |
| INPUT | key='\n' |
| EXPECTED | event=Flush("thời") |

### E-017: `auto_restore_english_word`
| Field | Value |
|-------|-------|
| SCENARIO | User types an English word in VNI mode with auto-restore enabled |
| SETUP | engine=VNI, auto_restore=true |
| INPUT | keys="hello " |
| EXPECTED | screen = "hello " (raw keystrokes restored) |
| CHECKS | the word "hello" is NOT converted to VN characters |

### E-018: `auto_restore_disabled`
| Field | Value |
|-------|-------|
| SCENARIO | User types an English word with auto-restore disabled |
| SETUP | engine=VNI, auto_restore=false |
| INPUT | keys="hello " |
| EXPECTED | screen = "he lò" (VN-composed output) |

### E-019: `macros`
| Field | Value |
|-------|-------|
| SCENARIO | User types a configured macro shortcut |
| SETUP | engine=VNI, macro "qq" → "xin chào" |
| INPUT | keys="qq " |
| EXPECTED | screen = "xin chào " |

### E-020: `backspace_during_composition`
| Field | Value |
|-------|-------|
| SCENARIO | User backspaces during word composition |
| SETUP | engine=VNI, buffer="th" |
| INPUT | key='\x08' |
| EXPECTED | buffer = "t" (last char removed) |

---

## Event Routing Suite

Test the daemon's decision-making logic (which keys to forward, consume, or inject). These are pure functions — no I/O, no root needed.

### EV-001: `non_grabbed_forward_normal_char`
| Field | Value |
|-------|-------|
| SCENARIO | Non-grabbed mode, VNI enabled, user types 't' |
| SETUP | mode=non-grabbed, engine enabled, key_state={} |
| INPUT | keycode=KEY_T, value=1 |
| EXPECTED | Action=Forward (key sent to app, engine buffers 't') |
| CHECKS | `consumed_keys` is empty, engine.buffer = "t" |

### EV-002: `non_grabbed_consume_control`
| Field | Value |
|-------|-------|
| SCENARIO | Non-grabbed mode, VNI enabled, user types '2' after 'o' |
| SETUP | mode=non-grabbed, engine buffer="o" |
| INPUT | keycode=KEY_2, value=1 |
| EXPECTED | Action=Inject([Backspace(2), Type("ò")]) |
| CHECKS | backspace count is 2 (1 for 'o' + 1 for '2' that reached app) |

### EV-003: `non_grabbed_skip_engine_disabled`
| Field | Value |
|-------|-------|
| SCENARIO | Non-grabbed mode, engine disabled, user types 't' |
| SETUP | mode=non-grabbed, engine disabled |
| INPUT | keycode=KEY_T, value=1 |
| EXPECTED | Action=Skip (key reaches app directly, daemon does nothing) |

### EV-004: `grabbed_forward_normal_char`
| Field | Value |
|-------|-------|
| SCENARIO | Grabbed mode, VNI enabled, user types 't' |
| SETUP | mode=grabbed, engine enabled, consumed_keys={} |
| INPUT | keycode=KEY_T, value=1 |
| EXPECTED | Action=Forward(injector.send_key_event(KEY_T, 1)), engine.buffer="t" |
| CHECKS | consumed_keys is empty, engine.buffer = "t" |

### EV-005: `grabbed_consume_control`
| Field | Value |
|-------|-------|
| SCENARIO | Grabbed mode, VNI enabled, user types '2' after 'o' |
| SETUP | mode=grabbed, engine buffer="o", consumed_keys={} |
| INPUT | keycode=KEY_2, value=1 |
| EXPECTED | Action=Inject([Backspace(1), Type("ò")]) |
| CHECKS | keycode is in consumed_keys, engine.buffer = "ò" |

### EV-006: `grabbed_modifier_passthrough`
| Field | Value |
|-------|-------|
| SCENARIO | Grabbed mode, Ctrl held, user presses 'c' |
| SETUP | mode=grabbed, key_state contains KEY_LEFTCTRL |
| INPUT | keycode=KEY_C, value=1 |
| EXPECTED | Action=Forward (Ctrl+C reaches app for copy) |
| CHECKS | injector.send_key_event(KEY_C, 1) called |

### EV-007: `grabbed_engine_disabled_forward`
| Field | Value |
|-------|-------|
| SCENARIO | Grabbed mode, engine disabled, user types 't' |
| SETUP | mode=grabbed, engine disabled |
| INPUT | keycode=KEY_T, value=1 |
| EXPECTED | Action=Forward(injector.send_key_event(KEY_T, 1)) |
| CHECKS | engine.process_key was NOT called, key forwarded directly |

### EV-008: `grabbed_backspace_engine`
| Field | Value |
|-------|-------|
| SCENARIO | Grabbed mode, VNI enabled, user presses Backspace |
| SETUP | mode=grabbed, engine buffer="th" |
| INPUT | keycode=KEY_BACKSPACE, value=1 |
| EXPECTED | engine.process_key('\x08') called, injector forwards backspace |
| CHECKS | engine.buffer = "t" |

### EV-009: `grabbed_release_consumed`
| Field | Value |
|-------|-------|
| SCENARIO | Grabbed mode, release of a consumed key |
| SETUP | mode=grabbed, consumed_keys contains keycode for '2' |
| INPUT | keycode=KEY_2, value=0 |
| EXPECTED | Action=Skip (release not forwarded to app) |
| CHECKS | keycode removed from consumed_keys |

### EV-010: `grabbed_release_unconsumed`
| Field | Value |
|-------|-------|
| SCENARIO | Grabbed mode, release of an unconsumed key |
| SETUP | mode=grabbed, consumed_keys={} |
| INPUT | keycode=KEY_T, value=0 |
| EXPECTED | Action=Forward(injector.send_key_event(KEY_T, 0)) |

### EV-011: `non_primary_device_skipped`
| Field | Value |
|-------|-------|
| SCENARIO | Grabbed mode, event from non-primary device (i=1) |
| SETUP | mode=grabbed, device index=1 |
| INPUT | keycode=KEY_A, value=1 |
| EXPECTED | Action=Skip (event passes through to app directly) |

### EV-012: `toggle_combo_ctrl_space`
| Field | Value |
|-------|-------|
| SCENARIO | User presses Ctrl+Space |
| SETUP | key_state contains KEY_LEFTCTRL, KEY_SPACE pressed |
| INPUT | keycode=KEY_SPACE, value=1 |
| EXPECTED | Action=ToggleEngine |
| CHECKS | daemon.toggle() called, engine state flipped |

### EV-013: `method_toggle_ctrl_leftshift`
| Field | Value |
|-------|-------|
| SCENARIO | User presses Ctrl+LeftShift |
| SETUP | key_state contains KEY_LEFTCTRL, KEY_LEFTSHIFT pressed |
| INPUT | keycode=KEY_LEFTSHIFT, value=1 |
| EXPECTED | Action=ToggleMethod |
| CHECKS | input method switches VNI↔Telex |

### EV-014: `password_field_disables_engine`
| Field | Value |
|-------|-------|
| SCENARIO | User focuses a password field |
| SETUP | mode=grabbed, engine enabled, app_state reports password |
| INPUT | keycode=KEY_A, value=1 |
| EXPECTED | engine disabled, buffer reset |

---

## Daemon Suite

Full pipeline tests that require root (`/dev/uinput`) for creating virtual keyboard devices. These tests spawn the daemon as a subprocess, send real keystrokes via a uinput virtual device, and verify output via clipboard.

### D-001: `grab_acquire_and_hold`
| Field | Value |
|-------|-------|
| SCENARIO | Daemon grabs a virtual keyboard and holds the grab |
| SETUP | Virtual keyboard created, daemon configured with grab=true |
| INPUT | (none) |
| EXPECTED | Daemon logs "Keyboard grabbed", grab persists for 10+ seconds |
| CHECKS | No "releasing grab" log message within 10 seconds |

### D-002: `vni_simple_word_paste`
| Field | Value |
|-------|-------|
| SCENARIO | User types "tho2i " in VNI grabbed mode |
| SETUP | Virtual keyboard created, daemon in grabbed VNI mode |
| INPUT | keys="tho2i " sent via virtual keyboard |
| EXPECTED | Clipboard contains "thời " after processing |
| CHECKS | clipboard content matches expected output |

### D-003: `telex_simple_word_paste`
| Field | Value |
|-------|-------|
| SCENARIO | User types "thoifw" in Telex grabbed mode |
| SETUP | Virtual keyboard created, daemon in grabbed Telex mode |
| INPUT | keys="thoifw" sent via virtual keyboard |
| EXPECTED | Clipboard contains "thổi" after processing |

### D-004: `english_mode_no_paste`
| Field | Value |
|-------|-------|
| SCENARIO | Engine disabled (EN mode), user types "hello" |
| SETUP | Virtual keyboard created, daemon in grabbed EN mode |
| INPUT | keys="hello " sent via virtual keyboard |
| EXPECTED | No clipboard paste operations, characters forwarded directly |

### D-005: `mouse_keyboard_double_input`
| Field | Value |
|-------|-------|
| SCENARIO | Two virtual keyboards, only primary grabbed, same keystroke on both |
| SETUP | Two virtual keyboards created, daemon in grabbed VNI mode |
| INPUT | KEY_A on both keyboards |
| EXPECTED | 'a' appears only once (no double-input) |

---

## Regression Suite

Every bug that has been fixed in Viet+ history, preserved as a test.

### R-001: `no_grab_release_on_idle`
| Field | Value |
|-------|-------|
| BUG | Grab released after 300ms idle before user started typing |
| FIX | Removed idle-timeout grab-release |
| SETUP | Virtual keyboard created, daemon grabs it |
| INPUT | No input for 10 seconds |
| EXPECTED | Grab still held, no "releasing grab" log |
| CHECKS | Daemon responds to keystroke after 10s idle |

### R-002: `clear_modifiers_after_paste`
| Field | Value |
|-------|-------|
| BUG | Ctrl+V paste in Chrome left Ctrl stuck → next char became Ctrl+<char> |
| FIX | `clear_modifiers()` releases all 8 modifier keys after every paste |
| SETUP | Virtual keyboard created, daemon in grabbed VNI mode |
| INPUT | Type "tho2i " then immediately type 'a' |
| EXPECTED | 'a' typed as normal character, not Ctrl+A |

### R-003: `non_grabbed_backspace_plus_one`
| Field | Value |
|-------|-------|
| BUG | VNI control key '2' reached app directly in non-grabbed mode, daemon only backspaced once (removed vowel but left '2') |
| FIX | Non-grabbed path adds 1 extra backspace for VNI control keys |
| SETUP | Virtual keyboard created, daemon in non-grabbed VNI mode |
| INPUT | Type "tho2i " |
| EXPECTED | No '2' character remains on screen |

### R-004: `no_double_input_nonprimary`
| Field | Value |
|-------|-------|
| BUG | Non-primary devices processed through engine in grabbed mode → double keystroke |
| FIX | `if i != 0 { continue; }` skips engine processing for non-primary devices |
| SETUP | Two virtual keyboards, one grabbed |
| INPUT | Type 'a' on non-primary keyboard |
| EXPECTED | 'a' appears exactly once (direct pass-through, no engine processing) |

### R-005: `grabbed_engine_disabled`
| Field | Value |
|-------|-------|
| BUG | Grabbed path lacked engine-disabled check → all keys consumed and paste-injected even in EN mode |
| FIX | Added `if !daemon.engine.is_enabled() { forward; continue; }` in grabbed path |
| SETUP | Virtual keyboard created, daemon in grabbed EN mode |
| INPUT | Type "hello" |
| EXPECTED | Characters forwarded directly, no clipboard paste |

### R-006: `chrome_ctrl_v_stuck`
| Field | Value |
|-------|-------|
| BUG | Chrome detected stuck Ctrl after clipboard paste → next character sent as Ctrl+<char> |
| FIX | `send_ctrl_v()` calls `clear_modifiers()` after Ctrl+V to force-release all modifiers |
| SETUP | Virtual keyboard created, daemon in grabbed VNI mode |
| INPUT | Type 10 VNI words rapidly |
| EXPECTED | No stuck modifier keys, all words typed correctly |

---

## Running Tests

```bash
# All tests (unit + integration, non-root tests only)
cargo test

# Engine-specific tests
cargo test -p vietc-engine

# Daemon unit tests only (no root needed)
cargo test -p vietc-daemon -- --skip daemon_suite

# Full daemon test suite (including root-requiring integration tests)
sudo cargo test -p vietc-daemon

# Run a specific test by name
sudo cargo test -p vietc-daemon vni_simple_word_paste -- --nocapture

# Run regression tests only
sudo cargo test -p vietc-daemon regression_ -- --nocapture
```

## Adding a New Test

1. **Pick the next TEST-NNN** in the appropriate suite.
2. **Document it** in this dictionary with full SCENARIO, SETUP, INPUT, EXPECTED, CHECKS.
3. **Write the code** — pure logic tests go in `daemon/src/event.rs`, config tests in `daemon/src/config.rs`, pipeline tests in `daemon/tests/`.
4. **Verify** — run `cargo test` and confirm the new test passes.
5. **Cross-distro** — if the test depends on a distro-specific backend, add the backend module before the test.
