use std::io::{self, Write};
use vietc_engine::{Engine, EngineEvent, EventStore, InputEvent, InputMethod};

struct CliState {
    engine: Engine,
    method: InputMethod,
    events: EventStore,
    macros: Vec<(String, String)>,
    auto_restore: bool,
}

impl CliState {
    fn new() -> Self {
        Self {
            engine: Engine::new(InputMethod::Telex),
            method: InputMethod::Telex,
            events: EventStore::new(),
            macros: Vec::new(),
            auto_restore: true,
        }
    }

    fn set_method(&mut self, method: InputMethod) {
        self.method = method;
        self.engine.set_method(method);
    }

    fn status(&self) {
        println!("  Method: {:?}", self.method);
        println!("  Enabled: {}", self.engine.is_enabled());
        println!("  Auto-restore: {}", self.auto_restore);
        println!("  Buffer: {:?}", self.engine.buffer());
        println!("  Macros: {} defined", self.macros.len());
        for (s, e) in &self.macros {
            println!("    {} -> {}", s, e);
        }
        println!("  Events: {} recorded", self.events.len());
    }
}

fn main() {
    let mut state = CliState::new();

    print_help();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            continue;
        }

        if input == "quit" || input == "exit" {
            break;
        }

        if input.starts_with(':') {
            handle_command(&mut state, input);
            continue;
        }

        state.engine.reset();

        let mut output = String::new();
        let mut events = Vec::new();

        for ch in input.chars() {
            state.events.push(InputEvent::KeyTyped(ch));

            match state.engine.process_key(ch) {
                None => {
                    output.push(ch);
                }
                Some(event) => {
                    events.push((ch, event.clone()));
                    match &event {
                        EngineEvent::Insert(text) | EngineEvent::Flush(text) => {
                            output.push_str(text);
                        }
                        EngineEvent::Paste(text) => {
                            output.push_str(text);
                        }
                        EngineEvent::Replace { backspaces, insert } => {
                            for _ in 0..*backspaces {
                                output.push('\x08');
                            }
                            output.push_str(insert);
                            if is_flush_char(ch) {
                                output.push(ch);
                            }
                        }
                        EngineEvent::UndoTones { backspaces, restored } => {
                            for _ in 0..*backspaces {
                                output.push('\x08');
                            }
                            output.push_str(restored);
                        }
                        EngineEvent::AutoRestore(word) => {
                            for _ in 0..word.len() {
                                output.push('\x08');
                            }
                            output.push_str(word);
                        }
                    }
                }
            }
        }

        if let Some(event) = state.engine.flush() {
            match &event {
                EngineEvent::Flush(text) | EngineEvent::Insert(text) => {
                    output.push_str(text);
                }
                _ => {}
            }
            events.push(('\n', event));
        }

        println!("  Events: {:?}", events);
        println!("  Raw: {:?}", output);

        let display = apply_backspaces(&output);
        println!("  Screen: {}", display);
    }
}

fn print_help() {
    println!("Viet+ IME - Test Harness");
    println!("=========================");
    println!("Type text with VNI/Telex to see engine output.");
    println!();
    println!("Commands:");
    println!("  :help              Show this help");
    println!("  :status            Show engine state");
    println!("  :vi                Enable Vietnamese mode");
    println!("  :en                Disable Vietnamese mode");
    println!("  :ar on|off         Toggle auto-restore");
    println!("  :vni               Switch to VNI input");
    println!("  :telex             Switch to Telex input");
    println!("  :reset             Reset engine buffer");
    println!("  :buffer            Show composing buffer");
    println!("  :events            Show event store history");
    println!("  :events clear      Clear event store");
    println!("  :macros            List macros");
    println!("  :macro add <s> <e> Add macro shortcut->expansion");
    println!("  :macro rm <s>      Remove a macro");
    println!("  :macro clear       Clear all macros");
    println!("  quit/exit          Quit");
    println!();
}

