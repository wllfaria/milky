use milky_bitboard::{Move, PromotedPieces, Square};
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
    /// This is sent to the engine when the user wants to change the internal parameters of the
    /// engine.
    ///
    /// One string will be sent for each parameter and this will only be sent when the engine is
    /// waiting. The name and value of the option in id should not be case sensitive and can inlude
    /// spaces.
    SetOption(SetOptionCommand),
    /// This is the command to try to register an engine or to tell the engine that registration
    /// will be done later. This command should always be sent if the engine has sent registration
    /// error at program startup.
    Register(RegisterCommand),
    /// This is sent to the engine when the next search (started with position and go) will be from
    /// a different game.
    ///
    /// As the engine's reaction to ucinewgame can take some time the GUI should always send isready
    /// after ucinewgame to wait for the engine to finish its operation.
    UciNewgame,
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
    /// Stop calculating as soon as possible.
    Stop,
    /// The user has played the expected move. This will be sent if the engine was told to ponder on
    /// the same move the user has played.
    ///
    /// The engine should continue searching but switch from pondering to normal search.
    PonderHit,
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
    /// This is needed for copyprotected engines. After the uciok command the engine can tell the
    /// GUI, that it will check the copy protection now. This is done by copyprotection checking.
    CopyProtection(CopyProtectionCommand),
    /// This is needed for engines that need a username and/or a code to function with all features.
    Registration(RegistrationCommand),
    /// The engine wants to send information to the GUI. This should be done whenever one of the
    /// info has changed.
    Info(InfoCommand),
    /// This command tells the GUI which parameters can be changed in the engine.
    ///
    /// This should be sent once at engine startup after the uci and the id commands if any
    /// parameter can be changed in the engine.
    Option(OptionCommand),
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
            UciCommand::SetOption(option_command) => write!(f, "{option_command}"),
            UciCommand::Register(register_command) => write!(f, "{register_command}"),
            UciCommand::UciNewgame => write!(f, "ucinewgame"),
            UciCommand::Position(position_command) => write!(f, "{position_command}"),
            UciCommand::Go(go_command) => write!(f, "{go_command}"),
            UciCommand::Stop => write!(f, "stop"),
            UciCommand::PonderHit => write!(f, "ponderhit"),
            UciCommand::Quit => write!(f, "quit"),

            UciCommand::Id(id_command) => write!(f, "{id_command}"),
            UciCommand::UciOk => write!(f, "uciok"),
            UciCommand::ReadyOk => write!(f, "readyok"),
            UciCommand::BestMove(best_move_command) => write!(f, "{best_move_command}"),
            UciCommand::CopyProtection(copy_command) => write!(f, "{copy_command}"),
            UciCommand::Registration(registration_command) => write!(f, "{registration_command}"),
            UciCommand::Info(info_command) => write!(f, "{info_command}"),
            UciCommand::Option(option_command) => write!(f, "{option_command}"),
        }
    }
}

/// This is sent to the engine when the user wants to change the internal parameters of the engine.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SetOptionCommand {
    /// Option name
    pub name: String,
    /// Option value
    pub value: Option<String>,
}

impl std::fmt::Display for SetOptionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut line = String::from("setoption");

        line.push_str(&format!(" name {}", self.name));

        if let Some(value) = self.value.as_ref() {
            line.push_str(&format!("value {value}"))
        }

        write!(f, "{line}")
    }
}

/// This is the command to try to register an engine or to tell the engine that registration will
/// be done later. This command should always be sent if the engine has sent registration error at
/// program startup.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegisterCommand {
    /// The user doesn't want to register the engine now.
    pub later: bool,
    /// The engine should be registered with the name.
    pub name: Option<String>,
    /// The engine should be registered with the code.
    pub code: Option<String>,
}

impl std::fmt::Display for RegisterCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut line = String::from("register");

        if self.later {
            line.push_str(" later");
        }

        if let Some(name) = &self.name {
            line.push_str(&format!(" name {name}"));
        }

        if let Some(code) = &self.code {
            line.push_str(&format!(" code {code}"));
        }

        write!(f, "{line}")
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
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

/// Tet up the position described in fenstring on the internal board and play the moves on the
/// internal chess board.
///
/// If the game was played from the start position the string startpos will be sent
///
/// NOTE: no "new" command is needed. However, if this position is from a different game than the
/// last position sent to the engine, the GUI should have sent a ucinewgame inbetween.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PositionCommand {
    /// Whether to use the start position, or a custom fenstring.
    pub start_position: bool,
    /// The parsed fenstring, for the specified position.
    pub fen: FenParts,
    /// Moves to be made after the position is loaded.
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

