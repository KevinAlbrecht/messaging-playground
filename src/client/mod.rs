pub enum Command {
    Paste,
}

impl Command {
    pub fn from_str(s: &str) -> Option<Command> {
        match s {
            "/paste" => Some(Command::Paste),
            _ => None,
        }
    }
}
