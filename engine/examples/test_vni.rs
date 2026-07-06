use vietc_engine::{Engine, EngineEvent, InputMethod};

fn process(keys: &str) -> String {
    let mut engine = Engine::new(InputMethod::Vni);
    engine.set_enabled(true);
    
    let mut screen = String::new();
    for ch in keys.chars() {
        let buf_before = engine.buffer().chars().count();
        if let Some(event) = engine.process_key(ch) {
            match event {
                EngineEvent::Replace { backspaces, insert } => {
                    for _ in 0..backspaces { screen.pop(); }
                    screen.push_str(&insert);
                }
                EngineEvent::Insert(text) => screen.push_str(&text),
                EngineEvent::Flush(text) => screen.push_str(&text),
                _ => {}
            }
        } else {
            screen.push(ch);
        }
    }
    screen
}

fn main() {
    println!("a6 -> {:?}", process("a6"));
    println!("d9 -> {:?}", process("d9"));
    println!("e6 -> {:?}", process("e6"));
    println!("o6 -> {:?}", process("o6"));
    println!("vie61t -> {:?}", process("vie61t"));
    println!("tie6ng1 -> {:?}", process("tie6ng1"));
}
