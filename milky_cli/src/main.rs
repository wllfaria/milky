use std::io::BufRead;

use milky_chess::transposition_table::TTFlag;
use milky_chess::{Milky, MoveKind};
use milky_protocols::Protocol;
use milky_protocols::uci::command::{
    BestMoveCommand, GoCommand, PositionCommand, START_POSITION, UciCommand,
};

static FEN_A: &str = "r2q1rk1/ppp2ppp/2n1bn2/2b1p3/3pP3/3P1NPP/PPP1NPB1/R1BQ1RK1 b - - 0 9 ";
static KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 ";
static FEN_C: &str = "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1";

fn perft_driver(milky: &mut Milky, depth: u8) -> usize {
    let mut nodes = 0;

    if depth == 0 {
        return nodes + 1;
    }

    milky.generate_moves();

    for piece_move in milky.moves.into_iter().take(milky.move_count) {
        if !milky.make_move(piece_move, MoveKind::AllMoves) {
            continue;
        }

        nodes += perft_driver(milky, depth - 1);

        milky.undo_move();
    }

    nodes
}

fn perft_test(milky: &mut Milky, depth: u8) {
    milky.generate_moves();
    let start = std::time::Instant::now();

    let mut nodes = 0;

    for piece_move in milky.moves.into_iter().take(milky.move_count) {
        if !milky.make_move(piece_move, MoveKind::AllMoves) {
            continue;
        }

        let cummulative_nodes = nodes;
        nodes += perft_driver(milky, depth - 1);

        let old_nodes = nodes - cummulative_nodes;

        milky.undo_move();

        println!("move {piece_move} nodes: {old_nodes}");
    }

    println!();
    println!("Depth: {depth}");
    println!("Nodes: {nodes}");
    println!("Time: {:?}", start.elapsed());
}

fn print_best_move(milky: &Milky, timer: std::time::Instant, depth: u8) {
    println!(
        "searching at depth {depth}, searched {} nodes, best move: {}",
        milky.nodes, milky.pv_table[0][0],
    );

    print!("PV line: ");
    for idx in 0..milky.pv_length[0] {
        print!("{} ", milky.pv_table[0][idx]);
    }
    println!();

    println!("search took: {:?}", timer.elapsed());
}

fn print_move_scores(milky: &mut Milky) {
    println!();
    for m in milky.moves.into_iter().take(milky.move_count) {
        let score = milky.score_move(m);
        println!("move: {m} score: {score}");
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    milky_chess::init_attack_tables();
    let mut milky = Milky::new();
    let mut uci = milky_protocols::uci::Uci;

    // #[cfg(not(debug_assertions))]
    #[cfg(debug_assertions)]
    {
        // milky.load_fen(milky_fen::parse_fen_string(START_POSITION).unwrap());
        milky.load_fen(milky_fen::parse_fen_string("4k3/Q7/8/4K3/8/8/8/8 w - - 0 1").unwrap());
        milky.print_board();

        // milky.transposition_table.clear();
        //
        // milky.transposition_table.set(
        //     milky.zobrist.position,
        //     1,
        //     19,
        //     TTFlag::Alpha,
        //     milky.pv_table[0][0],
        // );
        //
        // let score = milky
        //     .transposition_table
        //     .get(milky.zobrist.position, 20, 30, 1);

        // println!("{score:?}");

        // let start = std::time::Instant::now();
        milky.search_position(10);
        milky.make_move(milky.pv_table[0][0], MoveKind::AllMoves);
        milky.print_board();
        // print_best_move(&milky, start, depth);
        // perft_test(&mut milky, depth);

        return Ok(());
    }

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
    let best_move = milky.search_position(go_command.depth);

    BestMoveCommand {
        best_move: best_move.to_string(),
        ponder: None,
    }
}
