use milky_bitboard::{PromotedPieces, Square};
use milky_fen::FenParts;

use super::error::Result;
use super::parser::parse_uci_command;

pub static START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UciCommand {
    /// Tell the engine to use the UCI (Universal Chess Interface), this will be sent once as a
    /// first command after program boot to tell the engine to switch to UCI mode.
    ///
    /// After receiving the uci command the engine must identify itself with the id command and send
    /// the option commands to tell the GUI which engine settings the engine supports if any.
    ///
    /// After that the engine should send uciok to acknowledge the uci mode. If no uciok is sent
    /// within a certain time period, the engine task will be killed by the GUI.
    Uci,
    /// Switch the debug mode of the engine on and off. In debug mode the engine should send
    /// additional infos to the GUI, e.g. with the info string command, to help debugging,
    /// e.g. the commands that the engine has received etc.
    ///
    /// This mode should be switched off by default and this command can be sent any time, also
    /// when the engine is thinking.
    Debug(bool),
    /// This is used to synchronize the engine with the GUI. When the GUI has sent a command or
    /// multiple commands that can take some time to complete, this command can be used to wait for
    /// the engine to be ready again or to ping the engine to find out if it is still alive.
    ///
    /// This command must always be answered with readyok and can be sent also when the engine is
    /// calculating in which case the engine should also immediately answer with readyok without
    /// stopping the search.
    IsReady,
    /// Set up the position described in fenstring on the internal board and play the moves on the
    /// internal chess board.
    ///
    /// If the game was played from the start position the string startpos will be sent
    ///
    /// Note: no "new" command is needed. However, if this position is from a different game than
    /// the last position sent to the engine, the GUI should have sent a ucinewgame inbetween.
    Position(PositionCommand),
    /// Start calculating on the current position set up with the position command.
    ///
    /// There are a number of commands that can follow this command, all will be sent in the same
    /// string. If one command is not sent its value should be interpreted as it would not
    /// influence the search.
    Go(GoCommand),
    /// Quit the program as soon as possible
    Quit,

    /// This must be sent after receiving the uci command to identify the engine
    Id(IdCommand),
    /// Must be sent after the id and optional options to tell the GUI that the engine has sent all
    /// infos and is ready in uci mode.
    UciOk,
    /// This must be sent when the engine has received an isready command and has processed all
    /// input and is ready to accept new commands now.
    ///
    /// It is usually sent after a command that can take some time to be able to wait for the
    /// engine, but it can be used anytime, even when the engine is searching, and must always be
    /// answered with isready.
    ReadyOk,
    /// The engine has stopped searching and found the move move best in this position.
    ///
    /// The engine can send the move it likes to ponder on. The engine must not start pondering
    /// automatically.
    ///
    /// This command must always be sent if the engine stops searching, also in pondering mode if
    /// there is a stop command, so for every go command a bestmove command is needed
    ///
    /// Directly before that the engine should send a final info command with the final search
    /// information, the the GUI has the complete statistics about the last search.
    BestMove(BestMoveCommand),
}

impl UciCommand {
    pub fn parse(line: &str) -> Result<Option<UciCommand>> {
        parse_uci_command(line)
    }
}

