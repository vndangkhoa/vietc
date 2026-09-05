// SPDX-License-Identifier: MIT
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMethod {
    Telex,
    Vni,
}

#[derive(Debug, Clone)]
pub struct InputMethodRules {
    pub method: InputMethod,
    pub tone_keys: HashMap<char, (char, &'static str)>,
    pub mark_rules: Vec<MarkRule>,
}

#[derive(Debug, Clone)]
pub struct MarkRule {
    pub pattern: String,
    pub result: String,
    pub undo_behavior: Option<UndoBehavior>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UndoBehavior {
    AppendTrigger,
    ConsumeTrigger,
}

fn mark_rule(pattern: &str, result: &str, undo_behavior: Option<UndoBehavior>) -> MarkRule {
    MarkRule {
        pattern: pattern.into(),
        result: result.into(),
        undo_behavior,
    }
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
                mark_rule("aw", "ă", Some(UndoBehavior::AppendTrigger)),
                mark_rule("aa", "â", Some(UndoBehavior::AppendTrigger)),
                mark_rule("ee", "ê", Some(UndoBehavior::AppendTrigger)),
                mark_rule("oo", "ô", Some(UndoBehavior::AppendTrigger)),
                mark_rule("ow", "ơ", Some(UndoBehavior::AppendTrigger)),
                mark_rule("uw", "ư", Some(UndoBehavior::AppendTrigger)),
                mark_rule("dd", "đ", Some(UndoBehavior::ConsumeTrigger)),
            ],
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
                mark_rule("a6", "â", None),
                mark_rule("e6", "ê", None),
                mark_rule("o6", "ô", None),
                mark_rule("o7", "ơ", None),
                mark_rule("u7", "ư", None),
                mark_rule("a8", "ă", None),
                mark_rule("d9", "đ", None),
            ],
        },
    }
}
