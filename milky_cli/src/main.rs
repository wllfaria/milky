use std::io::BufRead;

use milky_chess::{Milky, MoveKind};
use milky_protocols::Protocol;
use milky_protocols::uci::command::{
    BestMoveCommand, GoCommand, PositionCommand, START_POSITION, UciCommand,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    milky_chess::init_attack_tables();
    let mut milky = Milky::new();
    let mut uci = milky_protocols::uci::Uci;

    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut line = String::new();

    // milky.load_fen(milky_fen::parse_fen_string(START_POSITION).unwrap());

    // for _ in 0..20 {
    //     milky.search_position(3);
    //     milky.make_move(milky.best_move, MoveKind::AllMoves);
    // }
    //
    // milky.print_board();

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
            UciCommand::Position(position) => load_position(&mut milky, position),
            UciCommand::IsReady => println!("{}", UciCommand::ReadyOk),
            UciCommand::Quit => break,
            UciCommand::Go(go) => println!("{}", handle_go_command(&mut milky, go)),

            UciCommand::Debug(value) => println!("{value:?}"),

            // This set of commands are only sent from the engine to the GUI
            UciCommand::Id(_) => unreachable!(),
            UciCommand::UciOk => unreachable!(),
            UciCommand::ReadyOk => unreachable!(),
            UciCommand::BestMove(_) => unreachable!(),
        }

        milky.print_board();
    }

    Ok(())
}

fn load_position(milky: &mut Milky, position: PositionCommand) {
    milky.load_fen(position.fen);

    for move_to_make in position.moves.iter() {
        milky.generate_moves();

        let valid_move = milky.moves.iter().find(|m| {
            m.source() == move_to_make.source
                && m.target() == move_to_make.target
                && m.promotion() == move_to_make.promotion
        });

        // TODO: we should probably error on invalid move
        let Some(&valid_move) = valid_move else {
            return;
        };

        milky.make_move(valid_move, MoveKind::AllMoves);
    }
}

fn handle_go_command(milky: &mut Milky, go_command: GoCommand) -> BestMoveCommand {
    milky.search_position(go_command.depth);

    BestMoveCommand {
        best_move: milky.best_move.to_string(),
        ponder: None,
    }
}
