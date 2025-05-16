use std::str::FromStr;

use milky_bitboard::{PromotedPieces, Square};

use super::command::{
    GoCommand, PartialMove, PositionCommand, RegisterCommand, SetOptionCommand, UciCommand,
};
use super::error::{Error, Result};

pub fn parse_uci_command(line: &str) -> Result<Option<UciCommand>> {
    if line.is_empty() {
        return Err(Error::InsufficientCommand("Empty command string".into()));
    }

    let mut split = line.split_whitespace();

    while let Some(token) = split.next() {
        match token {
            "uci" => return Ok(Some(UciCommand::Uci)),
            "debug" => return parse_debug_command(split),
            "isready" => return Ok(Some(UciCommand::IsReady)),
            "setoption" => return parse_set_option_command(split),
            "register" => return parse_register_command(split),
            "ucinewgame" => return Ok(Some(UciCommand::UciNewgame)),
            "position" => return parse_position_command(split),
            "go" => return parse_go_command(split),
            "stop" => return Ok(Some(UciCommand::Stop)),
            "ponderhit" => return Ok(Some(UciCommand::PonderHit)),
            "quit" => return Ok(Some(UciCommand::Quit)),
            _ => continue,
        }
    }

    Ok(None)
}

fn parse_debug_command<'a>(mut args: impl Iterator<Item = &'a str>) -> Result<Option<UciCommand>> {
    match args.next() {
        Some("on") => Ok(Some(UciCommand::Debug(true))),
        Some("off") => Ok(Some(UciCommand::Debug(false))),
        Some(other) => Err(Error::InvalidCommand(format!(
            "Debug command expects `on` or `off`, got: `{other}`"
        ))),
        None => Err(Error::InsufficientCommand(
            "Debug command requires `on` or `off`".into(),
        )),
    }
}

fn parse_position_command<'a>(
    mut split: impl Iterator<Item = &'a str>,
) -> Result<Option<UciCommand>> {
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
            PositionCommand {
                fen,
                moves: vec![],
                start_position: false,
            }
        }
        other => {
            return Err(Error::InvalidCommand(format!(
                "Expected `startpos` or `fen`, got: `{other}`"
            )));
        }
    };

    let Some(moves) = split.next() else {
        return Ok(Some(UciCommand::Position(position)));
    };

    if moves != "moves" {
        return Err(Error::InvalidCommand(format!(
            "Position expects `moves` or nothing, but got: {moves}"
        )));
    }

    for mov in split {
        if mov.len() > 5 || mov.len() < 4 {
            // ignore any non-valid moves
            continue;
        }

        let mov = parse_move(mov)?;

        position.moves.push(mov);
    }

    Ok(Some(UciCommand::Position(position)))
}

fn parse_move(mov: &str) -> Result<PartialMove> {
    let source = Square::from_algebraic_str(&mov[0..2])?;
    let target = Square::from_algebraic_str(&mov[2..4])?;
    let promotion = if mov.len() == 5 {
        PromotedPieces::from_algebraic_str(&mov[4..])?
    } else {
        PromotedPieces::NoPromotion
    };

    Ok(PartialMove {
        source,
        target,
        promotion,
    })
}

