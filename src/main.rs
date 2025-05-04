use std::io::Write;

use milky_bitboard::{Pieces, Side};
use milky_chess::{Milky, MoveKind, init_attack_tables};

fn wait_for_enter() {
    std::io::stdout().flush().unwrap(); // Ensure the message is printed
    let _ = std::io::stdin().read_line(&mut String::new());
}

fn perft_driver(milky: &mut Milky, nodes: &mut usize, depth: u8) {
    if depth == 0 {
        *nodes += 1;
        return;
    }

    let start = std::time::Instant::now();
    milky.generate_moves();

    for piece_move in milky.moves.into_iter().take(milky.move_count) {
        milky.snapshot_board();

        if !milky.make_move(piece_move, MoveKind::AllMoves) {
            continue;
        }

        perft_driver(milky, nodes, depth - 1);

        milky.undo_move();
    }

    println!("took: {:?}", start.elapsed());
}

fn main() {
    init_attack_tables();

    let depth = 1;
    let expected_nodes = 48;
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let fen = milky_fen::parse_fen_string(fen).unwrap();

    let mut milky = Milky::new();
    milky.load_fen(fen);

    let mut nodes = 0;
    perft_driver(&mut milky, &mut nodes, depth);
    assert_eq!(nodes, expected_nodes);
    // init_attack_tables();
    // let mut milky = Milky::new();
    //
    // // let original = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 ";
    // // let pos = "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1";
    // // let pos = "r3k2r/pPppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 ";
    // let pos = "r3k2r/p11pqpb1/1n2pnp1/2pPN3/1p2P3/2N2Q1p/PPPBqPPP/R3K2R w KQkq c6 0 1 "; // en passant
    // let fen_parts = milky_fen::parse_fen_string(pos).unwrap();
    //
    // milky.load_fen(fen_parts);
    // milky.print_board();
    // milky.generate_moves();
    //
    // for piece_move in milky.moves.into_iter().take(milky.move_count) {
    //     milky.snapshot_board();
    //
    //     if !milky.make_move(piece_move, MoveKind::AllMoves) {
    //         continue;
    //     }
    //
    //     println!();
    //     println!("Move: {piece_move}");
    //
    //     milky.print_board();
    //     // println!("{}", milky.boards[Pieces::BlackPawn]);
    //     // println!("{}", milky.occupancies[Side::White]);
    //
    //     wait_for_enter();
    //
    //     milky.undo_move();
    //     // milky.print_board();
    //     // println!("{}", milky.boards[Pieces::BlackPawn]);
    //     // println!("{}", milky.occupancies[Side::White]);
    //
    //     wait_for_enter();
    // }
}
