use milky_chess::Milky;
use milky_chess::moves::{MoveKind, generate_moves_bench, make_move_bench};

fn perft_driver(milky: &mut Milky, nodes: &mut usize, depth: u8) {
    if depth == 0 {
        *nodes += 1;
        return;
    }

    generate_moves_bench(&mut milky.move_ctx());

    for piece_move in milky
        .search_state()
        .moves
        .into_iter()
        .take(milky.search_state().move_count)
    {
        let is_valid = make_move_bench(&mut milky.move_ctx(), piece_move, MoveKind::AllMoves);
        if !is_valid {
            continue;
        }

        perft_driver(milky, nodes, depth - 1);

        milky.zobrist_mut().position = milky.board_state_mut().undo_move();
    }
}

#[divan::bench(args = [0, 1, 2, 3, 4, 5, 6], sample_count = 1, sample_size = 1)]
fn perft_initial_position(b: divan::Bencher, depth: u8) {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let fen = milky_fen::parse_fen_string(fen).unwrap();
    let expected_nodes = [1, 20, 400, 8902, 197_281, 4_865_609, 119_060_324];

    b.bench_local(|| {
        let mut milky = Milky::new();
        milky.load_position(fen.clone());

        let mut nodes = 0;
        perft_driver(&mut milky, &mut nodes, depth);

        assert_eq!(nodes, expected_nodes[depth as usize]);
    });
}

#[divan::bench(args = [0, 1, 2, 3, 4, 5], sample_count = 1, sample_size = 1)]
fn perft_kiwipete_position(b: divan::Bencher, depth: u8) {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -";
    let fen = milky_fen::parse_fen_string(fen).unwrap();
    let expected_nodes = [1, 48, 2039, 97_862, 4_085_603, 193_690_690, 8_031_647_685];

    b.bench_local(|| {
        let mut milky = Milky::new();
        milky.load_position(fen.clone());

        let mut nodes = 0;
        perft_driver(&mut milky, &mut nodes, depth);

        assert_eq!(nodes, expected_nodes[depth as usize]);
    });
}

#[divan::bench(args = [0, 1, 2, 3, 4, 5, 6], sample_count = 1, sample_size = 1)]
fn perft_endgame_position(b: divan::Bencher, depth: u8) {
    let fen = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1";
    let fen = milky_fen::parse_fen_string(fen).unwrap();
    let expected_nodes = [1, 14, 191, 2812, 43_238, 674_624, 11_030_083];

    b.bench_local(|| {
        let mut milky = Milky::new();
        milky.load_position(fen.clone());

        let mut nodes = 0;
        perft_driver(&mut milky, &mut nodes, depth);

        assert_eq!(nodes, expected_nodes[depth as usize]);
    });
}

#[divan::bench(args = [0, 1, 2, 3, 4, 5, 6], sample_count = 1, sample_size = 1)]
fn perft_complex_position(b: divan::Bencher, depth: u8) {
    let fen = "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1";
    let fen = milky_fen::parse_fen_string(fen).unwrap();
    let expected_nodes = [1, 6, 264, 9467, 422_333, 15_833_292, 706_045_033];

    b.bench_local(|| {
        let mut milky = Milky::new();
        milky.load_position(fen.clone());

        let mut nodes = 0;
        perft_driver(&mut milky, &mut nodes, depth);

        assert_eq!(nodes, expected_nodes[depth as usize]);
    });
}

#[divan::bench(args = [0, 1, 2, 3, 4, 5], sample_count = 1, sample_size = 1)]
fn perft_knight_fork(b: divan::Bencher, depth: u8) {
    let fen = "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8";
    let fen = milky_fen::parse_fen_string(fen).unwrap();
    let expected_nodes = [1, 44, 1486, 62_379, 2_103_487, 89_941_194];

    b.bench_local(|| {
        let mut milky = Milky::new();
        milky.load_position(fen.clone());

        let mut nodes = 0;
        perft_driver(&mut milky, &mut nodes, depth);

        assert_eq!(nodes, expected_nodes[depth as usize]);
    });
}

#[divan::bench(args = [0, 1, 2, 3, 4, 5], sample_count = 1, sample_size = 1)]
fn perft_italian_game(b: divan::Bencher, depth: u8) {
    let fen = "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10";
    let fen = milky_fen::parse_fen_string(fen).unwrap();
    let expected_nodes = [1, 46, 2079, 89_890, 3_894_594, 164_075_551, 6_923_051_137];

    b.bench_local(|| {
        let mut milky = Milky::new();
        milky.load_position(fen.clone());

        let mut nodes = 0;
        perft_driver(&mut milky, &mut nodes, depth);

        assert_eq!(nodes, expected_nodes[depth as usize]);
    });
}

fn main() {
    milky_chess::init_static_members();
    divan::main();
}
