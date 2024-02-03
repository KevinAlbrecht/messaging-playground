pub enum Command {
    Nick,
    List,
    Read,
}

impl Command {
    pub fn from_str(s: &str) -> Option<Command> {
        match s {
            "/nick" => Some(Command::Nick),
            "/list" => Some(Command::List),
            "/read" => Some(Command::Read),
            _ => None,
        }
    }
}
