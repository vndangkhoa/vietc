use std::fs::File;

const INITIALS: &[&str] = &[
    "", "b", "c", "ch", "d", "g", "gh", "h", "k", "kh", "l", "m", "n", "ng", "ngh", "nh", "p",
    "ph", "q", "r", "s", "t", "th", "tr", "v", "x",
];

const FINALS: &[&str] = &["", "c", "ch", "m", "n", "ng", "nh", "p", "t"];

fn is_valid(init: &str, fin: &str) -> bool {
    if init == "ngh" && !fin.is_empty() && fin != "n" && fin != "ng" && fin != "nh" {
        return false;
    }
    if init == "gh" && !fin.is_empty() {
        return false;
    }
    if init == "q" {
        return false;
    }
    if init == "g" && !fin.is_empty() && fin != "n" && fin != "ng" {
        return false;
    }
    if fin == "ch" && init.is_empty() {
        return false;
    }
    if fin == "nh" && init.is_empty() {
        return false;
    }
    true
}

fn main() {
    // Telex
    let telex_vowels: Vec<(&str, &str)> = vec![
        ("a", "af"),
        ("a", "as"),
        ("a", "aj"),
        ("a", "ar"),
        ("a", "ax"),
        ("a", "aw"),
        ("a", "aa"),
        ("e", "ee"),
        ("o", "oo"),
        ("o", "ow"),
        ("u", "uw"),
    ];

    let mut telex_inputs = Vec::new();
    for &init in INITIALS {
        for &fin in FINALS {
            if !is_valid(init, fin) {
                continue;
            }
            for &(base, mod_str) in &telex_vowels {
                let plain = format!("{}{}{}", init, base, fin);
                let full = format!("{}{}", plain, mod_str);
                if plain.len() > 10 {
                    continue;
                }
                telex_inputs.push(full);
            }
        }
    }
    // Limit to 500 cases to keep snapshot size reasonable but comprehensive
    telex_inputs.truncate(500);

    // VNI
    let vni_vowels: Vec<(&str, &str)> = vec![
        ("a", "1"),
        ("a", "2"),
        ("a", "3"),
        ("a", "4"),
        ("a", "5"),
        ("a", "6"),
        ("a", "8"),
        ("e", "6"),
        ("o", "6"),
        ("o", "7"),
        ("u", "7"),
    ];

    let mut vni_inputs = Vec::new();
    for &init in INITIALS {
        for &fin in FINALS {
            if !is_valid(init, fin) {
                continue;
            }
            for &(base, mod_str) in &vni_vowels {
                let plain = format!("{}{}{}", init, base, fin);
                let full = format!("{}{}", plain, mod_str);
                if plain.len() > 10 {
                    continue;
                }
                vni_inputs.push(full);
            }
        }
    }
    vni_inputs.truncate(500);

    // Ensure output directory exists
    std::fs::create_dir_all("tests/testdata").unwrap();

    let mut f_telex = File::create("tests/testdata/telex_inputs.json").unwrap();
    serde_json::to_writer_pretty(&mut f_telex, &telex_inputs).unwrap();

    let mut f_vni = File::create("tests/testdata/vni_inputs.json").unwrap();
    serde_json::to_writer_pretty(&mut f_vni, &vni_inputs).unwrap();

    println!(
        "Generated {} Telex and {} VNI test inputs under tests/testdata/",
        telex_inputs.len(),
        vni_inputs.len()
    );
}
