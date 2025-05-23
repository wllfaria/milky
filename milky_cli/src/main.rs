use std::io::BufRead;

use milky_chess::Milky;
use milky_uci::command::{BestMoveCommand, GoCommand, PositionCommand, UciCommand};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    milky_chess::init_static_members();
    let mut milky = Milky::new();
    let mut uci = milky_uci::Uci;

    // let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - ";
    // let fen = "r3k2r/p1ppqpb1/1n2pnp1/3PN3/1p2P3/2N2Q1p/PPPB1PPP/R3K2R w KQkq - 0 1 ";
    // let fen = milky_fen::parse_fen_string(fen).unwrap();
    // milky.load_position(fen);
    // println!("{milky}");
    // println!("{}", milky.evaluate());

    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();

    loop {
        line.clear();
        handle.read_line(&mut line)?;

        let Some(command) = uci.parse_command(&line)? else {
            continue;
        };

        match command {
            UciCommand::Uci => {
                println!("{}", UciCommand::Id(Default::default()));
                println!("{}", UciCommand::UciOk);
            }
            UciCommand::Debug(_) => continue,
            UciCommand::IsReady => println!("{}", UciCommand::ReadyOk),

            UciCommand::SetOption(_) => continue,
            UciCommand::Register(_) => continue,
            UciCommand::UciNewgame => continue,

            UciCommand::Position(position) => load_position(&mut milky, position),
            UciCommand::Go(go) => println!("{}", handle_go_command(&mut milky, go)),

            UciCommand::Stop => continue,
            UciCommand::PonderHit => continue,
            UciCommand::Quit => break,

            // This set of commands are only sent from the engine to the GUI
            UciCommand::Id(_) => unreachable!(),
            UciCommand::UciOk => unreachable!(),
            UciCommand::ReadyOk => unreachable!(),
            UciCommand::BestMove(_) => unreachable!(),
            UciCommand::CopyProtection(_) => unreachable!(),
            UciCommand::Registration(_) => unreachable!(),
            UciCommand::Info(_) => unreachable!(),
            UciCommand::Option(_) => unreachable!(),
        }
    }

    Ok(())
}

fn load_position(milky: &mut Milky, position: PositionCommand) {
    milky.new_game();
    milky.load_position(position.fen);
    milky.load_moves(position.moves.into_iter());
}

fn handle_go_command(milky: &mut Milky, go_command: GoCommand) -> BestMoveCommand {
    milky.think(go_command);

    BestMoveCommand {
        best_move: milky.search_state().best_move().to_string(),
        ponder: None,
    }
}
