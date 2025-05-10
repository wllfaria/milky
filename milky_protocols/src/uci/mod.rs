pub mod command;
pub mod error;
mod parser;

use command::UciCommand;
use error::Result;

use crate::Protocol;

pub struct Uci;

impl Protocol for Uci {
    fn parse_command<S: AsRef<str>>(&mut self, line: S) -> Result<Option<UciCommand>> {
        let line = line.as_ref();
        UciCommand::parse(line)
    }
}
