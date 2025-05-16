use std::io::BufRead;

use milky_chess::{Milky, MoveKind};
use milky_uci::command::{BestMoveCommand, GoCommand, PositionCommand, UciCommand};

#[allow(dead_code)]
static FEN_A: &str = "r2q1rk1/ppp2ppp/2n1bn2/2b1p3/3pP3/3P1NPP/PPP1NPB1/R1BQ1RK1 b - - 0 9 ";
#[allow(dead_code)]
static KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 ";
#[allow(dead_code)]
static FEN_C: &str = "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1";
#[allow(dead_code)]
static REPETITIONS: &str = "2r3k1/R7/8/1R6/8/8/P4KPP/8 w - - 0 40";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    milky_chess::init_attack_tables();
    let mut milky = Milky::new();
    let mut uci = milky_uci::Uci;

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

        milky.print_board();
    }

    Ok(())
}

fn load_position(milky: &mut Milky, position: PositionCommand) {
    milky.transposition_table.clear();
    milky.repetition_index = 0;
    milky.ply = 0;
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

        milky.record_repetition();
        milky.make_move(valid_move, MoveKind::AllMoves);
    }
}

fn handle_go_command(milky: &mut Milky, go_command: GoCommand) -> BestMoveCommand {
    let best_move = milky.search_position(go_command.depth);

    BestMoveCommand {
        best_move: best_move.to_string(),
        ponder: None,
    }
}
