// SPDX-License-Identifier: MIT
//! Built-in VNI/Telex test cases: raw keystrokes -> expected Vietnamese.
//!
//! Cases are taken from vietc's own README and integration test
//! (daemon/tests/daemon_suite.rs), so the expected output matches the
//! engine's actual behavior.

pub struct Case {
    pub method: &'static str,
    pub input: &'static str,
    pub expected: &'static str,
}

pub fn cases() -> Vec<Case> {
    vec![
        // --- VNI ---
        Case { method: "vni", input: "a1", expected: "á" },
        Case { method: "vni", input: "a2", expected: "à" },
        Case { method: "vni", input: "a6", expected: "â" },
        Case { method: "vni", input: "o7", expected: "ơ" },
        Case { method: "vni", input: "u7", expected: "ư" },
        Case { method: "vni", input: "d9", expected: "đ" },
        Case { method: "vni", input: "tho2i ", expected: "thời " },
        Case { method: "vni", input: "cha2o ", expected: "chào " },
        Case { method: "vni", input: "ko", expected: "không" },
        // --- Telex ---
        Case { method: "telex", input: "as", expected: "á" },
        Case { method: "telex", input: "aa", expected: "â" },
        Case { method: "telex", input: "ow", expected: "ơ" },
        Case { method: "telex", input: "uw", expected: "ư" },
        Case { method: "telex", input: "dd", expected: "đ" },
        Case { method: "telex", input: "chuongw ", expected: "chương " },
    ]
}
