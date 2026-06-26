use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMethod {
    Telex,
    Vni,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleEffect {
    Appending(char),
    MarkTransformation { base: char, marked: char },
    ToneTransformation { tone: char, name: &'static str },
}

#[derive(Debug, Clone)]
pub struct InputMethodRules {
    pub method: InputMethod,
    pub tone_keys: HashMap<char, (char, &'static str)>,
    pub mark_rules: Vec<(String, String)>,
    pub special_rules: Vec<RuleEffect>,
}

fn tone_map(entries: &[(char, char, &'static str)]) -> HashMap<char, (char, &'static str)> {
    entries.iter().map(|&(k, t, n)| (k, (t, n))).collect()
}

pub fn get_rules(method: InputMethod) -> InputMethodRules {
    match method {
        InputMethod::Telex => InputMethodRules {
            method,
            tone_keys: tone_map(&[
                ('f', 'f', "huyen"),
                ('s', 's', "sac"),
                ('r', 'r', "hoi"),
                ('x', 'x', "nga"),
                ('j', 'j', "nang"),
            ]),
            mark_rules: vec![
                ("aw".into(), "ă".into()),
                ("aa".into(), "â".into()),
                ("ee".into(), "ê".into()),
                ("oo".into(), "ô".into()),
                ("ow".into(), "ơ".into()),
                ("uw".into(), "ư".into()),
                ("dd".into(), "đ".into()),
            ],
            special_rules: vec![],
        },
        InputMethod::Vni => InputMethodRules {
            method,
            tone_keys: tone_map(&[
                ('1', '1', "sac"),
                ('2', '2', "huyen"),
                ('3', '3', "hoi"),
                ('4', '4', "nga"),
                ('5', '5', "nang"),
            ]),
            mark_rules: vec![
                ("a6".into(), "â".into()),
                ("e6".into(), "ê".into()),
                ("o6".into(), "ô".into()),
                ("o7".into(), "ơ".into()),
                ("u7".into(), "ư".into()),
                ("a8".into(), "ă".into()),
                ("d9".into(), "đ".into()),
            ],
            special_rules: vec![],
        },
    }
}
