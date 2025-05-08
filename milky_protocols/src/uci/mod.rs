use std::io::{BufRead, stdin};

mod error;
mod parser;

use error::Result;
use milky_fen::FenParts;

use crate::Protocol;

pub struct Uci;

impl Uci {}

static START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PositionCommand {
    fen: FenParts,
    moves: Vec<String>,
}

impl Default for PositionCommand {
    fn default() -> Self {
        PositionCommand {
            fen: milky_fen::parse_fen_string(START_POSITION).unwrap(),
            moves: Vec::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GoCommand {
    depth: u8,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UciCommand {
    Uci,
    Debug(bool),
    IsReady,
    Position(PositionCommand),
    Go(GoCommand),
}

impl UciCommand {
    pub fn parse(line: &str) -> Result<UciCommand> {
        parser::parse_uci_command(line)
    }
}

impl Protocol for Uci {
    fn start_loop(engine: &mut milky_chess::Milky) {
        loop {
            for line in stdin().lock().lines() {
                let line = line.unwrap();
            }
        }
    }
}
