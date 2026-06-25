use serde::Serialize;
use vietc_engine::{Engine, EngineEvent, InputMethod};

#[derive(Serialize)]
struct SnapshotTestCase {
    input: String,
    display: String,
    events: Vec<EngineEvent>,
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
            EngineEvent::Insert(text) => display.push_str(text),
            EngineEvent::Paste(text) => display.push_str(text),
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
            EngineEvent::UndoTones {
                backspaces,
                restored,
            } => {
                for _ in 0..*backspaces {
                    display.pop();
                }
                display.push_str(restored);
            }
        }
    }
    display
}

fn run_snapshot_test(inputs_json: &str, method: InputMethod) -> Vec<SnapshotTestCase> {
    let inputs: Vec<String> = serde_json::from_str(inputs_json).unwrap();
    let mut cases = Vec::new();

    for input in inputs {
        let mut engine = Engine::new(method);
        let mut events = Vec::new();
        for ch in input.chars() {
            if let Some(event) = engine.process_key(ch) {
                events.push(event);
            }
        }
        let display = get_display(&events);
        cases.push(SnapshotTestCase {
            input,
            display,
            events,
        });
    }

    cases
}

#[test]
fn test_telex_snapshots() {
    let inputs_json = include_str!("testdata/telex_inputs.json");
    let cases = run_snapshot_test(inputs_json, InputMethod::Telex);
    insta::assert_yaml_snapshot!(cases);
}

#[test]
fn test_vni_snapshots() {
    let inputs_json = include_str!("testdata/vni_inputs.json");
    let cases = run_snapshot_test(inputs_json, InputMethod::Vni);
    insta::assert_yaml_snapshot!(cases);
}
