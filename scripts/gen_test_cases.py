#!/usr/bin/env python3
"""Generate 1000+ Vietnamese IME test cases and produce a Rust test file."""
import json
import subprocess
import sys
import os

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
PROJECT_DIR = os.path.normpath(os.path.join(SCRIPT_DIR, ".."))
EXAMPLE_PATH = os.path.join(PROJECT_DIR, "target", "release", "examples", "gen_tests")

def build_generator():
    """Build the Rust test case generator."""
    subprocess.run(
        ["cargo", "run", "--example", "gen_tests", "--release"],
        cwd=PROJECT_DIR,
        capture_output=True,
        check=True,
    )

def run_generator():
    """Run the generator and return JSON lines."""
    result = subprocess.run(
        [EXAMPLE_PATH],
        capture_output=True,
        text=True,
        timeout=30,
    )
    cases = []
    for line in result.stdout.strip().split("\n"):
        line = line.strip()
        if not line or line.startswith("Generated"):
            continue
        try:
            cases.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return cases

def generate_rust_test(cases, output_path):
    """Generate a Rust test file with all cases."""
    from datetime import datetime
    lines = []
    lines.append("/// Auto-generated from gen_tests example")
    lines.append(f"/// Generated: {datetime.now().isoformat()}")
    lines.append(f"/// Total cases: {len(cases)}")
    lines.append("")
    lines.append("use vietc_engine::{Engine, EngineEvent, InputMethod};")
    lines.append("")
    lines.append("fn get_display(events: &[EngineEvent]) -> String {")
    lines.append("    let mut display = String::new();")
    lines.append("    for ev in events {")
    lines.append("        match ev {")
    lines.append("            EngineEvent::Flush(text) => {")
    lines.append("                if !display.ends_with(text) { display.push_str(text); }")
    lines.append("            }")
    lines.append("            EngineEvent::Insert(text) => display.push_str(text),")
    lines.append("            EngineEvent::Replace { backspaces, insert } => {")
    lines.append("                for _ in 0..*backspaces { display.pop(); }")
    lines.append("                display.push_str(insert);")
    lines.append("            }")
    lines.append("            EngineEvent::AutoRestore(word) => {")
    lines.append("                for _ in 0..word.len() { display.pop(); }")
    lines.append("                display.push_str(word);")
    lines.append("            }")
    lines.append("            EngineEvent::UndoTones { backspaces, restored } => {")
    lines.append("                for _ in 0..*backspaces { display.pop(); }")
    lines.append("                display.push_str(restored);")
    lines.append("            }")
    lines.append("        }")
    lines.append("    }")
    lines.append("    display")
    lines.append("}")
    lines.append("")
    lines.append("fn process_input(e: &mut Engine, input: &str) -> Vec<EngineEvent> {")
    lines.append("    let mut events = Vec::new();")
    lines.append("    for ch in input.chars() {")
    lines.append("        if let Some(ev) = e.process_key(ch) { events.push(ev); }")
    lines.append("    }")
    lines.append("    events")
    lines.append("}")
    lines.append("")

    lines.append(f"const TEST_CASES: &[(&str, &str, &str)] = &[")
    for c in cases:
        input_escaped = c["i"].replace("\\", "\\\\").replace("\"", "\\\"")
        expected_escaped = c["e"].replace("\\", "\\\\").replace("\"", "\\\"")
        mode = c["m"]
        lines.append(f'    ("{input_escaped}", "{expected_escaped}", "{mode}"),')
    lines.append("];")
    lines.append("")

    lines.append("#[test]")
    lines.append("fn test_generated_bulk() {")
    lines.append("    let mut failures = Vec::new();")
    lines.append("    for (i, &(input, expected, mode)) in TEST_CASES.iter().enumerate() {")
    lines.append("        let im = match mode {")
    lines.append('            "telex" => InputMethod::Telex,')
    lines.append('            "vni" => InputMethod::Vni,')
    lines.append("            _ => unreachable!(),")
    lines.append("        };")
    lines.append("        let mut e = Engine::new(im);")
    lines.append("        let actual = get_display(&process_input(&mut e, input));")
    lines.append("        if actual != expected {")
    lines.append("            failures.push(format!(\"[{}] '{}' -> '{}' (expected '{}')\", i, input, actual, expected));")
    lines.append("        }")
    lines.append("    }")
    lines.append("    if !failures.is_empty() {")
    lines.append("        for f in &failures[..10.min(failures.len())] {")
    lines.append('            eprintln!("{}", f);')
    lines.append("        }")
    lines.append('        panic!("{}/{} tests FAILED", failures.len(), TEST_CASES.len());')
    lines.append("    }")
    lines.append('    eprintln!("All {} generated tests PASSED", TEST_CASES.len());')
    lines.append("}")
    lines.append("")

    with open(output_path, "w", encoding="utf-8") as f:
        f.write("\n".join(lines))

    print(f"Generated {output_path} with {len(cases)} test cases")

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser()
    parser.add_argument("--output", default="engine/tests/generated_bulk.rs",
                        help="Output Rust test file path")
    args = parser.parse_args()

    output_path = os.path.join(PROJECT_DIR, args.output)
    os.makedirs(os.path.dirname(output_path), exist_ok=True)

    # Run generator & capture cases
    try:
        result = subprocess.run(
            ["cargo", "run", "--example", "gen_tests", "--release"],
            cwd=PROJECT_DIR,
            capture_output=True,
            text=True,
            timeout=120,
        )
        cases = []
        for line in result.stdout.strip().split("\n"):
            line = line.strip()
            if not line or line.startswith("Generated"):
                continue
            try:
                cases.append(json.loads(line))
            except json.JSONDecodeError:
                continue
    except subprocess.TimeoutExpired:
        print("ERROR: Generator timed out", file=sys.stderr)
        sys.exit(1)
    except subprocess.CalledProcessError as e:
        print(f"ERROR: Generator failed: {e.stderr}", file=sys.stderr)
        sys.exit(1)

    print(f"Captured {len(cases)} test cases from generator")
    generate_rust_test(cases, output_path)
