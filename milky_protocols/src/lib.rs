use uci::error::Result;

pub mod uci;

pub trait Protocol {
    fn parse_command<S: AsRef<str>>(&mut self, command: S) -> Result<uci::command::UciCommand>;
}
