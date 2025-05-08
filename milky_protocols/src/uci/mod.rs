use std::io::{BufRead, stdin};

mod command;
pub mod error;
mod parser;

use command::UciCommand;
use error::Result;

use crate::Protocol;

pub struct Uci;

impl Protocol for Uci {
    fn start_loop(&mut self, engine: &mut milky_chess::Milky) -> Result<()> {
        loop {
            for line in stdin().lock().lines() {
                let line = line.unwrap();
                let command = UciCommand::parse(&line)?;

                match command {
                    UciCommand::Uci => println!("{}", UciCommand::UciOk),
                    UciCommand::IsReady => println!("{}", UciCommand::ReadyOk),
                    UciCommand::Debug(value) => todo!(),
                    UciCommand::Position(position) => todo!(),
                    UciCommand::Go(go) => todo!(),

                    // This set of commands are only sent from the engine to the GUI
                    UciCommand::UciOk => unreachable!(),
                    UciCommand::ReadyOk => unreachable!(),
                    UciCommand::BestMove(_) => unreachable!(),
                }
            }
        }
    }
}
