use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Press,
    Release,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyEvent {
    pub code: u32,
    pub value: char,
    pub action: KeyAction,
}

impl KeyEvent {
    pub fn press(code: u32, value: char) -> Self {
        Self { code, value, action: KeyAction::Press }
    }

    pub fn release(code: u32, value: char) -> Self {
        Self { code, value, action: KeyAction::Release }
    }

    pub fn is_press(&self) -> bool {
        self.action == KeyAction::Press
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InjectResult {
    Success,
    Failed,
    NotSupported,
}

impl InjectResult {
    pub fn is_ok(&self) -> bool {
        *self == InjectResult::Success
    }
}

pub trait KeyInjector {
    fn send_backspace(&self) -> InjectResult;
    fn send_char(&self, ch: char) -> InjectResult;
    fn send_string(&self, s: &str) -> InjectResult;
    fn flush(&self) -> InjectResult;

    fn send_backspaces(&self, count: usize) -> InjectResult {
        for _ in 0..count {
            if self.send_backspace() != InjectResult::Success {
                return InjectResult::Failed;
            }
        }
        InjectResult::Success
    }

    fn inject_replacement(&self, backspaces: usize, text: &str) -> InjectResult {
        if self.send_backspaces(backspaces) != InjectResult::Success {
            return InjectResult::Failed;
        }
        self.send_string(text)
    }
}

impl fmt::Display for InjectResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InjectResult::Success => write!(f, "Success"),
            InjectResult::Failed => write!(f, "Failed"),
            InjectResult::NotSupported => write!(f, "NotSupported"),
        }
    }
}