/// Start calculating on the current position set up with the position command.
///
/// There are a number of commands that can follow this command, all will be sent in the same
/// string. If one command is not sent its value should be interpreted as it would not
/// influence the search.
/// the search.
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct GoCommand {
    /// Restricts search to only the specified moves.
    pub search_moves: Vec<PartialMove>,
    /// Engine is to ponder during opponent's time.
    pub ponder: bool,
    /// Remaning time for white (in ms).
    pub white_time: Option<u64>,
    /// Remaning time for black (in ms).
    pub black_time: Option<u64>,
    /// White increment per move (in ms).
    pub white_inc: Option<u64>,
    /// Black increment per move (in ms).
    pub black_inc: Option<u64>,
    /// How many moves until next time control.
    pub movestogo: Option<u32>,
    /// Search with fixed ply depth.
    pub depth: u8,
    /// Search only X number of nodes.
    pub nodes: Option<u64>,
    /// Search for a forced mate in N plies.
    pub mate: Option<u32>,
    /// Search for this exact amount of time.
    pub movetime: Option<u64>,
    /// Search indefinitely until stopped via `UciCommand::Stop`.
    pub infinite: bool,
}

impl std::fmt::Display for GoCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "go depth {}", self.depth)
    }
}

/// Identifies the engine to the GUI.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IdCommand {
    /// Name of the engine.
    pub name: &'static str,
    /// Author of the engine.
    pub author: &'static str,
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

/// The engine has stopped searching and found the move move best in this position.
///
/// The engine can send the move it likes to ponder on. The engine must not start pondering
/// automatically.
///
/// This command must always be sent if the engine stops searching, also in pondering mode if there
/// is a stop command, so for every go command a bestmove command is needed
///
/// Directly before that the engine should send a final info command with the final search
/// information, the GUI has the complete statistics about the last search.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct BestMoveCommand {
    /// The best move found by the engine.
    pub best_move: String,
    /// Move the engine would like to ponder on.
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CopyProtectionCommand {
    Checking,
    Ok,
    Error,
}

