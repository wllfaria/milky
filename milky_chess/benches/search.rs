use milky_chess::Milky;

// #[divan::bench(args = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10])]
// fn search_king_and_rook_endgame(depth: u8) {
//     let fen = "8/8/4k3/8/8/2R5/8/5K2 w - - 0 1";
//     let fen = milky_fen::parse_fen_string(fen).unwrap();
//
//     let mut milky = Milky::new();
//     milky.load_fen(fen);
//     milky.search_position(depth);
// }
//
// #[divan::bench(args = [1, 2, 3, 4, 5, 6, 7], sample_count = 10, sample_size = 1)]
// fn search_initial_position(depth: u8) {
//     let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
//     let fen = milky_fen::parse_fen_string(fen).unwrap();
//
//     let mut milky = Milky::new();
//     milky.load_fen(fen);
//     milky.search_position(depth);
// }

#[divan::bench(args = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10], sample_count = 1, sample_size = 1)]
fn search_kiwipete(depth: u8) {
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 ";
    let fen = milky_fen::parse_fen_string(fen).unwrap();

    let mut milky = Milky::new();
    milky.load_fen(fen);
    milky.search_position(depth);
}

fn main() {
    milky_chess::init_attack_tables();
    divan::main();
}