fn parse_go_command<'a>(mut split: impl Iterator<Item = &'a str>) -> Result<Option<UciCommand>> {
    let mut command = GoCommand {
        depth: 1,
        search_moves: vec![],
        ponder: false,
        white_time: None,
        black_time: None,
        white_inc: None,
        black_inc: None,
        movestogo: None,
        nodes: None,
        mate: None,
        movetime: None,
        infinite: false,
    };

    while let Some(next) = split.next() {
        match next {
            "searchmoves" => {
                for mov in split.by_ref() {
                    if mov.len() > 5 || mov.len() < 4 {
                        // ignore any non-valid moves
                        continue;
                    }

                    let mov = parse_move(mov)?;
                    command.search_moves.push(mov);
                }
            }
            "ponder" => command.ponder = true,
            "depth" => command.depth = parse_number(&mut split, "depth")?,
            "wtime" => command.white_time = Some(parse_number(&mut split, next)?),
            "btime" => command.black_time = Some(parse_number(&mut split, next)?),
            "winc" => command.white_inc = Some(parse_number(&mut split, next)?),
            "binc" => command.black_inc = Some(parse_number(&mut split, next)?),
            "movestogo" => command.movestogo = Some(parse_number(&mut split, next)?),
            "nodes" => command.nodes = Some(parse_number(&mut split, next)?),
            "mate" => command.mate = Some(parse_number(&mut split, next)?),
            "movetime" => command.movetime = Some(parse_number(&mut split, next)?),
            "infinite" => command.infinite = true,
            other => {
                return Err(Error::InvalidCommand(format!(
                    "Unknown `go` argument: `{other}`"
                )));
            }
        }
    }

    Ok(Some(UciCommand::Go(command)))
}

fn parse_number<'a, T: FromStr>(
    mut split: impl Iterator<Item = &'a str>,
    keyword: &str,
) -> Result<T> {
    let Some(numeral_str) = split.next() else {
        return Err(Error::InvalidCommand(format!(
            "Expected number after `{keyword}`"
        )));
    };

    numeral_str
        .parse()
        .map_err(|_| Error::InvalidCommand(format!("Invalid number for `{keyword}`")))
}

fn parse_set_option_command<'a>(
    mut split: impl Iterator<Item = &'a str>,
) -> Result<Option<UciCommand>> {
    let Some(keyword_name) = split.next() else {
        return Err(Error::InvalidCommand(
            "Expected `name` after `setoption`".into(),
        ));
    };

    if keyword_name != "name" {
        return Err(Error::InvalidCommand("Expected `name` keyword".into()));
    }

    let name = split
        .by_ref()
        .take_while(|&slice| slice != "value")
        .collect::<Vec<_>>()
        .join(" ");

    let value = split.collect::<Vec<_>>().join(" ");
    let value = match value.is_empty() {
        true => None,
        false => Some(value),
    };

    Ok(Some(UciCommand::SetOption(SetOptionCommand {
        name,
        value,
    })))
}