impl std::fmt::Display for UciCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UciCommand::Uci => write!(f, "uci"),
            UciCommand::Debug(true) => write!(f, "debug on"),
            UciCommand::Debug(false) => write!(f, "debug off"),
            UciCommand::IsReady => write!(f, "isready"),
            UciCommand::Position(position_command) => write!(f, "{position_command}"),
            UciCommand::Go(go_command) => write!(f, "{go_command}"),
            UciCommand::Quit => write!(f, "quit"),

            UciCommand::Id(id_command) => write!(f, "{id_command}"),
            UciCommand::UciOk => write!(f, "uciok"),
            UciCommand::ReadyOk => write!(f, "readyok"),
            UciCommand::BestMove(best_move_command) => write!(f, "{best_move_command}"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PartialMove {
    pub source: Square,
    pub target: Square,
    pub promotion: PromotedPieces,
}

impl std::fmt::Display for PartialMove {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.source, self.target, self.promotion)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PositionCommand {
    pub start_position: bool,
    pub fen: FenParts,
    pub moves: Vec<PartialMove>,
}

impl Default for PositionCommand {
    fn default() -> Self {
        PositionCommand {
            start_position: true,
            fen: milky_fen::parse_fen_string(START_POSITION).unwrap(),
            moves: Vec::default(),
        }
    }
}

impl std::fmt::Display for PositionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut line = String::from("position ");

        if self.start_position {
            line.push_str("startpos");
        } else {
            line.push_str("fen ");
            line.push_str(&self.fen.original);
        }

        if !self.moves.is_empty() {
            line.push_str(" moves");
        }

        for mov in self.moves.iter() {
            line.push_str(&format!(" {mov}"))
        }

        write!(f, "{line}")
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GoCommand {
    /// Search x plies only.
    pub depth: u8,
}

impl std::fmt::Display for GoCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "go depth {}", self.depth)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdCommand {
    name: &'static str,
    author: &'static str,
}

impl Default for IdCommand {
    fn default() -> Self {
        Self {
            name: "milky",
            author: "wiru",
        }
    }
}

impl std::fmt::Display for IdCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "id name {}", self.name)?;
        write!(f, "id author {}", self.author)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BestMoveCommand {
    pub best_move: String,
    pub ponder: Option<String>,
}

impl std::fmt::Display for BestMoveCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut line = String::from("bestmove");
        line.push_str(&format!(" {}", self.best_move));

        if let Some(ponder) = &self.ponder {
            line.push_str(&format!(" ponder {ponder}"));
        }

        write!(f, "{line}")
    }
}

#[cfg(test)]
mod tests {
    use milky_fen::parse_fen_string;

    use super::*;

    #[test]
    fn test_best_move_command_print() {
        let command = BestMoveCommand {
            best_move: "e2e4".into(),
            ponder: Some("d2d4".into()),
        };
        let command_str = command.to_string();
        assert_eq!(command_str, "bestmove e2e4 ponder d2d4");

        let command = BestMoveCommand {
            best_move: "e2e4".into(),
            ponder: None,
        };
        let command_str = command.to_string();
        assert_eq!(command_str, "bestmove e2e4");
    }

    fn make_move(move_str: &str) -> PartialMove {
        let source = Square::from_algebraic_str(&move_str[0..2]).unwrap();
        let target = Square::from_algebraic_str(&move_str[2..4]).unwrap();
        let promotion = if move_str.len() == 5 {
            PromotedPieces::from_algebraic_str(&move_str[4..]).unwrap()
        } else {
            PromotedPieces::NoPromotion
        };
        PartialMove {
            source,
            target,
            promotion,
        }
    }

    #[test]
    fn test_position_command_print() {
        let command = PositionCommand {
            start_position: true,
            fen: parse_fen_string(START_POSITION).unwrap(),
            moves: vec![make_move("e2e4"), make_move("e7e5")],
        };
        let command_str = command.to_string();
        assert_eq!(command_str, "position startpos moves e2e4 e7e5");

        let command = PositionCommand {
            start_position: true,
            fen: parse_fen_string(START_POSITION).unwrap(),
            moves: vec![],
        };
        let command_str = command.to_string();
        assert_eq!(command_str, "position startpos");

        let command = PositionCommand {
            start_position: false,
            fen: parse_fen_string("8/8/8/8/8/8/8/8 w KQkq - 0 1").unwrap(),
            moves: vec![],
        };
        let command_str = command.to_string();
        assert_eq!(command_str, "position fen 8/8/8/8/8/8/8/8 w KQkq - 0 1");

        let command = PositionCommand {
            start_position: false,
            fen: parse_fen_string("8/8/8/8/8/8/8/8 w KQkq - 0 1").unwrap(),
            moves: vec![make_move("e2e4"), make_move("e7e5")],
        };
        let command_str = command.to_string();
        assert_eq!(
            command_str,
            "position fen 8/8/8/8/8/8/8/8 w KQkq - 0 1 moves e2e4 e7e5"
        );
    }
}