impl std::fmt::Display for CopyProtectionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Checking => write!(f, "copyprotection checking"),
            Self::Ok => write!(f, "copyprotection ok"),
            Self::Error => write!(f, "copyprotection error"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum RegistrationCommand {
    Checking,
    Ok,
    Error,
}

impl std::fmt::Display for RegistrationCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Checking => write!(f, "registration checking"),
            Self::Ok => write!(f, "registration ok"),
            Self::Error => write!(f, "registration error"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScoreInfo {
    /// The score from the engine's point of view in centipawns.
    Cp(i32),
    /// Mate in y moves, not plies.
    ///
    /// If the engine is getting mated use negative values for y.
    Mate(i32),
    /// The score is just a lower bound.
    LowerBound,
    /// The score is just an upper bound.
    UpperBound,
}

impl std::fmt::Display for ScoreInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Cp(score) => write!(f, "cp {score}"),
            Self::Mate(mate) => write!(f, "mate {mate}"),
            Self::LowerBound => write!(f, "lowerbound"),
            Self::UpperBound => write!(f, "upperbound"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct CurrentLineInfo {
    /// The number of the cpu if the engine is running on more than one cpu.
    pub cpu_number: u32,
    /// The line of moves being calculated
    pub line: Vec<Move>,
}

impl std::fmt::Display for CurrentLineInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut line = String::from("currline");

        if self.cpu_number != 0 {
            line.push_str(&format!(" {}", self.cpu_number));
        }

        for mv in &self.line {
            line.push_str(&format!(" {mv}"));
        }

        write!(f, "{line}")
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct InfoCommand {
    /// Search depth in plies.
    pub depth: Option<u8>,
    /// Selected search depth in plies.
    ///
    /// If the engine sends `seldepth` there must also be a depth present in the same string.
    pub selective_depth: Option<u8>,
    /// The time searched in ms, this should be sent together with the pv.
    pub time: Option<u64>,
    /// Total nodes searched.
    ///
    /// The engine should send this regularly.
    pub nodes: Option<u64>,
    /// The best line found.
    pub pv: Option<Vec<Move>>,
    /// This is used in the Multi PV mode.
    ///
    /// For the best move/pv add `multipv` 1 in the string when you send the pv. In k-best mode
    /// always send all k variants in k strings together.
    pub multi_pv: Option<u64>,
    /// Current score for the searched position.
    pub score: Option<ScoreInfo>,
    /// Currently searching move X
    pub current_move: Option<Move>,
    /// Currently searching number X, the first move should be 1 and not 0.
    pub current_move_number: Option<u32>,
    /// How full the transposition table (hash table) is during search.
    ///
    /// The value is per mille (0–1000), not percent (0–100).
    pub hashfull: Option<u16>,
    /// The amount of nodes searched per second.
    pub nodes_per_second: Option<u32>,
    /// Amount of positions found in endgame table bases.
    pub table_base_hits: Option<u32>,
    /// Amount of position found in shredder endgame databases.
    pub shredder_base_hits: Option<u32>,
    /// Engine CPU usage.
    ///
    /// The value is per mille (0–1000), not percent (0–100).
    pub cpu_load: Option<u32>,
    /// Any string value to be displayed by the GUI.
    pub string: Option<String>,
    /// Move 1 (0th index) is refuted by the line Move 2..N (1st index..N)
    ///
    /// Example: after move d1h5 is searched, the engine can send info refutation d1h5 g6h5 if g6h5
    /// is the best answer after d1h5 or if g6h5 refutes the move d1h5.
    ///
    /// If there is no refutation for d1h5 found, the engine should just send info refutation d1h5.
    ///
    /// The engine should only send this if the option `UCI_ShowRefutations` is set to true.
    pub refutation: Option<Vec<Move>>,
    /// The current line being calculated.
    ///
    /// If the engine is just using one cpu, cpunr can be omitted.
    /// If cpunr is greater than 1, always send all k lines in k strings together.
    ///
    /// The engine should only send this if the option `UCI_ShowCurrLine` is set to true.
    pub current_line: Option<CurrentLineInfo>,
}

impl std::fmt::Display for InfoCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut line = String::from("info");

        if let Some(depth) = self.depth {
            line.push_str(&format!(" depth {depth}"));
        }

        if let Some(selective_depth) = self.selective_depth {
            line.push_str(&format!(" seldepth {selective_depth}"));
        }

        if let Some(time) = self.time {
            line.push_str(&format!(" time {time}"));
        }

        if let Some(nodes) = self.nodes {
            line.push_str(&format!(" nodes {nodes}"));
        }

        if let Some(pv) = &self.pv {
            line.push_str(" pv");

            for mv in pv.iter() {
                line.push_str(&format!(" {mv}"));
            }
        }

        if let Some(multi_pv) = self.multi_pv {
            line.push_str(&format!(" multipv {multi_pv}"));
        }

        if let Some(score) = &self.score {
            line.push_str(&format!(" score {score}"));
        }

        if let Some(curr_move) = self.current_move {
            line.push_str(&format!(" currmove {curr_move}"));
        }

        if let Some(curr_move_number) = self.current_move_number {
            line.push_str(&format!(" currmovenumber {curr_move_number}"));
        }

        if let Some(hashfull) = self.hashfull {
            line.push_str(&format!(" hashfull {hashfull}"));
        }

        if let Some(nps) = self.nodes_per_second {
            line.push_str(&format!(" nps {nps}"));
        }

        if let Some(tbhits) = self.table_base_hits {
            line.push_str(&format!(" tbhits {tbhits}"));
        }

        if let Some(sbhits) = self.shredder_base_hits {
            line.push_str(&format!(" sbhits {sbhits}"));
        }

        if let Some(cpu_load) = self.cpu_load {
            line.push_str(&format!(" cpuload {cpu_load}"));
        }

        if let Some(string) = &self.string {
            line.push_str(&format!(" string {string}"));
        }

        if let Some(refutation) = &self.refutation {
            line.push_str(" refutation");

            for mv in refutation {
                line.push_str(&format!(" {mv}"));
            }
        }

        if let Some(curr_line) = &self.current_line {
            line.push_str(&format!(" {curr_line}"));
        }

        write!(f, "{line}")
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptionType {
    Check { default: bool },
    Spin { default: i32, min: i32, max: i32 },
    Combo { default: String, vars: Vec<String> },
    Button,
    String { default: String },
}

/// This command tells the GUI which parameters can be changed in the engine.
///
/// Certain options have a fixed value for id, which means that the semantics of this option is
/// fixed. Usually those options should not be displayed in the normal engine options window of the
/// GUI but get a special treatment.
///
/// "Pondering" for example should be set automatically when pondering is enabled or disabled in
/// the GUI options.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct OptionCommand {
    /// Name of the option.
    name: String,
    option_type: OptionType,
}

impl std::fmt::Display for OptionCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut line = format!("option name {} type ", self.name);

        match &self.option_type {
            OptionType::Check { default } => {
                line.push_str(&format!(
                    "check default {}",
                    if *default { "true" } else { "false" }
                ));
            }
            OptionType::Spin { default, min, max } => {
                line.push_str(&format!("spin default {default} min {min} max {max}"));
            }
            OptionType::Combo { default, vars } => {
                line.push_str(&format!("combo default {default}"));
                for var in vars {
                    line.push_str(&format!(" var {var}"));
                }
            }
            OptionType::Button => {
                line.push_str("button");
            }
            OptionType::String { default } => {
                line.push_str(&format!("string default {default}"));
            }
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
