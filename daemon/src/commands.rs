#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputCommand {
    Type(String),
    Backspace(usize),
}
