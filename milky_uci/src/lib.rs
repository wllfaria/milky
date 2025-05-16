pub mod command;
pub mod error;
mod parser;

use command::UciCommand;
use error::Result;

pub struct Uci;

impl Uci {
    pub fn parse_command<S: AsRef<str>>(&mut self, line: S) -> Result<Option<UciCommand>> {
        UciCommand::parse(line.as_ref())
    }
}
