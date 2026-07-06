#[derive(Debug)]
pub enum OutputCommand {
    Type(String),
    Backspace(usize),
}
