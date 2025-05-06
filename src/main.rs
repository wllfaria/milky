use milky_chess::{Milky, MoveKind, init_attack_tables};

fn perft_driver(milky: &mut Milky, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
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

fn main() {
    init_attack_tables();

    let depth = 3;
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";
    let fen = milky_fen::parse_fen_string(fen).unwrap();

    let mut milky = Milky::new();
    milky.load_fen(fen);

    let nodes = perft_driver(&mut milky, depth);
    println!("{nodes:?}");
}
