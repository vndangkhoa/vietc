use std::io::{self, Write};
use vietc_engine::{Engine, EngineEvent, InputMethod};

fn get_display(events: &[EngineEvent]) -> String {
    let mut display = String::new();
    for ev in events {
        match ev {
            EngineEvent::Flush(text) => { if !display.ends_with(text) { display.push_str(text); } }
            EngineEvent::Insert(text) => display.push_str(text),
            EngineEvent::Replace { backspaces, insert } => {
                for _ in 0..*backspaces { display.pop(); }
                display.push_str(insert);
            }
            EngineEvent::AutoRestore(word) => {
                for _ in 0..word.len() { display.pop(); }
                display.push_str(word);
            }
            EngineEvent::UndoTones { backspaces, restored } => {
                for _ in 0..*backspaces { display.pop(); }
                display.push_str(restored);
            }
        }
    }
    display
}

fn process_input(e: &mut Engine, input: &str) -> Vec<EngineEvent> {
    let mut events = Vec::new();
    for ch in input.chars() {
        if let Some(ev) = e.process_key(ch) { events.push(ev); }
    }
    events
}

const INITIALS: &[&str] = &[
    "", "b", "c", "ch", "d", "g", "gh", "h", "k", "kh", "l", "m", "n",
    "ng", "ngh", "nh", "p", "ph", "q", "r", "s", "t", "th", "tr", "v", "x",
];

const FINALS: &[&str] = &["", "c", "ch", "m", "n", "ng", "nh", "p", "t"];

fn is_valid(init: &str, fin: &str) -> bool {
    if init == "ngh" && !fin.is_empty() && fin != "n" && fin != "ng" && fin != "nh" { return false; }
    if init == "gh" && !fin.is_empty() { return false; }
    if init == "q" { return false; }
    if init == "g" && !fin.is_empty() && fin != "n" && fin != "ng" { return false; }
    if fin == "ch" && init == "" { return false; }
    if fin == "nh" && init == "" { return false; }
    true
}

fn main() {
    // Telex base vowels (as typed, before mod)
    let telex_vowels: Vec<(&str, &str)> = vec![
        ("a", "af"), ("a", "as"), ("a", "aj"), ("a", "ar"), ("a", "ax"),
        ("a", "aw"), ("a", "aa"),
        ("e", "ee"),
        ("o", "oo"), ("o", "ow"),
        ("u", "uw"),
    ];

    let mut count = 0;
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    for &init in INITIALS {
        for &fin in FINALS {
            if !is_valid(init, fin) { continue; }
            for &(base, mod_str) in &telex_vowels {
                let plain = format!("{}{}{}", init, base, fin);
                let full = format!("{}{}", plain, mod_str);
                if plain.len() > 10 { continue; }

                let mut e = Engine::new(InputMethod::Telex);
                let result = get_display(&process_input(&mut e, &full));

                if !result.is_empty() && result.len() <= 12 && result != full && result != plain {
                    count += 1;
                    let _ = writeln!(handle, "{{\"i\":\"{full}\",\"e\":\"{result}\",\"m\":\"telex\"}}");
                }
                if count >= 1000 { break; }
            }
            if count >= 1000 { break; }
        }
        if count >= 1000 { break; }
    }

    eprintln!("Generated {count} test cases");
}
