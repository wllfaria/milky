use criterion::{Criterion, criterion_group, criterion_main};
use milky_chess::{Milky, MoveKind};

fn perft_driver(milky: &mut Milky, nodes: &mut usize, depth: u8) {
    if depth == 0 {
        *nodes += 1;
        return;
    }

    milky.generate_moves();

    for piece_move in milky.moves.into_iter().take(milky.move_count) {
        milky.snapshot_board();

        if !milky.make_move(piece_move, MoveKind::AllMoves) {
            continue;
        }

        perft_driver(milky, nodes, depth - 1);

        milky.undo_move();
    }
}

fn perft_depth_1(c: &mut Criterion) {
    milky_chess::init_attack_tables();

    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";
    let fen = milky_fen::parse_fen_string(fen).unwrap();
    let depth = 1;
    let expected_nodes = 48;

    c.bench_function("perft depth 1", |b| {
        b.iter(|| {
            let mut milky = Milky::new();
            milky.load_fen(fen.clone());

            let mut nodes = 0;
            perft_driver(&mut milky, &mut nodes, depth);

            assert_eq!(nodes, expected_nodes);
        });
    });
}

criterion_group!(benches, perft_depth_1);
criterion_main!(benches);
