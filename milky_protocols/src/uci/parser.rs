use super::error::{Error, Result};
use super::{GoCommand, PositionCommand, UciCommand};

pub fn parse_uci_command(line: &str) -> Result<UciCommand> {
    if line.is_empty() {
        return Err(Error::InsufficientCommand("Empty command string".into()));
    }

    let mut split = line.split_whitespace();

    while let Some(token) = split.next() {
        match token {
            "uci" => return Ok(UciCommand::Uci),
            "debug" => return parse_debug_command(split),
            "isready" => return Ok(UciCommand::IsReady),
            "position" => return parse_position_command(split),
            "go" => return parse_go_command(split),
            "stop" => todo!("stop not yet implemented"),
            "ponderhit" => todo!("ponderhit not yet implemented"),
            "quit" => todo!("quit not yet implemented"),
            // Add more known commands as needed
            _ => continue,
        }
    }

    Err(Error::UnknownCommand("No recognizable UCI command found"))
}

fn parse_debug_command<'a>(mut args: impl Iterator<Item = &'a str>) -> Result<UciCommand> {
    match args.next() {
        Some("on") => Ok(UciCommand::Debug(true)),
        Some("off") => Ok(UciCommand::Debug(false)),
        Some(other) => Err(Error::InvalidCommand(format!(
            "Debug command expects `on` or `off`, got: `{other}`"
        ))),
        None => Err(Error::InsufficientCommand(
            "Debug command requires `on` or `off`".into(),
        )),
    }
}

fn parse_position_command<'a>(mut split: impl Iterator<Item = &'a str>) -> Result<UciCommand> {
    let Some(next) = split.next() else {
        return Err(Error::InsufficientCommand(
            "Position command must specify `startpos` or `fen`".into(),
        ));
    };

    let mut position = match next {
        "startpos" => PositionCommand::default(),
        "fen" => {
            // as far as I could check on specifications, UCI requires FEN strings to not have any
            // abbreviations, so it should always contains 6 parts
            let fen = split.by_ref().take(6).collect::<Vec<_>>().join(" ");
            let fen = milky_fen::parse_fen_string(&fen)?;
            PositionCommand { fen, moves: vec![] }
        }
        other => {
            return Err(Error::InvalidCommand(format!(
                "Expected `startpos` or `fen`, got: `{other}`"
            )));
        }
    };

    let Some(moves) = split.next() else {
        return Ok(UciCommand::Position(position));
    };

    if moves != "moves" {
        return Err(Error::InvalidCommand(format!(
            "Position expects `moves` or nothing, but got: {moves}"
        )));
    }

    for mov in split {
        if mov.len() > 5 || mov.len() < 4 {
            return Err(Error::InvalidCommand(format!(
                "move string must be in short algebraic notation, but got: {mov}"
            )));
        }
        position.moves.push(mov.to_string());
    }

    Ok(UciCommand::Position(position))
}

