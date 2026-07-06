/// Characters that flush the current word and start a new one.
pub fn is_flush_char(ch: char) -> bool {
    matches!(ch, ' ' | '.' | ',' | '!' | '?' | ';' | ':' | '\t' | '\n')
}

pub fn is_vn_control_key(method: &str, ch: char) -> bool {
    match method {
        "telex" => matches!(ch.to_ascii_lowercase(), 'f' | 's' | 'r' | 'x' | 'j' | 'w'),
        "vni" => matches!(ch, '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '0'),
        _ => false,
    }
}

pub fn is_modifier_pressed(key_state: &evdev::AttributeSet<evdev::Key>) -> bool {
    key_state.contains(evdev::Key::KEY_LEFTCTRL)
        || key_state.contains(evdev::Key::KEY_RIGHTCTRL)
        || key_state.contains(evdev::Key::KEY_LEFTALT)
        || key_state.contains(evdev::Key::KEY_RIGHTALT)
        || key_state.contains(evdev::Key::KEY_LEFTMETA)
        || key_state.contains(evdev::Key::KEY_RIGHTMETA)
}

pub fn is_modifier_held_shift(key_state: &evdev::AttributeSet<evdev::Key>) -> bool {
    key_state.contains(evdev::Key::KEY_LEFTSHIFT) || key_state.contains(evdev::Key::KEY_RIGHTSHIFT)
}

pub fn is_caps_lock_on(device: &evdev::Device) -> bool {
    if let Ok(leds) = device.get_led_state() {
        leds.contains(evdev::LedType::LED_CAPSL)
    } else {
        false
    }
}

pub fn is_method_toggle_state(key_state: &evdev::AttributeSet<evdev::Key>) -> bool {
    let ctrl_pressed = key_state.contains(evdev::Key::KEY_LEFTCTRL)
        || key_state.contains(evdev::Key::KEY_RIGHTCTRL);
    let shift_pressed = key_state.contains(evdev::Key::KEY_LEFTSHIFT);
    ctrl_pressed && shift_pressed
        && !key_state.contains(evdev::Key::KEY_RIGHTSHIFT)
        && !key_state.contains(evdev::Key::KEY_LEFTALT)
        && !key_state.contains(evdev::Key::KEY_RIGHTALT)
        && !key_state.contains(evdev::Key::KEY_LEFTMETA)
        && !key_state.contains(evdev::Key::KEY_RIGHTMETA)
}

pub fn is_toggle_combination_state(key_state: &evdev::AttributeSet<evdev::Key>, key: &str) -> bool {
    let ctrl_pressed = key_state.contains(evdev::Key::KEY_LEFTCTRL)
        || key_state.contains(evdev::Key::KEY_RIGHTCTRL);

    if !ctrl_pressed {
        return false;
    }

    let target = match key.to_lowercase().as_str() {
        "space" => evdev::Key::KEY_SPACE,
        "shift" => evdev::Key::KEY_LEFTSHIFT,
        "capslock" => evdev::Key::KEY_CAPSLOCK,
        "ctrl" => evdev::Key::KEY_LEFTCTRL,
        "alt" => evdev::Key::KEY_LEFTALT,
        _ => return false,
    };

    key_state.contains(target)
}

pub fn key_to_char(key: evdev::Key) -> Option<char> {
    match key {
        evdev::Key::KEY_A => Some('a'),
        evdev::Key::KEY_B => Some('b'),
        evdev::Key::KEY_C => Some('c'),
        evdev::Key::KEY_D => Some('d'),
        evdev::Key::KEY_E => Some('e'),
        evdev::Key::KEY_F => Some('f'),
        evdev::Key::KEY_G => Some('g'),
        evdev::Key::KEY_H => Some('h'),
        evdev::Key::KEY_I => Some('i'),
        evdev::Key::KEY_J => Some('j'),
        evdev::Key::KEY_K => Some('k'),
        evdev::Key::KEY_L => Some('l'),
        evdev::Key::KEY_M => Some('m'),
        evdev::Key::KEY_N => Some('n'),
        evdev::Key::KEY_O => Some('o'),
        evdev::Key::KEY_P => Some('p'),
        evdev::Key::KEY_Q => Some('q'),
        evdev::Key::KEY_R => Some('r'),
        evdev::Key::KEY_S => Some('s'),
        evdev::Key::KEY_T => Some('t'),
        evdev::Key::KEY_U => Some('u'),
        evdev::Key::KEY_V => Some('v'),
        evdev::Key::KEY_W => Some('w'),
        evdev::Key::KEY_X => Some('x'),
        evdev::Key::KEY_Y => Some('y'),
        evdev::Key::KEY_Z => Some('z'),
        evdev::Key::KEY_0 => Some('0'),
        evdev::Key::KEY_1 => Some('1'),
        evdev::Key::KEY_2 => Some('2'),
        evdev::Key::KEY_3 => Some('3'),
        evdev::Key::KEY_4 => Some('4'),
        evdev::Key::KEY_5 => Some('5'),
        evdev::Key::KEY_6 => Some('6'),
        evdev::Key::KEY_7 => Some('7'),
        evdev::Key::KEY_8 => Some('8'),
        evdev::Key::KEY_9 => Some('9'),
        evdev::Key::KEY_SPACE => Some(' '),
        evdev::Key::KEY_DOT => Some('.'),
        evdev::Key::KEY_COMMA => Some(','),
        evdev::Key::KEY_MINUS => Some('-'),
        evdev::Key::KEY_EQUAL => Some('='),
        evdev::Key::KEY_SEMICOLON => Some(';'),
        evdev::Key::KEY_APOSTROPHE => Some('\''),
        evdev::Key::KEY_SLASH => Some('/'),
        evdev::Key::KEY_BACKSPACE => Some('\x08'),
        evdev::Key::KEY_ENTER => Some('\n'),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::OutputCommand;
    use vietc_engine::{Engine, EngineEvent, InputMethod};

    fn event_to_commands(event: Option<EngineEvent>) -> Vec<OutputCommand> {
        let mut commands = Vec::new();
        if let Some(event) = event {
            match event {
                EngineEvent::Flush(text) | EngineEvent::Insert(text) | EngineEvent::Paste(text) => {
                    commands.push(OutputCommand::Type(text));
                }
                EngineEvent::AutoRestore(word) => {
                    commands.push(OutputCommand::Backspace(word.chars().count()));
                    commands.push(OutputCommand::Type(word));
                }
                EngineEvent::Replace { backspaces, insert } => {
                    commands.push(OutputCommand::Backspace(backspaces));
                    commands.push(OutputCommand::Type(insert));
                }
                EngineEvent::UndoTones { backspaces, restored } => {
                    commands.push(OutputCommand::Backspace(backspaces));
                    commands.push(OutputCommand::Type(restored));
                }
            }
        }
        commands
    }

    fn render(method_str: &str, keys: &str) -> String {
        let method = match method_str {
            "vni" => InputMethod::Vni,
            _ => InputMethod::Telex,
        };
        let mut engine = Engine::new(method);
        engine.set_enabled(true);
        engine.set_auto_restore(true);

        let mut screen: Vec<char> = Vec::new();
        for ch in keys.chars() {
            let buf_before = engine.buffer().chars().count();
            let commands = event_to_commands(engine.process_key(ch));
            if !commands.is_empty() {
                for cmd in &commands {
                    match cmd {
                        OutputCommand::Backspace(n) => {
                            for _ in 0..*n {
                                screen.pop();
                            }
                        }
                        OutputCommand::Type(text) => screen.extend(text.chars()),
                    }
                }
                if is_flush_char(ch) {
                    screen.push(ch);
                }
            } else if is_vn_control_key(method_str, ch)
                && engine.buffer().chars().count() <= buf_before
            {
            } else {
                screen.push(ch);
            }
        }
        screen.into_iter().collect()
    }

    #[test]
    fn leading_control_letters_are_kept() {
        assert_eq!(render("telex", "xuaw"), "xưa");
        assert_eq!(render("telex", "trong"), "trong");
        assert_eq!(render("telex", "ruwngf"), "rừng");
    }

    #[test]
    fn spaces_between_words_are_preserved() {
        assert_eq!(render("telex", "Ngayf xuaw"), "Ngày xưa");
        assert_eq!(render("telex", "khu ruwngf raamj"), "khu rừng rậm");
        assert_eq!(render("telex", "con Voi raats"), "con Voi rất");
    }

    #[test]
    fn full_sentence_renders_correctly() {
        let keys = "Ngayf xuaw, trong mootj khu ruwngf raamj cos mootj con Voi raats hung duwx.";
        let expected = "Ngày xưa, trong một khu rừng rậm có một con Voi rất hung dữ.";
        assert_eq!(render("telex", keys), expected);
    }
}
