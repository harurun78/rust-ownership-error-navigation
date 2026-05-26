#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    pub args: Vec<Vec<u8>>,
}

impl Command {
    pub fn new(args: Vec<Vec<u8>>) -> Self {
        Self { args }
    }
}