fn parse_go_command<'a>(mut split: impl Iterator<Item = &'a str>) -> Result<UciCommand> {
    let mut command = GoCommand { depth: 245 };

    while let Some(next) = split.next() {
        match next {
            "depth" => {
                let Some(depth_str) = split.next() else {
                    return Err(Error::InvalidCommand(
                        "Expected number after `depth`".into(),
                    ));
                };

                command.depth = depth_str
                    .parse()
                    .map_err(|_| Error::InvalidCommand("Invalid number for `depth`".into()))?;
            }
            "searchmoves" => todo!(),
            "ponder" => todo!(),
            "wtime" => todo!(),
            "btime" => todo!(),
            "winc" => todo!(),
            "binc" => todo!(),
            "movestogo" => todo!(),
            "nodes" => todo!(),
            "mate" => todo!(),
            "movetime" => todo!(),
            "infinite" => todo!(),
            other => {
                return Err(Error::InvalidCommand(format!(
                    "Unknown `go` argument: `{other}`"
                )));
            }
        }
    }

    Ok(UciCommand::Go(command))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uci::{GoCommand, START_POSITION};

    #[test]
    fn test_parse_uci_protocol_command() {
        let command = "uci";
        let result = parse_uci_command(command).unwrap();
        assert_eq!(result, UciCommand::Uci);

        let command = "uci gibberish after";
        let result = parse_uci_command(command).unwrap();
        assert_eq!(result, UciCommand::Uci);

        let command = "gibberish before uci";
        let result = parse_uci_command(command).unwrap();
        assert_eq!(result, UciCommand::Uci);
    }

    #[test]
    fn test_parse_debug_command() {
        let command = "debug on";
        let result = parse_uci_command(command).unwrap();
        assert_eq!(result, UciCommand::Debug(true));

        let command = "debug off";
        let result = parse_uci_command(command).unwrap();
        assert_eq!(result, UciCommand::Debug(false));

        let command = "       debug       off";
        let result = parse_uci_command(command).unwrap();
        assert_eq!(result, UciCommand::Debug(false));

        let command = "gibberish debug off gibberish";
        let result = parse_uci_command(command).unwrap();
        assert_eq!(result, UciCommand::Debug(false));

        let command = "       debug     ";
        let result = parse_uci_command(command).unwrap_err();
        assert!(matches!(result, Error::InsufficientCommand(_)));

        let command = "       debug   gibberish on  ";
        let result = parse_uci_command(command).unwrap_err();
        assert!(matches!(result, Error::InvalidCommand(_)));
    }

    #[test]
    fn test_parse_isready_command() {
        let command = "isready";
        let result = parse_uci_command(command).unwrap();
        assert_eq!(result, UciCommand::IsReady);

        let command = "       isready     ";
        let result = parse_uci_command(command).unwrap();
        assert_eq!(result, UciCommand::IsReady);

        let command = "gibberish isready  gibberish ";
        let result = parse_uci_command(command).unwrap();
        assert_eq!(result, UciCommand::IsReady);
    }

    #[test]
    fn test_parse_position_startpos_command() {
        let command = "position startpos";
        let result = parse_uci_command(command).unwrap();
        assert_eq!(result, UciCommand::Position(Default::default()));

        let command = "      position startpos       ";
        let result = parse_uci_command(command).unwrap();
        assert_eq!(result, UciCommand::Position(Default::default()));

        let command = "   gibberish   position startpos      ";
        let result = parse_uci_command(command).unwrap();
        assert_eq!(result, UciCommand::Position(Default::default()));

        let command = "   gibberish   position startpos      gibberish ";
        let result = parse_uci_command(command).unwrap_err();
        assert!(matches!(result, Error::InvalidCommand(_)));
    }

    #[test]
    fn test_parse_position_fen_command() {
        let command = "position fen 8/1B6/8/5p2/8/8/5Qrq/1K1R2bk w - - 0 1";
        let result = parse_uci_command(command).unwrap();

        let expected = PositionCommand {
            fen: milky_fen::parse_fen_string("8/1B6/8/5p2/8/8/5Qrq/1K1R2bk w - - 0 1").unwrap(),
            moves: vec![],
        };
        assert_eq!(result, UciCommand::Position(expected));

        let command = "      position fen 8/1B6/8/5p2/8/8/5Qrq/1K1R2bk w - - 0 1         ";
        let result = parse_uci_command(command).unwrap();

        let expected = PositionCommand {
            fen: milky_fen::parse_fen_string("8/1B6/8/5p2/8/8/5Qrq/1K1R2bk w - - 0 1").unwrap(),
            moves: vec![],
        };
        assert_eq!(result, UciCommand::Position(expected));

        let command = "   gibberish   position fen 8/1B6/8/5p2/8/8/5Qrq/1K1R2bk w - - 0 1         ";
        let result = parse_uci_command(command).unwrap();

        let expected = PositionCommand {
            fen: milky_fen::parse_fen_string("8/1B6/8/5p2/8/8/5Qrq/1K1R2bk w - - 0 1").unwrap(),
            moves: vec![],
        };
        assert_eq!(result, UciCommand::Position(expected));
    }

    #[test]
    fn test_parse_position_startpos_with_moves_command() {
        let command = "position startpos moves e2e4 e7e5 g1f3 b8c6 f1b5";
        let result = parse_uci_command(command).unwrap();

        let expected = PositionCommand {
            fen: milky_fen::parse_fen_string(START_POSITION).unwrap(),
            moves: vec![
                "e2e4".into(),
                "e7e5".into(),
                "g1f3".into(),
                "b8c6".into(),
                "f1b5".into(),
            ],
        };
        assert_eq!(result, UciCommand::Position(expected));

        let command =
            "   gibberish    position      startpos       moves e2e4 e7e5 g1f3 b8c6 f1b5         ";
        let result = parse_uci_command(command).unwrap();

        let expected = PositionCommand {
            fen: milky_fen::parse_fen_string(START_POSITION).unwrap(),
            moves: vec![
                "e2e4".into(),
                "e7e5".into(),
                "g1f3".into(),
                "b8c6".into(),
                "f1b5".into(),
            ],
        };
        assert_eq!(result, UciCommand::Position(expected));

        let command = "   gibberish    position      startpos       moves e2e4 e7e5 g1f3 b8c6 f1b5      gibberish   ";
        let result = parse_uci_command(command).unwrap_err();
        assert!(matches!(result, Error::InvalidCommand(_)));
    }

    #[test]
    fn test_parse_go_depth_command() {
        let command = "go depth 5";
        let result = parse_uci_command(command).unwrap();
        assert_eq!(result, UciCommand::Go(GoCommand { depth: 5 }));

        let command = "     gibberish     go depth 5";
        let result = parse_uci_command(command).unwrap();
        assert_eq!(result, UciCommand::Go(GoCommand { depth: 5 }));

        let command = "     gibberish     go gibberish depth 5";
        let result = parse_uci_command(command).unwrap_err();
        assert!(matches!(result, Error::InvalidCommand(_)));
    }
}