fn parse_register_command<'a>(
    mut split: impl Iterator<Item = &'a str>,
) -> Result<Option<UciCommand>> {
    let Some(register_kind) = split.next() else {
        return Err(Error::InvalidCommand(
            "Expected `name` or `later` after `register` command".into(),
        ));
    };

    if register_kind == "later" {
        return Ok(Some(UciCommand::Register(RegisterCommand {
            later: true,
            name: None,
            code: None,
        })));
    }

    let name = split
        .by_ref()
        .take_while(|&slice| slice != "code")
        .collect::<Vec<_>>()
        .join(" ");

    let code = split.collect::<Vec<_>>().join(" ");
    let code = match code.is_empty() {
        true => None,
        false => Some(code),
    };

    Ok(Some(UciCommand::Register(RegisterCommand {
        later: false,
        name: Some(name),
        code,
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::uci::command::{GoCommand, RegisterCommand, START_POSITION, SetOptionCommand};

    #[test]
    fn test_parse_uci_protocol_command() {
        let command = "uci";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(result, UciCommand::Uci);

        let command = "uci gibberish after";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(result, UciCommand::Uci);

        let command = "gibberish before uci";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(result, UciCommand::Uci);
    }

    #[test]
    fn test_parse_debug_command() {
        let command = "debug on";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(result, UciCommand::Debug(true));

        let command = "debug off";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(result, UciCommand::Debug(false));

        let command = "       debug       off";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(result, UciCommand::Debug(false));

        let command = "gibberish debug off gibberish";
        let result = parse_uci_command(command).unwrap().unwrap();
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
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(result, UciCommand::IsReady);

        let command = "       isready     ";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(result, UciCommand::IsReady);

        let command = "gibberish isready  gibberish ";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(result, UciCommand::IsReady);
    }

    #[test]
    fn test_parse_position_startpos_command() {
        let command = "position startpos";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(result, UciCommand::Position(Default::default()));

        let command = "      position startpos       ";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(result, UciCommand::Position(Default::default()));

        let command = "   gibberish   position startpos      ";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(result, UciCommand::Position(Default::default()));

        let command = "   gibberish   position startpos      gibberish ";
        let result = parse_uci_command(command).unwrap_err();
        assert!(matches!(result, Error::InvalidCommand(_)));
    }

    #[test]
    fn test_parse_position_fen_command() {
        let command = "position fen 8/1B6/8/5p2/8/8/5Qrq/1K1R2bk w - - 0 1";
        let result = parse_uci_command(command).unwrap().unwrap();

        let expected = PositionCommand {
            fen: milky_fen::parse_fen_string("8/1B6/8/5p2/8/8/5Qrq/1K1R2bk w - - 0 1").unwrap(),
            moves: vec![],
            start_position: false,
        };
        assert_eq!(result, UciCommand::Position(expected));

        let command = "      position fen 8/1B6/8/5p2/8/8/5Qrq/1K1R2bk w - - 0 1         ";
        let result = parse_uci_command(command).unwrap().unwrap();

        let expected = PositionCommand {
            fen: milky_fen::parse_fen_string("8/1B6/8/5p2/8/8/5Qrq/1K1R2bk w - - 0 1").unwrap(),
            moves: vec![],
            start_position: false,
        };
        assert_eq!(result, UciCommand::Position(expected));

        let command = "   gibberish   position fen 8/1B6/8/5p2/8/8/5Qrq/1K1R2bk w - - 0 1         ";
        let result = parse_uci_command(command).unwrap().unwrap();

        let expected = PositionCommand {
            fen: milky_fen::parse_fen_string("8/1B6/8/5p2/8/8/5Qrq/1K1R2bk w - - 0 1").unwrap(),
            moves: vec![],
            start_position: false,
        };
        assert_eq!(result, UciCommand::Position(expected));
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
    fn test_parse_position_startpos_with_moves_command() {
        let command = "position startpos moves e2e4 e7e5 g1f3 b8c6 f1b5";
        let result = parse_uci_command(command).unwrap().unwrap();

        let expected = PositionCommand {
            start_position: true,
            fen: milky_fen::parse_fen_string(START_POSITION).unwrap(),
            moves: vec![
                make_move("e2e4"),
                make_move("e7e5"),
                make_move("g1f3"),
                make_move("b8c6"),
                make_move("f1b5"),
            ],
        };
        assert_eq!(result, UciCommand::Position(expected));

        let command =
            "   gibberish    position      startpos       moves e2e4 e7e5 g1f3 b8c6 f1b5         ";
        let result = parse_uci_command(command).unwrap().unwrap();

        let expected = PositionCommand {
            fen: milky_fen::parse_fen_string(START_POSITION).unwrap(),
            start_position: true,
            moves: vec![
                make_move("e2e4"),
                make_move("e7e5"),
                make_move("g1f3"),
                make_move("b8c6"),
                make_move("f1b5"),
            ],
        };
        assert_eq!(result, UciCommand::Position(expected));
    }

    #[test]
    fn test_parse_go_depth_command() {
        let command = "go depth 5";
        let expected = GoCommand {
            depth: 5,
            ..Default::default()
        };

        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(result, UciCommand::Go(expected));

        let command = "     gibberish     go depth 5";
        let expected = GoCommand {
            depth: 5,
            ..Default::default()
        };

        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(result, UciCommand::Go(expected));

        let command = "     gibberish     go gibberish depth 5";
        let result = parse_uci_command(command).unwrap_err();
        assert!(matches!(result, Error::InvalidCommand(_)));
    }

    #[test]
    fn test_go_with_all_options() {
        let cmd = "go wtime 10000 btime 8000 winc 100 binc 100 depth 20 ponder movestogo 10 nodes 500000 mate 2 movetime 3000 infinite searchmoves e2e4 e7e5 b1c3 b8c6";
        let result = parse_uci_command(cmd).unwrap().unwrap();

        assert_eq!(
            result,
            UciCommand::Go(GoCommand {
                depth: 20,
                ponder: true,
                search_moves: vec![
                    parse_move("e2e4").unwrap(),
                    parse_move("e7e5").unwrap(),
                    parse_move("b1c3").unwrap(),
                    parse_move("b8c6").unwrap(),
                ],
                white_time: Some(10000),
                black_time: Some(8000),
                white_inc: Some(100),
                black_inc: Some(100),
                movestogo: Some(10),
                nodes: Some(500000),
                mate: Some(2),
                movetime: Some(3000),
                infinite: true,
            })
        );
    }

    #[test]
    fn test_go_invalid_keyword() {
        let cmd = "go depthx 10";
        let result = parse_uci_command(cmd);
        assert!(matches!(result, Err(Error::InvalidCommand(_))));
    }

    #[test]
    fn test_go_missing_value() {
        let cmd = "go depth";
        let result = parse_uci_command(cmd);
        assert!(matches!(result, Err(Error::InvalidCommand(_))));
    }

    #[test]
    fn test_go_empty() {
        let cmd = "go";
        let result = parse_uci_command(cmd).unwrap().unwrap();

        assert_eq!(
            result,
            UciCommand::Go(GoCommand {
                depth: 1,
                search_moves: vec![],
                ponder: false,
                white_time: None,
                black_time: None,
                white_inc: None,
                black_inc: None,
                movestogo: None,
                nodes: None,
                mate: None,
                movetime: None,
                infinite: false,
            })
        );
    }

    #[test]
    fn test_parse_setoption_command() {
        let command = "setoption name Threads";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(
            result,
            UciCommand::SetOption(SetOptionCommand {
                name: "Threads".into(),
                value: None,
            })
        );

        let command = "setoption name Threads value 4";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(
            result,
            UciCommand::SetOption(SetOptionCommand {
                name: "Threads".into(),
                value: Some("4".into()),
            })
        );

        let command = "setoption name Multi PV value 3";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(
            result,
            UciCommand::SetOption(SetOptionCommand {
                name: "Multi PV".into(),
                value: Some("3".into()),
            })
        );

        let command = "setoption name EvalFile value path/to/file.txt";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(
            result,
            UciCommand::SetOption(SetOptionCommand {
                name: "EvalFile".into(),
                value: Some("path/to/file.txt".into()),
            })
        );

        let command = "setoption value 3";
        assert!(parse_uci_command(command).is_err());

        let command = "setoption identifier";
        assert!(parse_uci_command(command).is_err());

        let command = "setoption";
        assert!(parse_uci_command(command).is_err());
    }

    #[test]
    fn test_parse_register_command() {
        let command = "register name wiru code ABC-123";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(
            result,
            UciCommand::Register(RegisterCommand {
                name: Some("wiru".to_string()),
                code: Some("ABC-123".to_string()),
                later: false,
            })
        );
    }

    #[test]
    fn test_parse_register_name_only() {
        let command = "register name wiru";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(
            result,
            UciCommand::Register(RegisterCommand {
                name: Some("wiru".to_string()),
                code: None,
                later: false,
            })
        );
    }

    #[test]
    fn test_parse_register_later() {
        let command = "register later";
        let result = parse_uci_command(command).unwrap().unwrap();
        assert_eq!(
            result,
            UciCommand::Register(RegisterCommand {
                name: None,
                code: None,
                later: true,
            })
        );
    }
}