fn handle_command(state: &mut CliState, input: &str) {
    let parts: Vec<&str> = input.splitn(4, ' ').collect();
    let cmd = parts[0];

    match cmd {
        ":help" | ":h" => print_help(),

        ":status" | ":st" => state.status(),

        ":vi" => {
            state.engine.set_enabled(true);
            println!("[Vietnamese mode ON]");
        }

        ":en" => {
            state.engine.set_enabled(false);
            println!("[Vietnamese mode OFF]");
        }

        ":ar" => {
            if parts.len() < 2 {
                println!("[Usage: :ar on|off]");
                return;
            }
            match parts[1] {
                "on" => {
                    state.auto_restore = true;
                    state.engine.set_auto_restore(true);
                    println!("[Auto-restore ON]");
                }
                "off" => {
                    state.auto_restore = false;
                    state.engine.set_auto_restore(false);
                    println!("[Auto-restore OFF]");
                }
                _ => println!("[Usage: :ar on|off]"),
            }
        }

        ":vni" => {
            state.set_method(InputMethod::Vni);
            println!("[Switched to VNI]");
        }

        ":telex" => {
            state.set_method(InputMethod::Telex);
            println!("[Switched to Telex]");
        }

        ":reset" => {
            state.engine.reset();
            println!("[Engine reset]");
        }

        ":buffer" => {
            println!("[Buffer: {:?}]", state.engine.buffer());
        }

        ":events" | ":ev" => {
            if parts.len() > 1 && parts[1] == "clear" {
                state.events.clear();
                println!("[Event store cleared]");
                return;
            }
            if state.events.is_empty() {
                println!("[No events]");
            } else {
                println!("[Events: {}]", state.events.len());
                for (i, event) in state.events.iter().enumerate() {
                    println!("  {}: {:?}", i, event);
                }
                println!("  Raw keystrokes: {:?}", state.events.raw_keystrokes());
                println!("  Pattern hash: {}", state.events.pattern_hash());
            }
        }

        ":macros" => {
            if state.macros.is_empty() {
                println!("[No macros defined]");
            } else {
                println!("[Macros: {}]", state.macros.len());
                for (s, e) in &state.macros {
                    println!("  {} -> {}", s, e);
                }
            }
        }

        ":macro" => {
            if parts.len() < 2 {
                println!("[Usage: :macro add <shortcut> <expansion> or :macro rm <s> or :macro clear]");
                return;
            }
            match parts[1] {
                "add" | "a" => {
                    if parts.len() < 4 {
                        println!("[Usage: :macro add <shortcut> <expansion>]");
                        return;
                    }
                    let shortcut = parts[2].to_string();
                    let expansion = parts[3].to_string();
                    state.engine.add_macro(shortcut.clone(), expansion.clone());
                    if let Some(pos) = state.macros.iter().position(|(s, _)| *s == shortcut) {
                        state.macros[pos].1 = expansion.clone();
                    } else {
                        state.macros.push((shortcut.clone(), expansion.clone()));
                    }
                    println!("[Macro added: {} -> {}]", shortcut, expansion);
                }
                "rm" | "remove" | "del" => {
                    if parts.len() < 3 {
                        println!("[Usage: :macro rm <shortcut>]");
                        return;
                    }
                    let shortcut = parts[2];
                    if let Some(pos) = state.macros.iter().position(|(s, _)| s == shortcut) {
                        state.macros.remove(pos);
                        state.engine.clear_macros();
                        for (s, e) in &state.macros {
                            state.engine.add_macro(s.clone(), e.clone());
                        }
                        println!("[Macro removed: {}]", shortcut);
                    } else {
                        println!("[Macro not found: {}]", shortcut);
                    }
                }
                "clear" | "c" => {
                    state.engine.clear_macros();
                    state.macros.clear();
                    println!("[All macros cleared]");
                }
                _ => {
                    let shortcut = parts[1].to_string();
                    let expansion = parts.get(2).map(|s| s.to_string()).unwrap_or_default();
                    if expansion.is_empty() {
                        println!("[Usage: :macro add <shortcut> <expansion>]");
                        return;
                    }
                    state.engine.add_macro(shortcut.clone(), expansion.clone());
                    if let Some(pos) = state.macros.iter().position(|(s, _)| *s == shortcut) {
                        state.macros[pos].1 = expansion.clone();
                    } else {
                        state.macros.push((shortcut.clone(), expansion.clone()));
                    }
                    println!("[Macro added: {} -> {}]", shortcut, expansion);
                }
            }
        }

        _ => {
            println!("[Unknown command: {}. Type :help for available commands]", cmd);
        }
    }
}

fn is_flush_char(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '.' | ',' | '!' | '?' | ';' | ':' | '\n')
}

fn apply_backspaces(s: &str) -> String {
    let mut result = String::new();
    for ch in s.chars() {
        if ch == '\x08' {
            result.pop();
        } else {
            result.push(ch);
        }
    }
    result
}
