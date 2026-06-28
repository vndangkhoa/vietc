// SPDX-License-Identifier: MIT
use std::io::{self, Write};
use vietc_engine::{Engine, EngineEvent, InputMethod};

fn main() {
    let mut engine = Engine::new(InputMethod::Telex);

    println!("Viet+ IME - Test Harness");
    println!("==========================");
    println!("Type Vietnamese using Telex input.");
    println!("Press Enter to flush, type 'quit' to exit.");
    println!("Toggle method with ':vni' or ':telex'");
    println!();

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input == "quit" || input == "exit" {
            break;
        }

        if input == ":vni" {
            engine.set_method(InputMethod::Vni);
            println!("[Switched to VNI]");
            continue;
        }

        if input == ":telex" {
            engine.set_method(InputMethod::Telex);
            println!("[Switched to Telex]");
            continue;
        }

        if input == ":reset" {
            engine.reset();
            println!("[Engine reset]");
            continue;
        }

        if input == ":buffer" {
            println!("[Buffer: {:?}]", engine.buffer());
            continue;
        }

        let mut output = String::new();
        let mut events = Vec::new();

        for ch in input.chars() {
            if let Some(event) = engine.process_key(ch) {
                events.push((ch, event.clone()));
                match &event {
                    EngineEvent::Flush(text) => {
                        output.push_str(text);
                    }
                    EngineEvent::Insert(text) => {
                        output.push_str(text);
                    }
                    EngineEvent::AutoRestore(word) => {
                        // Auto-restore: delete the word and re-insert it
                        for _ in 0..word.len() {
                            output.push('\x08'); // backspace
                        }
                        output.push_str(word);
                    }
                    EngineEvent::Replace { backspaces, insert } => {
                        for _ in 0..*backspaces {
                            output.push('\x08');
                        }
                        output.push_str(insert);
                    }
                    EngineEvent::UndoTones {
                        backspaces,
                        restored,
                    } => {
                        for _ in 0..*backspaces {
                            output.push('\x08');
                        }
                        output.push_str(restored);
                    }
                    EngineEvent::Paste(text) => {
                        output.push_str(text);
                    }
                }
            }
        }

        // Flush remaining buffer
        if let Some(event) = engine.flush() {
            match &event {
                EngineEvent::Flush(text) => {
                    output.push_str(text);
                }
                EngineEvent::Insert(text) => {
                    output.push_str(text);
                }
                _ => {}
            }
            events.push(('\n', event));
        }

        println!("  Events: {:?}", events);
        println!("  Output: {:?}", output);

        // Show what it would look like
        let display: String = output.chars().filter(|c| *c != '\x08').collect();
        println!("  Display: {}", display);
    }
}
