use vietc_engine::{Engine, EngineEvent, InputMethod};

fn trace(input: &str, method: InputMethod) {
    let mut e = Engine::new(method);
    eprintln!("\n=== {:?}: {} ===", method, input);
    eprintln!("Ch | prev_buf  → new_buf   | expected_screen     | Event");
    eprintln!("---+-----------+-----------+---------------------+------");
    for ch in input.chars() {
        let prev = e.buffer().to_string();
        let event = e.process_key(ch);
        let curr = e.buffer().to_string();
        let expected = format!("{}{}", prev, ch);
        let event_str = match &event {
            Some(EngineEvent::Replace { backspaces, insert }) => {
                format!("Replace({}, {:?})", backspaces, insert)
            }
            Some(EngineEvent::Insert(t)) => format!("Insert({:?})", t),
            Some(EngineEvent::Flush(t)) => format!("Flush({:?})", t),
            Some(EngineEvent::AutoRestore(w)) => format!("AutoRestore({:?})", w),
            Some(EngineEvent::UndoTones {
                backspaces,
                restored,
            }) => format!("UndoTones({}, {:?})", backspaces, restored),
            Some(EngineEvent::Paste(t)) => format!("Paste({:?})", t),
            None => "None".to_string(),
        };
        eprintln!(
            "'{}' | {:<9} → {:<9} | {:<19} | {}",
            ch, prev, curr, expected, event_str
        );
        if let Some(EngineEvent::Replace { backspaces, insert }) = &event {
            // In grab mode, backspace - 1 (key consumed)
            let grab_bs = backspaces.saturating_sub(1);
            // In non-grab mode, full backspace
            eprintln!(
                "    |           |           | grab_bs={} non_grab_bs={} insert={:?}",
                grab_bs, backspaces, insert
            );
        }
    }
    // Flush
    if let Some(event) = e.flush() {
        eprintln!(
            "FL  |           |           |                     | {:?}",
            event
        );
    }
}

fn main() {
    // Category 1: Basic A group
    trace("traan", InputMethod::Telex); // trâ
    trace("traanw", InputMethod::Telex); // trân → w → trăn
    trace("tranwa", InputMethod::Telex); // trăn → a → trân

    // Category 2: Basic O group
    trace("coon", InputMethod::Telex); // côn
    trace("coonw", InputMethod::Telex); // côn → w → cơn
    trace("conwo", InputMethod::Telex); // cơn → o → côn

    // Category 3: Smart cluster
    trace("chuoonw", InputMethod::Telex); // chuôn → w → chươn
    trace("chuonwo", InputMethod::Telex); // chươn → o → chuôn

    // Category 4: With tones
    trace("traansw", InputMethod::Telex); // trấn → w → trắn

    // Basic typing
    trace("chaof ", InputMethod::Telex); // chào + space

    // VNI tests
    trace("tran6", InputMethod::Vni);
    trace("tran61", InputMethod::Vni);
    trace("tran618", InputMethod::Vni);
    trace("con67", InputMethod::Vni);
    trace("con627", InputMethod::Vni);

    // Smart cluster VNI
    trace("chuon67", InputMethod::Vni);
    trace("chuon76", InputMethod::Vni);
}
