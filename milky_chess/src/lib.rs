mod evaluate;
mod random;
mod time_manager;
pub mod transposition_table;
pub mod zobrist;

use std::num::Wrapping;
use std::sync::OnceLock;

use evaluate::evaluate_position;
use milky_bitboard::{
    BitBoard, CastlingRights, Move, MoveFlags, Pieces, PromotedPieces, Rank, Side, Square,
};
use milky_fen::FenParts;
use random::Random;
use transposition_table::{TTFlag, TranspositionTable};
use zobrist::{GamePosition, Zobrist, ZobristKey};

pub static MAX_PLY: usize = 64;
pub static MAX_REPETITIONS: usize = 1024;
pub static INFINITY: i32 = 50000;
pub static MATE_UPPER_BOUND: i32 = 49000;
pub static MATE_LOWER_BOUND: i32 = 48000;

static PAWN_ATTACKS: OnceLock<[[BitBoard; 64]; 2]> = OnceLock::new();
static KNIGHT_ATTACKS: OnceLock<[BitBoard; 64]> = OnceLock::new();
static KING_ATTACKS: OnceLock<[BitBoard; 64]> = OnceLock::new();
static BISHOP_ATTACKS: OnceLock<Box<[[BitBoard; 512]]>> = OnceLock::new();
static ROOK_ATTACKS: OnceLock<Box<[[BitBoard; 4096]]>> = OnceLock::new();

static BISHOP_BLOCKERS: OnceLock<[BitBoard; 64]> = OnceLock::new();
static ROOK_BLOCKERS: OnceLock<[BitBoard; 64]> = OnceLock::new();

macro_rules! attacks {
    ($attacks:ident) => {{ $attacks.get().unwrap() }};
}

/// Every bit is set except for the bits on the A file
static EMPTY_A_FILE: BitBoard = BitBoard::new(0xFEFEFEFEFEFEFEFE);

/// Every bit is set except for the bits on the H file
static EMPTY_H_FILE: BitBoard = BitBoard::new(0x7F7F7F7F7F7F7F7F);

/// Every bit is set except for the bits on the GH files
static EMPTY_GH_FILE: BitBoard = BitBoard::new(0x3F3F3F3F3F3F3F3F);

/// Every bit is set except for the bits on the AB files
static EMPTY_AB_FILE: BitBoard = BitBoard::new(0xFCFCFCFCFCFCFCFC);

/// ┌────────────────┬─────────────┬────────┬─────────────────────────────────────────────────────────┐
/// │ Castling right │ Move square │ Result │ Description                                             │
/// ├────────────────┼─────────────┼────────┼─────────────────────────────────────────────────────────┤
/// │ 1111 (kqQK)    │ 1111 (15)   │ 1111   │ Neither rook or king moved, castling is unchanged       │
/// ├────────────────┼─────────────┼────────┼─────────────────────────────────────────────────────────┤
/// │ 1111 (qkQK)    │ 1100 (12)   │ 1100   │ White king moved, white can no longer castle            │
/// ├────────────────┼─────────────┼────────┼─────────────────────────────────────────────────────────┤
/// │ 1111 (qkQK)    │ 1110 (14)   │ 1110   │ White king's rook moved, white can't castle king side   │
/// ├────────────────┼─────────────┼────────┼─────────────────────────────────────────────────────────┤
/// │ 1111 (qkQK)    │ 1101 (13)   │ 1101   │ White queen's rook moved, white can't castle queen side │
/// ├────────────────┼─────────────┼────────┼─────────────────────────────────────────────────────────┤
/// │ 1111 (qkQK)    │ 0011 ( 3)   │ 0011   │ Black king moved, black can no longer castle            │
/// ├────────────────┼─────────────┼────────┼─────────────────────────────────────────────────────────┤
/// │ 1111 (qkQK)    │ 1011 (11)   │ 1011   │ Black king's rook moved, black can't castle king side   │
/// ├────────────────┼─────────────┼────────┼─────────────────────────────────────────────────────────┤
/// │ 1111 (qkQK)    │ 0111 ( 7)   │ 0111   │ Black queen's rook moved, black can't castle queen side │
/// └────────────────┴─────────────┴────────┴─────────────────────────────────────────────────────────┘
#[rustfmt::skip]
static CASTLING_RIGHTS: [u8; 64] = [
     7, 15, 15, 15,  3, 15, 15, 11,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    15, 15, 15, 15, 15, 15, 15, 15,
    13, 15, 15, 15, 12, 15, 15, 14,
];

#[rustfmt::skip]
pub static PIECE_SCORE: [i32; 12] = [
    100,  // white pawn score
    300,  // white knight scrore
    350,  // white bishop score
    500,  // white rook score
   1000,  // white queen score
  10000,  // white king score
   -100,  // black pawn score
   -300,  // black knight scrore
   -350,  // black bishop score
   -500,  // black rook score
  -1000,  // black queen score
 -10000,  // black king score
];

/// # Most Valuable Victim / Less Valuable Attacker table
///
/// This table is used to apply a bonus to captures based on the values of the pieces. Getting a
/// better beta cuttof on alpha-beta-search.
///
/// # Ordering is:
///
/// .  P   N   B   R   Q   K
/// P 105 205 305 405 505 605
/// N 104 204 304 404 504 604
/// B 103 203 303 403 503 603
/// R 102 202 302 402 502 602
/// Q 101 201 301 401 501 601
/// K 100 200 300 400 500 600
///
/// The table contains twice the size above to enable indexing with `Pieces`.
///
#[rustfmt::skip]
static MVV_LVA: [[i32; 12]; 12] = [
    [105, 205, 305, 405, 505, 605,  105, 205, 305, 405, 505, 605],
    [104, 204, 304, 404, 504, 604,  104, 204, 304, 404, 504, 604],
    [103, 203, 303, 403, 503, 603,  103, 203, 303, 403, 503, 603],
    [102, 202, 302, 402, 502, 602,  102, 202, 302, 402, 502, 602],
    [101, 201, 301, 401, 501, 601,  101, 201, 301, 401, 501, 601],
    [100, 200, 300, 400, 500, 600,  100, 200, 300, 400, 500, 600],

    [105, 205, 305, 405, 505, 605,  105, 205, 305, 405, 505, 605],
    [104, 204, 304, 404, 504, 604,  104, 204, 304, 404, 504, 604],
    [103, 203, 303, 403, 503, 603,  103, 203, 303, 403, 503, 603],
    [102, 202, 302, 402, 502, 602,  102, 202, 302, 402, 502, 602],
    [101, 201, 301, 401, 501, 601,  101, 201, 301, 401, 501, 601],
    [100, 200, 300, 400, 500, 600,  100, 200, 300, 400, 500, 600],
];

#[rustfmt::skip]
pub static PAWN_POS_SCORE: [i32; 64] = [
    90,  90,  90,  90,  90,  90,  90,  90,
    30,  30,  30,  40,  40,  30,  30,  30,
    20,  20,  20,  30,  30,  30,  20,  20,
    10,  10,  10,  20,  20,  10,  10,  10,
     5,   5,  10,  20,  20,   5,   5,   5,
     0,   0,   0,   5,   5,   0,   0,   0,
     0,   0,   0, -10, -10,   0,   0,   0,
     0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
pub static KNIGHT_POS_SCORE: [i32; 64] = [
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5,   0,   0,  10,  10,   0,   0,  -5,
    -5,   5,  20,  20,  20,  20,   5,  -5,
    -5,  10,  20,  30,  30,  20,  10,  -5,
    -5,  10,  20,  30,  30,  20,  10,  -5,
    -5,   5,  20,  10,  10,  20,   5,  -5,
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5, -10,   0,   0,   0,   0, -10,  -5,
];

#[rustfmt::skip]
pub static BISHOP_POS_SCORE: [i32; 64] = [
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   0,   0,  10,  10,   0,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,  10,   0,   0,   0,   0,  10,   0,
     0,  30,   0,   0,   0,   0,  30,   0,
     0,   0, -10,   0,   0, -10,   0,   0,
];

#[rustfmt::skip]
pub static ROOK_POS_SCORE: [i32; 64] = [
    50,  50,  50,  50,  50,  50,  50,  50,
    50,  50,  50,  50,  50,  50,  50,  50,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,   0,  20,  20,   0,   0,   0,
];

#[rustfmt::skip]
pub static KING_POS_SCORE: [i32; 64] = [
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   0,   5,   5,   5,   5,   0,   0,
     0,   5,   5,  10,  10,   5,   5,   0,
     0,   5,  10,  20,  20,  10,   5,   0,
     0,   5,  10,  20,  20,  10,   5,   0,
     0,   0,   5,  10,  10,   5,   0,   0,
     0,   5,   5,  -5,  -5,   0,   5,   0,
     0,   0,   5,   0, -15,   0,  10,   0,
];

#[rustfmt::skip]
static BISHOP_RELEVANT_OCCUPANCIES: [u32; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6,
    5, 5, 5, 5, 5, 5, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 7, 7, 7, 5, 5,
    5, 5, 5, 5, 5, 5, 5, 5,
    6, 5, 5, 5, 5, 5, 5, 6,
];

/// Magic numbers used for bishop magic bitboard indexing.
static BISHOP_MAGIC_BITBOARDS: [BitBoard; 64] = [
    BitBoard::new(0x40040844404084),
    BitBoard::new(0x2004208A004208),
    BitBoard::new(0x10190041080202),
    BitBoard::new(0x108060845042010),
    BitBoard::new(0x581104180800210),
    BitBoard::new(0x2112080446200010),
    BitBoard::new(0x1080820820060210),
    BitBoard::new(0x3C0808410220200),
    BitBoard::new(0x4050404440404),
    BitBoard::new(0x21001420088),
    BitBoard::new(0x24D0080801082102),
    BitBoard::new(0x1020A0A020400),
    BitBoard::new(0x40308200402),
    BitBoard::new(0x4011002100800),
    BitBoard::new(0x401484104104005),
    BitBoard::new(0x801010402020200),
    BitBoard::new(0x400210C3880100),
    BitBoard::new(0x404022024108200),
    BitBoard::new(0x810018200204102),
    BitBoard::new(0x4002801A02003),
    BitBoard::new(0x85040820080400),
    BitBoard::new(0x810102C808880400),
    BitBoard::new(0xE900410884800),
    BitBoard::new(0x8002020480840102),
    BitBoard::new(0x220200865090201),
    BitBoard::new(0x2010100A02021202),
    BitBoard::new(0x152048408022401),
    BitBoard::new(0x20080002081110),
    BitBoard::new(0x4001001021004000),
    BitBoard::new(0x800040400A011002),
    BitBoard::new(0xE4004081011002),
    BitBoard::new(0x1C004001012080),
    BitBoard::new(0x8004200962A00220),
    BitBoard::new(0x8422100208500202),
    BitBoard::new(0x2000402200300C08),
    BitBoard::new(0x8646020080080080),
    BitBoard::new(0x80020A0200100808),
    BitBoard::new(0x2010004880111000),
    BitBoard::new(0x623000A080011400),
    BitBoard::new(0x42008C0340209202),
    BitBoard::new(0x209188240001000),
    BitBoard::new(0x400408A884001800),
    BitBoard::new(0x110400A6080400),
    BitBoard::new(0x1840060A44020800),
    BitBoard::new(0x90080104000041),
    BitBoard::new(0x201011000808101),
    BitBoard::new(0x1A2208080504F080),
    BitBoard::new(0x8012020600211212),
    BitBoard::new(0x500861011240000),
    BitBoard::new(0x180806108200800),
    BitBoard::new(0x4000020E01040044),
    BitBoard::new(0x300000261044000A),
    BitBoard::new(0x802241102020002),
    BitBoard::new(0x20906061210001),
    BitBoard::new(0x5A84841004010310),
    BitBoard::new(0x4010801011C04),
    BitBoard::new(0xA010109502200),
    BitBoard::new(0x4A02012000),
    BitBoard::new(0x500201010098B028),
    BitBoard::new(0x8040002811040900),
    BitBoard::new(0x28000010020204),
    BitBoard::new(0x6000020202D0240),
    BitBoard::new(0x8918844842082200),
    BitBoard::new(0x4010011029020020),
];

#[rustfmt::skip]
static ROOK_RELEVANT_OCCUPANCIES: [u32; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12,
];

static ROOK_MAGIC_BITBOARDS: [BitBoard; 64] = [
    BitBoard::new(0x8A80104000800020),
    BitBoard::new(0x140002000100040),
    BitBoard::new(0x2801880A0017001),
    BitBoard::new(0x100081001000420),
    BitBoard::new(0x200020010080420),
    BitBoard::new(0x3001C0002010008),
    BitBoard::new(0x8480008002000100),
    BitBoard::new(0x2080088004402900),
    BitBoard::new(0x800098204000),
    BitBoard::new(0x2024401000200040),
    BitBoard::new(0x100802000801000),
    BitBoard::new(0x120800800801000),
    BitBoard::new(0x208808088000400),
    BitBoard::new(0x2802200800400),
    BitBoard::new(0x2200800100020080),
    BitBoard::new(0x801000060821100),
    BitBoard::new(0x80044006422000),
    BitBoard::new(0x100808020004000),
    BitBoard::new(0x12108A0010204200),
    BitBoard::new(0x140848010000802),
    BitBoard::new(0x481828014002800),
    BitBoard::new(0x8094004002004100),
    BitBoard::new(0x4010040010010802),
    BitBoard::new(0x20008806104),
    BitBoard::new(0x100400080208000),
    BitBoard::new(0x2040002120081000),
    BitBoard::new(0x21200680100081),
    BitBoard::new(0x20100080080080),
    BitBoard::new(0x2000A00200410),
    BitBoard::new(0x20080800400),
    BitBoard::new(0x80088400100102),
    BitBoard::new(0x80004600042881),
    BitBoard::new(0x4040008040800020),
    BitBoard::new(0x440003000200801),
    BitBoard::new(0x4200011004500),
    BitBoard::new(0x188020010100100),
    BitBoard::new(0x14800401802800),
    BitBoard::new(0x2080040080800200),
    BitBoard::new(0x124080204001001),
    BitBoard::new(0x200046502000484),
    BitBoard::new(0x480400080088020),
    BitBoard::new(0x1000422010034000),
    BitBoard::new(0x30200100110040),
    BitBoard::new(0x100021010009),
    BitBoard::new(0x2002080100110004),
    BitBoard::new(0x202008004008002),
    BitBoard::new(0x20020004010100),
    BitBoard::new(0x2048440040820001),
    BitBoard::new(0x101002200408200),
    BitBoard::new(0x40802000401080),
    BitBoard::new(0x4008142004410100),
    BitBoard::new(0x2060820C0120200),
    BitBoard::new(0x1001004080100),
    BitBoard::new(0x20C020080040080),
    BitBoard::new(0x2935610830022400),
    BitBoard::new(0x44440041009200),
    BitBoard::new(0x280001040802101),
    BitBoard::new(0x2100190040002085),
    BitBoard::new(0x80C0084100102001),
    BitBoard::new(0x4024081001000421),
    BitBoard::new(0x20030A0244872),
    BitBoard::new(0x12001008414402),
    BitBoard::new(0x2006104900A0804),
    BitBoard::new(0x1004081002402),
];

pub fn init_attack_tables() {
    init_leaper_piece_attacks();
    init_slider_piece_attacks(SliderPieceKind::Bishop);
    init_slider_piece_attacks(SliderPieceKind::Rook);
}

fn init_leaper_piece_attacks() {
    let mut pawn_attacks = [[BitBoard::default(); 64]; 2];
    let mut knight_attacks = [BitBoard::default(); 64];
    let mut king_attacks = [BitBoard::default(); 64];

    for square in 0..64 {
        let square = Square::from_u64_unchecked(square);

        pawn_attacks[Side::White][square] = compute_pawn_attacks(Side::White, square);
        pawn_attacks[Side::Black][square] = compute_pawn_attacks(Side::Black, square);
        knight_attacks[square] = compute_knight_attacks(square);
        king_attacks[square] = compute_king_attacks(square);
    }

    PAWN_ATTACKS.get_or_init(|| pawn_attacks);
    KNIGHT_ATTACKS.get_or_init(|| knight_attacks);
    KING_ATTACKS.get_or_init(|| king_attacks);
}

fn init_slider_piece_attacks(kind: SliderPieceKind) {
    let mut bishop_blockers = [BitBoard::default(); 64];
    let mut rook_blockers = [BitBoard::default(); 64];

    let mut bishop_attacks = vec![[BitBoard::default(); 512]; 64].into_boxed_slice();
    let mut rook_attacks = vec![[BitBoard::default(); 4096]; 64].into_boxed_slice();

    for index in 0..64 {
        let square = Square::from_u64_unchecked(index);
        bishop_blockers[index as usize] = compute_bishop_blockers(square);
        rook_blockers[index as usize] = compute_rook_blockers(square);

        let blockers = match kind {
            SliderPieceKind::Bishop => bishop_blockers[index as usize],
            SliderPieceKind::Rook => rook_blockers[index as usize],
        };

        let relevant_bits = blockers.count_ones();
        let occupancy_variations = 1 << relevant_bits;

        for occ_idx in 0..occupancy_variations {
            let occupancy = set_occupancy(occ_idx, relevant_bits, blockers);

            let magic_index = match kind {
                SliderPieceKind::Bishop => {
                    let magic = occupancy * BISHOP_MAGIC_BITBOARDS[index as usize];
                    let shift = 64 - BISHOP_RELEVANT_OCCUPANCIES[index as usize] as u64;
                    magic >> shift
                }
                SliderPieceKind::Rook => {
                    let magic = occupancy * ROOK_MAGIC_BITBOARDS[index as usize];
                    let shift = 64 - ROOK_RELEVANT_OCCUPANCIES[index as usize] as u64;
                    magic >> shift
                }
            };

            match kind {
                SliderPieceKind::Bishop => {
                    bishop_attacks[square as usize][*magic_index as usize] =
                        compute_bishop_attacks(square, occupancy);
                }
                SliderPieceKind::Rook => {
                    rook_attacks[square as usize][*magic_index as usize] =
                        compute_rook_attacks(square, occupancy);
                }
            }
        }
    }

    match kind {
        SliderPieceKind::Bishop => {
            BISHOP_BLOCKERS.get_or_init(|| bishop_blockers);
            BISHOP_ATTACKS.get_or_init(|| bishop_attacks);
        }
        SliderPieceKind::Rook => {
            ROOK_BLOCKERS.get_or_init(|| rook_blockers);
            ROOK_ATTACKS.get_or_init(|| rook_attacks);
        }
    }
}

fn compute_pawn_attacks(side: Side, square: Square) -> BitBoard {
    let bitboard = BitBoard::from_square(square);

    match side {
        Side::White => ((bitboard >> 7) & EMPTY_A_FILE) | ((bitboard >> 9) & EMPTY_H_FILE),
        Side::Black => ((bitboard << 7) & EMPTY_H_FILE) | ((bitboard << 9) & EMPTY_A_FILE),
        _ => unreachable!(),
    }
}

fn compute_knight_attacks(square: Square) -> BitBoard {
    let mut attacks = BitBoard::default();
    let bitboard = BitBoard::from_square(square);

    attacks |= (bitboard >> 17) & EMPTY_H_FILE; // two up, one left (north north west)
    attacks |= (bitboard >> 15) & EMPTY_A_FILE; // two up, one right (north north east)
    attacks |= (bitboard >> 10) & EMPTY_GH_FILE; // one up, two left (west nort west)
    attacks |= (bitboard >> 6) & EMPTY_AB_FILE; // one up, two right (east north east)
    attacks |= (bitboard << 17) & EMPTY_A_FILE; // one down, two left (west south west)
    attacks |= (bitboard << 15) & EMPTY_H_FILE; // one down, two right (east south east)
    attacks |= (bitboard << 10) & EMPTY_AB_FILE; // two down, one left (south south west)
    attacks |= (bitboard << 6) & EMPTY_GH_FILE; // two down, one right (south south east)

    attacks
}

fn compute_king_attacks(square: Square) -> BitBoard {
    let mut attacks = BitBoard::default();
    let bitboard = BitBoard::from_square(square);

    attacks |= (bitboard >> 7) & EMPTY_A_FILE;
    attacks |= bitboard >> 8;
    attacks |= (bitboard >> 9) & EMPTY_H_FILE;
    attacks |= (bitboard << 7) & EMPTY_H_FILE;
    attacks |= bitboard << 8;
    attacks |= (bitboard << 9) & EMPTY_A_FILE;
    attacks |= bitboard << 1 & EMPTY_A_FILE;
    attacks |= bitboard >> 1 & EMPTY_H_FILE;

    attacks
}

fn compute_bishop_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let mut attacks = BitBoard::default();

    let directions = [
        (1, 1),   // NE
        (-1, 1),  // SE
        (1, -1),  // NW
        (-1, -1), // SW
    ];

    let rank = square as i8 / 8;
    let file = square as i8 % 8;

    for (rank_dir, file_dir) in directions {
        let mut r = rank + rank_dir;
        let mut f = file + file_dir;

        while (0..8).contains(&r) && (0..8).contains(&f) {
            let index = (r * 8 + f) as u64;
            let square = Square::from_u64_unchecked(index);
            attacks.set_bit(square);

            if !(BitBoard::from_square(square) & blockers).is_empty() {
                break;
            }

            r += rank_dir;
            f += file_dir;
        }
    }

    attacks
}

fn compute_rook_attacks(square: Square, blockers: BitBoard) -> BitBoard {
    let mut attacks = BitBoard::default();

    let directions = [
        (0, 1),  // N
        (1, 0),  // E
        (-1, 0), // W
        (0, -1), // S
    ];

    let rank = square as i8 / 8;
    let file = square as i8 % 8;

    for (rank_dir, file_dir) in directions {
        let mut r = rank + rank_dir;
        let mut f = file + file_dir;

        while (0..8).contains(&r) && (0..8).contains(&f) {
            let index = (r * 8 + f) as u64;
            let square = Square::from_u64_unchecked(index);
            attacks.set_bit(square);

            if !(BitBoard::from_square(square) & blockers).is_empty() {
                break;
            }

            r += rank_dir;
            f += file_dir;
        }
    }

    attacks
}

fn compute_bishop_blockers(square: Square) -> BitBoard {
    let mut blockers = BitBoard::default();

    let directions = [
        (1, 1),   // NE
        (-1, 1),  // SE
        (1, -1),  // NW
        (-1, -1), // SW
    ];

    let rank = square as i8 / 8;
    let file = square as i8 % 8;

    for (rank_dir, file_dir) in directions {
        let mut r = rank + rank_dir;
        let mut f = file + file_dir;

        while (1..7).contains(&r) && (1..7).contains(&f) {
            let index = (r * 8 + f) as u64;
            blockers.set_bit(Square::from_u64_unchecked(index));
            r += rank_dir;
            f += file_dir;
        }
    }

    blockers
}

fn compute_rook_blockers(square: Square) -> BitBoard {
    let mut blockers = BitBoard::default();

    let directions = [
        (0, 1),  // N
        (1, 0),  // E
        (-1, 0), // W
        (0, -1), // S
    ];

    let rank = square as i8 / 8;
    let file = square as i8 % 8;

    for (rank_dir, file_dir) in directions {
        let mut r = rank + rank_dir;
        let mut f = file + file_dir;

        while (0..8).contains(&r) && (0..8).contains(&f) {
            // if either direction moved at least once, skip the edges this ensure we generate
            // the moves for cornered rooks
            if (rank_dir != 0 && (r == 0 || r == 7)) || (file_dir != 0 && (f == 0 || f == 7)) {
                break;
            }
            let index = (r * 8 + f) as u64;
            blockers.set_bit(Square::from_u64_unchecked(index));
            r += rank_dir;
            f += file_dir;
        }
    }

    blockers
}

fn set_occupancy(index: usize, bits_in_mask: u32, mut attackers: BitBoard) -> BitBoard {
    let mut occupancy = BitBoard::default();

    for count in 0..bits_in_mask {
        let square = attackers.trailing_zeros();
        attackers.clear_bit(square);

        if index & (1 << count) != 0 {
            occupancy.set_bit(square);
        }
    }

    occupancy
}

fn get_bishop_attacks(square: Square, mut occupancy: BitBoard) -> BitBoard {
    occupancy &= BISHOP_BLOCKERS.get().unwrap()[square];
    occupancy *= BISHOP_MAGIC_BITBOARDS[square];
    occupancy >>= (64 - BISHOP_RELEVANT_OCCUPANCIES[square as usize]) as u64;

    attacks!(BISHOP_ATTACKS)[square as usize][*occupancy as usize]
}

fn get_rook_attacks(square: Square, mut occupancy: BitBoard) -> BitBoard {
    occupancy &= ROOK_BLOCKERS.get().unwrap()[square];
    occupancy *= ROOK_MAGIC_BITBOARDS[square];
    occupancy >>= (64 - ROOK_RELEVANT_OCCUPANCIES[square as usize]) as u64;

    attacks!(ROOK_ATTACKS)[square as usize][*occupancy as usize]
}

fn get_queen_attacks(square: Square, occupancy: BitBoard) -> BitBoard {
    let bishop_occupancies = occupancy;
    let rook_occupancies = occupancy;

    let mut queen_attacks = get_bishop_attacks(square, bishop_occupancies);
    queen_attacks |= get_rook_attacks(square, rook_occupancies);

    queen_attacks
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum SliderPieceKind {
    Rook,
    Bishop,
}

#[derive(Debug)]
pub struct BoardSnapshot {
    pub boards: [BitBoard; 12],
    pub occupancies: [BitBoard; 3],
    pub side_to_move: Side,
    pub en_passant: Square,
    pub castling_rights: CastlingRights,
    pub position_key: ZobristKey,
}

impl Default for BoardSnapshot {
    fn default() -> Self {
        Self {
            boards: [BitBoard::default(); 12],
            occupancies: [BitBoard::default(); 3],
            side_to_move: Side::White,
            en_passant: Square::OffBoard,
            castling_rights: CastlingRights::all(),
            position_key: ZobristKey::default(),
        }
    }
}

#[derive(Debug)]
pub struct Milky {
    rng: Random,
    pub boards: [BitBoard; 12],
    pub occupancies: [BitBoard; 3],
    pub side_to_move: Side,
    pub en_passant: Square,
    pub castling_rights: CastlingRights,
    pub ply: usize,
    pub moves: [Move; 256],
    pub move_count: usize,
    pub snapshots: Vec<BoardSnapshot>,
    pub nodes: u64,

    pub killer_moves: [[Move; 64]; 2],
    pub history_moves: [[i32; 64]; 12],

    pub pv_table: [[Move; MAX_PLY]; MAX_PLY],
    pub pv_length: [usize; MAX_PLY],

    pub score_pv: bool,
    pub follow_pv: bool,

    pub zobrist: Zobrist,
    pub transposition_table: TranspositionTable,

    pub repetition_table: [ZobristKey; MAX_REPETITIONS],
    pub repetition_index: usize,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum MoveKind {
    AllMoves,
    Captures,
}

impl Default for Milky {
    fn default() -> Self {
        Self::new()
    }
}

impl Milky {
    pub fn new() -> Self {
        Self {
            rng: Random::default(),
            boards: [BitBoard::empty(); 12],
            occupancies: [BitBoard::empty(); 3],
            side_to_move: Side::White,
            en_passant: Square::OffBoard,
            castling_rights: CastlingRights::all(),
            snapshots: Vec::new(),
            moves: [Move::default(); 256],
            move_count: 0,
            ply: 0,
            nodes: 0,
            killer_moves: [[Move::default(); 64]; 2],
            history_moves: [[0; 64]; 12],

            pv_table: [[Move::default(); MAX_PLY]; MAX_PLY],
            pv_length: [0; MAX_PLY],
            score_pv: false,
            follow_pv: false,
            zobrist: Zobrist::new(),
            transposition_table: TranspositionTable::new(),
            repetition_table: [ZobristKey::default(); MAX_REPETITIONS],
            repetition_index: 0,
        }
    }

    fn push_move(&mut self, piece_move: Move) {
        self.moves[self.move_count] = piece_move;
        self.move_count += 1;
    }

    pub fn sort_moves(&mut self) {
        let mut scored_moves: Vec<(i32, Move)> = self
            .moves
            .into_iter()
            .take(self.move_count)
            .map(|m| (self.score_move(m), m))
            .collect();

        scored_moves.sort_by_key(|&(score, _)| std::cmp::Reverse(score));

        scored_moves
            .into_iter()
            .enumerate()
            .for_each(|(idx, (_, m))| self.moves[idx] = m);
    }

    fn enable_pv_scoring(&mut self) {
        self.follow_pv = false;

        for piece_move in self.moves.into_iter().take(self.move_count) {
            if self.pv_table[0][self.ply] == piece_move {
                self.score_pv = true;
                self.follow_pv = true;
            }
        }
    }

    /// Scores a move based on the following heuristics:
    ///
    /// - PV move
    /// - Captures in MVV/LVA
    /// - 1st killer move
    /// - 2nd killer move
    /// - History moves
    /// - Unsorted moves
    pub fn score_move(&mut self, piece_move: Move) -> i32 {
        const PV_MOVE_SCORE: i32 = 20_000;
        const MVV_LVA_BONUS: i32 = 10_000;
        const FIRST_KILLER_MOVE: i32 = 9_000;
        const SECOND_KILLER_MOVE: i32 = 8_000;

        if self.score_pv && self.pv_table[0][self.ply] == piece_move {
            self.score_pv = false;
            return PV_MOVE_SCORE;
        }

        if piece_move.is_capture() {
            let attacker = piece_move.piece();
            let victim_square = piece_move.target();

            let (start_piece, end_piece) = if self.side_to_move == Side::White {
                (Pieces::BlackPawn, Pieces::BlackKing)
            } else {
                (Pieces::WhitePawn, Pieces::WhiteKing)
            };

            // Victim is initialized as white pawn to make en-passant moves easier.
            //
            // Since side doesn't matter for en-passant, even when white is the attacker, white
            // pawn takes white pawn have the same score as white capturing black.
            let victim = (start_piece as usize..=end_piece as usize)
                .find(|&idx| self.boards[idx].get_bit(victim_square).is_set())
                .map(Pieces::from_usize_unchecked)
                .unwrap_or(Pieces::WhitePawn);

            return MVV_LVA[attacker][victim] + MVV_LVA_BONUS;
        }

        if self.killer_moves[0][self.ply] == piece_move {
            FIRST_KILLER_MOVE
        } else if self.killer_moves[1][self.ply] == piece_move {
            SECOND_KILLER_MOVE
        } else {
            self.history_moves[piece_move.piece()][piece_move.target()]
        }
    }

    pub fn record_repetition(&mut self) {
        self.repetition_index += 1;
        self.repetition_table[self.repetition_index] = self.zobrist.position;
    }

    fn quiescence(&mut self, mut alpha: Wrapping<i32>, beta: Wrapping<i32>) -> i32 {
        self.nodes += 1;

        if self.ply > MAX_PLY - 1 {
            return evaluate_position(self.side_to_move, self.boards);
        }

        let evaluation = evaluate_position(self.side_to_move, self.boards);

        if evaluation >= beta.0 {
            return beta.0;
        }

        if evaluation > alpha.0 {
            alpha = Wrapping(evaluation);
        }

        self.generate_moves();
        self.sort_moves();

        for piece_move in self.moves.into_iter().take(self.move_count) {
            self.ply += 1;
            self.record_repetition();

            if !self.make_move(piece_move, MoveKind::Captures) {
                self.ply -= 1;
                self.repetition_index -= 1;
                continue;
            }

            let score = -Wrapping(self.quiescence(-beta, -alpha));

            self.ply -= 1;
            self.repetition_index -= 1;

            self.undo_move();

            if score > alpha {
                alpha = score;

                if score >= beta {
                    return beta.0;
                }
            }
        }

        alpha.0
    }

    fn is_repetition(&self) -> bool {
        self.repetition_table[0..self.repetition_index].contains(&self.zobrist.position)
    }

    fn negamax(&mut self, mut alpha: Wrapping<i32>, beta: Wrapping<i32>, mut depth: u8) -> i32 {
        const FULL_DEPTH_MOVES: i32 = 4;
        const REDUCTION_LIMIT: u8 = 3;

        let mut tt_flag = TTFlag::Alpha;

        if self.ply != 0 && self.is_repetition() {
            return 0;
        }

        let pv_node = beta.0 - alpha.0 > 1;

        let score =
            self.transposition_table
                .get(self.zobrist.position, alpha.0, beta.0, depth, self.ply);

        if let (Some(score), true, true) = (score, self.ply != 0, !pv_node) {
            return score;
        }

        self.pv_length[self.ply] = self.ply;

        if depth == 0 {
            return self.quiescence(alpha, beta);
        }

        if self.ply > MAX_PLY - 1 {
            return evaluate_position(self.side_to_move, self.boards);
        }

        self.nodes += 1;

        let king_square = match self.side_to_move {
            Side::White => self.boards[Pieces::WhiteKing].trailing_zeros(),
            Side::Black => self.boards[Pieces::BlackKing].trailing_zeros(),
            _ => unreachable!(),
        };

        let in_check = self.is_square_attacked(king_square, self.side_to_move.enemy());
        if in_check {
            // Extend the search depth if in check, this is useful to find forced mates or tactical
            // defenses in dangerous positions
            depth += 1;
        }

        if depth >= REDUCTION_LIMIT && !in_check && self.ply != 0 {
            self.snapshot_board();

            self.ply += 1;
            self.record_repetition();

            if self.en_passant.is_available() {
                self.zobrist.position ^= self.zobrist.en_passant[self.en_passant];
            }

            self.en_passant = Square::OffBoard;
            self.side_to_move = self.side_to_move.enemy();

            self.zobrist.position ^= self.zobrist.side_key;

            let score = -Wrapping(self.negamax(-beta, -beta + Wrapping(1), depth - 1 - 2));
            self.ply -= 1;
            self.repetition_index -= 1;
            self.undo_move();

            if score >= beta {
                return beta.0;
            }
        }

        self.generate_moves();

        // If move is within the PV path from the previous iteration, give it a small bonus to
        // improve its position in ordering.
        //
        // This is making the assumption that if we already have a PV, following its path is more
        // likely to have better results.
        if self.follow_pv {
            self.enable_pv_scoring();
        }

        // Order moves by MVV-LVA score to improve pruning efficiency
        self.sort_moves();

        let mut legal_moves = 0;
        let mut moves_searched = 0;

        for piece_move in self.moves.into_iter().take(self.move_count) {
            self.ply += 1;
            self.record_repetition();

            if !self.make_move(piece_move, MoveKind::AllMoves) {
                self.ply -= 1;
                self.repetition_index -= 1;
                continue;
            }

            legal_moves += 1;

            let score = if moves_searched == 0 {
                -Wrapping(self.negamax(-beta, -alpha, depth - 1))
            } else {
                // To apply late move reduction, a move cannot be a capture or a promotion, the
                // king must not be in check and the search must also be past the depth allowed to
                // be reduced
                let should_reduce = moves_searched >= FULL_DEPTH_MOVES
                    && depth >= REDUCTION_LIMIT
                    && !in_check
                    && !piece_move.is_capture()
                    && !piece_move.promotion().is_promoting();

                // Apply late move reduction by reducing the depth by 2 per ply
                let shallow = if should_reduce {
                    -Wrapping(self.negamax(-alpha - Wrapping(1), -alpha, depth - 2))
                } else {
                    // This move should not yet reduce, but we are also on a non-pv path, so
                    // instead of going down the search, we give it a fake score slightly above
                    // alpha that ensures it will trigger the full search below.
                    alpha + Wrapping(1)
                };

                if shallow > alpha {
                    // LMR found a better move, so we search at full depth but with a narrower
                    // window to double check if it is a better move.
                    let deeper = -Wrapping(self.negamax(-alpha - Wrapping(1), -alpha, depth - 1));

                    // If the narrower window also proves to improve alpha, we do a final full
                    // depth and full width window search.
                    if deeper > alpha && deeper < beta {
                        -Wrapping(self.negamax(-beta, -alpha, depth - 1))
                    } else {
                        deeper
                    }
                } else {
                    shallow
                }
            };

            self.ply -= 1;
            self.repetition_index -= 1;
            self.undo_move();
            moves_searched += 1;

            // Alpha raise
            //
            // The move is better than alpha and smaller than beta, which means it is an
            // improvement on our previously found primary variance and we want to update our
            // primary variance table
            if score > alpha {
                // since we found an exact score, update the flag used at the end
                tt_flag = TTFlag::Exact;

                // History heuristic
                //
                // Keep track of quiet moves that increases alpha by giving them a bonus based on
                // its depth, this put those moves higher on the move sorting
                if !piece_move.is_capture() {
                    self.history_moves[piece_move.piece()][piece_move.target()] += depth as i32;
                }

                alpha = score;

                // Principal variation bookkeeping, the current move is the new best move, so we
                // update the PV table at the current depth to store this move, and copy all the
                // other PV nodes from the deeper ply
                self.pv_table[self.ply][self.ply] = piece_move;
                for next_ply in self.ply + 1..self.pv_length[self.ply + 1] {
                    self.pv_table[self.ply][next_ply] = self.pv_table[self.ply + 1][next_ply];
                }

                self.pv_length[self.ply] = self.pv_length[self.ply + 1];

                // Beta cutoff
                //
                // If the current move is so good it exceeds beta, there is no need to search its
                // siblings, as this move is so good the opponent would never allow it to happen.
                //
                // This is a fail-hard alpha/beta search
                if score >= beta {
                    self.transposition_table.set(
                        self.zobrist.position,
                        depth,
                        beta.0,
                        TTFlag::Beta,
                        self.ply,
                    );

                    if !piece_move.is_capture() {
                        // When a non-capture (killer move) causes a beta cutoff, we store keep track of
                        // them in order to give them a higher priority in searching when there's a
                        // similar position.
                        self.killer_moves[1][self.ply] = self.killer_moves[0][self.ply];
                        self.killer_moves[0][self.ply] = piece_move;
                    }

                    return beta.0;
                }
            }
        }

        if legal_moves == 0 {
            if in_check {
                return -MATE_UPPER_BOUND + self.ply as i32;
            } else {
                return 0;
            }
        }

        self.transposition_table
            .set(self.zobrist.position, depth, alpha.0, tt_flag, self.ply);

        alpha.0
    }

    pub fn search_position(&mut self, depth: u8) -> Move {
        const ASPIRATION_WINDOW: i32 = 50;

        self.nodes = 0;
        self.follow_pv = false;
        self.score_pv = false;

        self.killer_moves = [[Move::default(); 64]; 2];
        self.history_moves = [[0; 64]; 12];
        self.pv_table = [[Move::default(); MAX_PLY]; MAX_PLY];
        self.pv_length = [0; MAX_PLY];

        let mut alpha = Wrapping(-INFINITY);
        let mut beta = Wrapping(INFINITY);

        for curr_depth in 1..=depth {
            self.follow_pv = true;

            let score = self.negamax(alpha, beta, curr_depth);
            if score <= alpha.0 || score >= beta.0 {
                alpha = Wrapping(-INFINITY);
                beta = Wrapping(INFINITY);
                continue;
            }

            alpha = Wrapping(score - ASPIRATION_WINDOW);
            beta = Wrapping(score + ASPIRATION_WINDOW);

            if score > -MATE_UPPER_BOUND && score < -MATE_LOWER_BOUND {
                print!(
                    "info score mate {} depth {curr_depth} nodes {} pv ",
                    -(score + MATE_UPPER_BOUND) / 2 - 1,
                    self.nodes,
                )
            } else if score > MATE_LOWER_BOUND && score < MATE_UPPER_BOUND {
                print!(
                    "info score mate {} depth {curr_depth} nodes {} pv ",
                    (MATE_UPPER_BOUND - score) / 2 + 1,
                    self.nodes,
                )
            } else {
                print!(
                    "info score cp {score} depth {curr_depth} nodes {} pv ",
                    self.nodes
                );
            }

            for idx in 0..self.pv_length[0] {
                print!("{} ", self.pv_table[0][idx]);
            }

            println!();
        }

        println!("bestmove {}", self.pv_table[0][0]);

        self.pv_table[0][0]
    }

    pub fn load_fen(&mut self, fen_parts: FenParts) {
        let occupancies = [
            fen_parts.white_occupancy,
            fen_parts.black_occupancy,
            fen_parts.both_occupancy,
        ];

        self.boards = fen_parts.positions;
        self.occupancies = occupancies;
        self.en_passant = fen_parts.en_passant;
        self.side_to_move = fen_parts.side_to_move;
        self.castling_rights = fen_parts.castling_rights;
        self.zobrist.position = self.zobrist.hash_position(GamePosition {
            boards: self.boards,
            side_to_move: self.side_to_move,
            en_passant: self.en_passant,
            castling_rights: self.castling_rights,
        })
    }

    pub fn print_board(&self) {
        println!();

        for rank in 0..8 {
            let mut line = String::with_capacity(20);
            line.push_str(&format!("  {} ", 8 - rank));

            for file in 0..8 {
                let square = Square::from_u64_unchecked(rank * 8 + file);
                let mut piece = String::from(".");

                for (idx, &board) in self.boards.iter().enumerate() {
                    if !board.get_bit(square).is_empty() {
                        piece = Pieces::from_usize_unchecked(idx).to_string();
                        break;
                    }
                }

                line.push(' ');
                line.push_str(&piece);
            }

            println!("{line}");
        }

        println!();
        println!("     a b c d e f g h");
        println!();
        println!("     Side:      {}", self.side_to_move);
        println!("     Castling:   {}", self.castling_rights);
        println!("     Enpassant:    {}", self.en_passant);
        println!("     Zobrist key: {}", self.zobrist.position);
        println!();
    }

    #[allow(dead_code)]
    fn init_magic_numbers(&mut self) {
        for square in 0..64 {
            let square = Square::from_u64_unchecked(square);
            let rook_magic = self.find_magic_number(
                square,
                ROOK_RELEVANT_OCCUPANCIES[square as usize],
                SliderPieceKind::Rook,
            );
            println!("0x{rook_magic:X},");
        }

        println!();
        println!();

        for square in 0..64 {
            let square = Square::from_u64_unchecked(square);
            let bishop_magic = self.find_magic_number(
                square,
                BISHOP_RELEVANT_OCCUPANCIES[square as usize],
                SliderPieceKind::Bishop,
            );
            println!("0x{bishop_magic:X},");
        }
    }

    #[allow(dead_code)]
    fn find_magic_number(
        &mut self,
        square: Square,
        relevant_bits: u32,
        kind: SliderPieceKind,
    ) -> u64 {
        let mut occupancies = [BitBoard::default(); 4096];
        let mut attacks = [BitBoard::default(); 4096];
        let mut used_attacks = [BitBoard::default(); 4096];

        let blockers = match kind {
            SliderPieceKind::Rook => compute_rook_blockers(square),
            SliderPieceKind::Bishop => compute_bishop_blockers(square),
        };

        let occupancy_idx = 1 << relevant_bits;

        for index in 0..occupancy_idx {
            occupancies[index] = set_occupancy(index, relevant_bits, blockers);

            attacks[index] = match kind {
                SliderPieceKind::Rook => compute_rook_attacks(square, occupancies[index]),
                SliderPieceKind::Bishop => compute_bishop_attacks(square, occupancies[index]),
            }
        }

        'search: for _ in 0..100_000_000 {
            let magic_number = self.rng.gen_u64() & self.rng.gen_u64() & self.rng.gen_u64();
            if ((blockers.wrapping_mul(magic_number)) & 0xFF00_0000_0000_0000).count_ones() < 6 {
                continue;
            }

            for attack in used_attacks.iter_mut().take(occupancy_idx) {
                *attack = BitBoard::default()
            }

            for index in 0..occupancy_idx {
                let magic_index = ((occupancies[index].wrapping_mul(magic_number))
                    >> (64 - relevant_bits)) as usize;

                if used_attacks[magic_index].is_empty() {
                    used_attacks[magic_index] = attacks[index];
                } else if used_attacks[magic_index] != attacks[index] {
                    continue 'search;
                }
            }

            return magic_number;
        }

        0
    }

    #[inline]
    pub fn is_square_attacked(&self, square: Square, side: Side) -> bool {
        let (
            pawn_side,
            pawn_board,
            knight_board,
            king_board,
            bishop_board,
            rook_board,
            queen_board,
        ) = match side {
            Side::White => (
                Side::Black,
                self.boards[Pieces::WhitePawn],
                self.boards[Pieces::WhiteKnight],
                self.boards[Pieces::WhiteKing],
                self.boards[Pieces::WhiteBishop],
                self.boards[Pieces::WhiteRook],
                self.boards[Pieces::WhiteQueen],
            ),
            Side::Black => (
                Side::White,
                self.boards[Pieces::BlackPawn],
                self.boards[Pieces::BlackKnight],
                self.boards[Pieces::BlackKing],
                self.boards[Pieces::BlackBishop],
                self.boards[Pieces::BlackRook],
                self.boards[Pieces::BlackQueen],
            ),
            _ => unreachable!(),
        };

        if attacks!(PAWN_ATTACKS)[pawn_side][square].is_attacked(pawn_board) {
            return true;
        }

        if attacks!(KNIGHT_ATTACKS)[square].is_attacked(knight_board) {
            return true;
        }

        if attacks!(KING_ATTACKS)[square].is_attacked(king_board) {
            return true;
        }

        let occupancy = self.occupancies[Side::Both];

        if get_bishop_attacks(square, occupancy).is_attacked(bishop_board) {
            return true;
        }

        if get_rook_attacks(square, occupancy).is_attacked(rook_board) {
            return true;
        }

        if get_queen_attacks(square, occupancy).is_attacked(queen_board) {
            return true;
        }

        false
    }

    pub fn generate_moves(&mut self) {
        self.move_count = 0;
        for (idx, board) in self.boards.into_iter().enumerate() {
            let piece = Pieces::from_usize_unchecked(idx);

            if piece.side() != self.side_to_move {
                continue;
            }

            match piece {
                Pieces::WhitePawn | Pieces::BlackPawn => {
                    self.generate_pawn_moves(self.side_to_move, board, piece)
                }
                Pieces::WhiteKing | Pieces::BlackKing => {
                    self.generate_king_moves(self.side_to_move, board, piece)
                }
                Pieces::WhiteKnight | Pieces::BlackKnight => {
                    self.generate_knight_moves(self.side_to_move, board, piece)
                }
                Pieces::WhiteBishop | Pieces::BlackBishop => {
                    self.generate_bishop_moves(self.side_to_move, board, piece)
                }
                Pieces::WhiteRook | Pieces::BlackRook => {
                    self.generate_rook_moves(self.side_to_move, board, piece)
                }
                Pieces::WhiteQueen | Pieces::BlackQueen => {
                    self.generate_queen_moves(self.side_to_move, board, piece)
                }
            }
        }
    }

    fn generate_pawn_moves(&mut self, side: Side, board: BitBoard, piece: Pieces) {
        let promotion_rank = match side {
            Side::White => Rank::Seventh,
            Side::Black => Rank::Second,
            _ => unreachable!(),
        };

        let initial_rank = match side {
            Side::White => Rank::Second,
            Side::Black => Rank::Seventh,
            _ => unreachable!(),
        };

        let promotion_options = [
            PromotedPieces::Knight,
            PromotedPieces::Bishop,
            PromotedPieces::Rook,
            PromotedPieces::Queen,
        ];

        for square in board {
            let one_forward = match side {
                Side::White => square.one_forward(),
                Side::Black => square.one_backward(),
                _ => unreachable!(),
            };

            // Skip if the move would leave the board
            let Some(one_forward) = one_forward else {
                continue;
            };

            if self.occupancies[Side::Both].get_bit(one_forward).is_empty() {
                if square.is_on_rank(promotion_rank) {
                    for option in promotion_options {
                        self.push_move(Move::new(
                            square,
                            one_forward,
                            piece,
                            option,
                            MoveFlags::empty(),
                        ));
                    }
                } else {
                    self.push_move(Move::new(
                        square,
                        one_forward,
                        piece,
                        PromotedPieces::NoPromotion,
                        MoveFlags::empty(),
                    ));
                }

                if square.is_on_rank(initial_rank) {
                    // SAFETY: one_forward is valid (verified above)
                    let two_forward = match side {
                        Side::White => one_forward.one_forward().unwrap(),
                        Side::Black => one_forward.one_backward().unwrap(),
                        _ => unreachable!(),
                    };

                    if self.occupancies[Side::Both].get_bit(two_forward).is_empty() {
                        self.push_move(Move::new(
                            square,
                            two_forward,
                            piece,
                            PromotedPieces::NoPromotion,
                            MoveFlags::DOUBLE_PUSH,
                        ));
                    }
                }
            }

            let enemy_occupancies = self.occupancies[side.enemy()];
            let pawn_attacks = attacks!(PAWN_ATTACKS)[side][square];
            let attacks = pawn_attacks.attacked_squares(enemy_occupancies);

            for target in attacks {
                if square.is_on_rank(promotion_rank) {
                    for option in promotion_options {
                        self.push_move(Move::new(
                            square,
                            target,
                            piece,
                            option,
                            MoveFlags::CAPTURE,
                        ));
                    }
                } else {
                    self.push_move(Move::new(
                        square,
                        target,
                        piece,
                        PromotedPieces::NoPromotion,
                        MoveFlags::CAPTURE,
                    ));
                }
            }

            if self.en_passant != Square::OffBoard {
                let en_passant_attacks =
                    pawn_attacks.attacked_squares(BitBoard::from_square(self.en_passant));

                if en_passant_attacks.is_set() {
                    let target = en_passant_attacks.trailing_zeros();
                    self.push_move(Move::new(
                        square,
                        target,
                        piece,
                        PromotedPieces::NoPromotion,
                        MoveFlags::union(MoveFlags::EN_PASSANT, MoveFlags::CAPTURE),
                    ));
                }
            }
        }
    }

    fn generate_pre_computed_moves<F>(
        &mut self,
        side: Side,
        piece: Pieces,
        board: BitBoard,
        get_attacks: F,
    ) where
        F: Fn(Square) -> BitBoard,
    {
        for square in board {
            let attacks = get_attacks(square);
            let occupancies = !self.occupancies[side];
            let attacks = attacks.attacked_squares(occupancies);

            for target in attacks {
                let occupancies = self.occupancies[side.enemy()];

                if occupancies.get_bit(target).is_set() {
                    self.push_move(Move::new(
                        square,
                        target,
                        piece,
                        PromotedPieces::NoPromotion,
                        MoveFlags::CAPTURE,
                    ));
                } else {
                    self.push_move(Move::new(
                        square,
                        target,
                        piece,
                        PromotedPieces::NoPromotion,
                        MoveFlags::empty(),
                    ));
                }
            }
        }
    }

    fn generate_king_moves(&mut self, side: Side, board: BitBoard, piece: Pieces) {
        let king_side = match side {
            Side::White => CastlingRights::WHITE_K,
            Side::Black => CastlingRights::BLACK_K,
            _ => unreachable!(),
        };

        let queen_side = match side {
            Side::White => CastlingRights::WHITE_Q,
            Side::Black => CastlingRights::BLACK_Q,
            _ => unreachable!(),
        };

        let king_square = match side {
            Side::White => Square::E1,
            Side::Black => Square::E8,
            _ => unreachable!(),
        };

        // Check whether white king can castle to the king's side
        if self.castling_rights.contains(king_side) {
            let required_free_squares = match side {
                Side::White => (Square::F1, Square::G1),
                Side::Black => (Square::F8, Square::G8),
                _ => unreachable!(),
            };

            // When castling king's side, the squares between the king and king's rook must be
            // empty. That is, for white, squares f1 and g1, and for black, squares f8 and g8.
            let first = self.occupancies[Side::Both].get_bit(required_free_squares.0);
            let second = self.occupancies[Side::Both].get_bit(required_free_squares.1);

            // king cannot be in check and the square next to the king  cannot be attacked. That
            // is, for white, squares e1 and f1, and for black, squares e8 and f8.
            let is_king_attacked = self.is_square_attacked(king_square, side.enemy());
            let is_next_attacked = self.is_square_attacked(required_free_squares.0, side.enemy());

            if first.is_empty() && second.is_empty() && !is_king_attacked && !is_next_attacked {
                self.push_move(Move::new(
                    king_square,
                    required_free_squares.1,
                    piece,
                    PromotedPieces::NoPromotion,
                    MoveFlags::CASTLING,
                ))
            }
        }

        // Check whether white king can castle to the queen's side
        if self.castling_rights.contains(queen_side) {
            let required_free_squares = match side {
                Side::White => (Square::D1, Square::C1, Square::B1),
                Side::Black => (Square::D8, Square::C8, Square::B8),
                _ => unreachable!(),
            };

            // When castling queen's side, the squares between the king and queen's rook must be
            // empty. That is, for white, squares d1, c1 and b1, and for black, squares d8, c8 and
            // b8.
            let first = self.occupancies[Side::Both].get_bit(required_free_squares.0);
            let second = self.occupancies[Side::Both].get_bit(required_free_squares.1);
            let third = self.occupancies[Side::Both].get_bit(required_free_squares.2);

            // king cannot be in check and the square next to the king  cannot be attacked. That
            // is, for white, squares e1 and f1, and for black, squares e8 and f8.
            let is_king_attacked = self.is_square_attacked(king_square, side.enemy());
            let is_next_attacked = self.is_square_attacked(required_free_squares.0, side.enemy());

            if first.is_empty()
                && second.is_empty()
                && third.is_empty()
                && !is_king_attacked
                && !is_next_attacked
            {
                self.push_move(Move::new(
                    king_square,
                    required_free_squares.1,
                    piece,
                    PromotedPieces::NoPromotion,
                    MoveFlags::CASTLING,
                ))
            }
        }

        self.generate_pre_computed_moves(side, piece, board, |square| {
            attacks!(KING_ATTACKS)[square]
        });
    }

    fn generate_knight_moves(&mut self, side: Side, board: BitBoard, piece: Pieces) {
        self.generate_pre_computed_moves(side, piece, board, |square| {
            attacks!(KNIGHT_ATTACKS)[square]
        });
    }

    fn generate_bishop_moves(&mut self, side: Side, board: BitBoard, piece: Pieces) {
        let occupancies = self.occupancies[Side::Both];
        self.generate_pre_computed_moves(side, piece, board, |square| {
            get_bishop_attacks(square, occupancies)
        });
    }

    fn generate_rook_moves(&mut self, side: Side, board: BitBoard, piece: Pieces) {
        let occupancies = self.occupancies[Side::Both];
        self.generate_pre_computed_moves(side, piece, board, |square| {
            get_rook_attacks(square, occupancies)
        });
    }

    fn generate_queen_moves(&mut self, side: Side, board: BitBoard, piece: Pieces) {
        let occupancies = self.occupancies[Side::Both];
        self.generate_pre_computed_moves(side, piece, board, |square| {
            get_queen_attacks(square, occupancies)
        });
    }

    pub fn snapshot_board(&mut self) {
        self.snapshots.push(BoardSnapshot {
            boards: self.boards,
            occupancies: self.occupancies,
            side_to_move: self.side_to_move,
            en_passant: self.en_passant,
            castling_rights: self.castling_rights,
            position_key: self.zobrist.position,
        });
    }

    pub fn undo_move(&mut self) {
        if let Some(snapshot) = self.snapshots.pop() {
            self.boards = snapshot.boards;
            self.occupancies = snapshot.occupancies;
            self.side_to_move = snapshot.side_to_move;
            self.en_passant = snapshot.en_passant;
            self.castling_rights = snapshot.castling_rights;
            self.zobrist.position = snapshot.position_key;
        } else {
            panic!("Tried to undo_move with no snapshots on stack!");
        }
    }

    pub fn make_move(&mut self, piece_move: Move, move_kind: MoveKind) -> bool {
        match move_kind {
            MoveKind::Captures => {
                if piece_move.is_capture() {
                    self.make_move(piece_move, MoveKind::AllMoves)
                } else {
                    false
                }
            }
            MoveKind::AllMoves => {
                self.snapshot_board();

                let source = piece_move.source();
                let target = piece_move.target();
                let piece = piece_move.piece();

                self.boards[piece].clear_bit(source);
                self.boards[piece].set_bit(target);

                self.zobrist.position ^= self.zobrist.pieces_table[piece][source];
                self.zobrist.position ^= self.zobrist.pieces_table[piece][target];

                if piece_move.is_capture() {
                    let (start, end) = match self.side_to_move {
                        Side::White => (Pieces::BlackPawn as usize, Pieces::BlackKing as usize),
                        Side::Black => (Pieces::WhitePawn as usize, Pieces::WhiteKing as usize),
                        _ => unreachable!(),
                    };

                    for piece in start..=end {
                        // if there is a piece on target square, remove that piece and break out
                        if self.boards[piece].get_bit(target).is_set() {
                            self.boards[piece].clear_bit(target);
                            self.zobrist.position ^= self.zobrist.pieces_table[piece][target];
                            break;
                        }
                    }
                }

                if piece_move.promotion().is_promoting() {
                    // remove pawn from its original bitboard and move add the promoted piece to its
                    // corresponding promoted piece
                    let pawn_side = match self.side_to_move {
                        Side::White => Pieces::WhitePawn,
                        Side::Black => Pieces::BlackPawn,
                        _ => unreachable!(),
                    };

                    let promotion = piece_move.promotion();
                    let promoted_piece = promotion.into_piece(self.side_to_move);

                    self.boards[pawn_side].clear_bit(target);
                    self.boards[promoted_piece].set_bit(target);
                    self.zobrist.position ^= self.zobrist.pieces_table[pawn_side][target];
                    self.zobrist.position ^= self.zobrist.pieces_table[promoted_piece][target];
                }

                if piece_move.is_en_passant() {
                    let pawn_side = match self.side_to_move {
                        Side::White => Pieces::BlackPawn,
                        Side::Black => Pieces::WhitePawn,
                        _ => unreachable!(),
                    };

                    let square = match self.side_to_move {
                        Side::White => target.one_backward().unwrap(),
                        Side::Black => target.one_forward().unwrap(),
                        _ => unreachable!(),
                    };

                    self.boards[pawn_side].clear_bit(square);
                    self.zobrist.position ^= self.zobrist.pieces_table[pawn_side][square];
                }

                if self.en_passant.is_available() {
                    self.zobrist.position ^= self.zobrist.en_passant[self.en_passant];
                }
                self.en_passant = Square::OffBoard;

                if piece_move.is_double_push() {
                    self.en_passant = match self.side_to_move {
                        Side::White => target.one_backward().unwrap(),
                        Side::Black => target.one_forward().unwrap(),
                        _ => unreachable!(),
                    };
                    self.zobrist.position ^= self.zobrist.en_passant[self.en_passant];
                }

                if piece_move.is_castling() {
                    let (piece, source, target) = match target {
                        // White castles king side
                        Square::G1 => (Pieces::WhiteRook, Square::H1, Square::F1),
                        // White castles queen side
                        Square::C1 => (Pieces::WhiteRook, Square::A1, Square::D1),
                        // Black castles king side
                        Square::G8 => (Pieces::BlackRook, Square::H8, Square::F8),
                        // Black castles queen side
                        Square::C8 => (Pieces::BlackRook, Square::A8, Square::D8),
                        _ => unreachable!(),
                    };

                    self.boards[piece].clear_bit(source);
                    self.boards[piece].set_bit(target);
                    self.zobrist.position ^= self.zobrist.pieces_table[piece][source];
                    self.zobrist.position ^= self.zobrist.pieces_table[piece][target];
                }

                let source_rights = CASTLING_RIGHTS[source as usize];
                let target_rights = CASTLING_RIGHTS[target as usize];

                self.zobrist.position ^=
                    self.zobrist.castling_rights[self.castling_rights.bits() as usize];

                self.castling_rights = self
                    .castling_rights
                    .intersection(CastlingRights::from_bits_retain(source_rights));

                self.castling_rights = self
                    .castling_rights
                    .intersection(CastlingRights::from_bits_retain(target_rights));

                self.zobrist.position ^=
                    self.zobrist.castling_rights[self.castling_rights.bits() as usize];

                self.occupancies[Side::White] = BitBoard::default();
                self.occupancies[Side::Black] = BitBoard::default();
                self.occupancies[Side::Both] = BitBoard::default();

                for &board in &self.boards[Pieces::white_pieces_range()] {
                    self.occupancies[Side::White] |= board;
                }

                for &board in &self.boards[Pieces::black_pieces_range()] {
                    self.occupancies[Side::Black] |= board;
                }

                let white = self.occupancies[Side::White];
                let black = self.occupancies[Side::Black];
                self.occupancies[Side::Both] |= white;
                self.occupancies[Side::Both] |= black;

                self.side_to_move = self.side_to_move.enemy();
                self.zobrist.position ^= self.zobrist.side_key;
                let king = match self.side_to_move {
                    Side::White => Pieces::BlackKing,
                    Side::Black => Pieces::WhiteKing,
                    _ => unreachable!(),
                };

                let king_square = self.boards[king].trailing_zeros();
                if self.is_square_attacked(king_square, self.side_to_move) {
                    self.undo_move();
                    return false;
                }

                true
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bitboard_from_squares(squares: &[Square]) -> BitBoard {
        BitBoard::from(squares)
    }

    #[test]
    fn test_white_pawn_attacks_center() {
        let attacks = compute_pawn_attacks(Side::White, Square::D4);
        let expected = bitboard_from_squares(&[Square::C5, Square::E5]);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_white_pawn_attacks_edge() {
        let attacks = compute_pawn_attacks(Side::White, Square::A2);
        let expected = bitboard_from_squares(&[Square::B3]);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_black_pawn_attacks_center() {
        let attacks = compute_pawn_attacks(Side::Black, Square::D5);
        let expected = bitboard_from_squares(&[Square::C4, Square::E4]);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_black_pawn_attacks_edge() {
        let attacks = compute_pawn_attacks(Side::Black, Square::H7);
        let expected = bitboard_from_squares(&[Square::G6]);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_knight_attacks_center() {
        let attacks = compute_knight_attacks(Square::D4);
        let expected = bitboard_from_squares(&[
            Square::C6,
            Square::E6,
            Square::B5,
            Square::F5,
            Square::B3,
            Square::F3,
            Square::C2,
            Square::E2,
        ]);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_knight_attacks_corner() {
        let attacks = compute_knight_attacks(Square::A1);
        let expected = bitboard_from_squares(&[Square::B3, Square::C2]);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_king_attacks_center() {
        let attacks = compute_king_attacks(Square::D4);
        let expected = bitboard_from_squares(&[
            Square::C5,
            Square::D5,
            Square::E5,
            Square::C4,
            Square::E4,
            Square::C3,
            Square::D3,
            Square::E3,
        ]);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_king_attacks_corner() {
        let attacks = compute_king_attacks(Square::A8);
        let expected = bitboard_from_squares(&[Square::A7, Square::B7, Square::B8]);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_king_attacks_edge() {
        let attacks = compute_king_attacks(Square::A4);
        let expected =
            bitboard_from_squares(&[Square::A3, Square::A5, Square::B5, Square::B4, Square::B3]);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_rook_blockers_center() {
        let blockers = compute_rook_blockers(Square::D4);
        let expected = bitboard_from_squares(&[
            // vertical (excluding edges)
            Square::D3,
            Square::D2,
            Square::D5,
            Square::D6,
            Square::D7,
            // horizontal (excluding edges)
            Square::C4,
            Square::B4,
            Square::E4,
            Square::F4,
            Square::G4,
        ]);
        assert_eq!(blockers, expected);
    }

    #[test]
    fn test_bishop_blockers_center() {
        let blockers = compute_bishop_blockers(Square::D4);
        let expected = bitboard_from_squares(&[
            // NE
            Square::E5,
            Square::F6,
            Square::G7,
            // NW
            Square::C5,
            Square::B6,
            // SE
            Square::E3,
            Square::F2,
            // SW
            Square::C3,
            Square::B2,
        ]);
        assert_eq!(blockers, expected);
    }

    #[test]
    fn test_rook_attacks_with_blockers() {
        let blockers = bitboard_from_squares(&[Square::D6, Square::F4, Square::F3]);
        let attacks = compute_rook_attacks(Square::D4, blockers);
        let expected = bitboard_from_squares(&[
            // up to D6 (stop at blocker)
            Square::D5,
            Square::D6,
            // all the way down
            Square::D3,
            Square::D2,
            Square::D1,
            // all the way to the left
            Square::C4,
            Square::B4,
            Square::A4,
            // right to F4 (stop at first blocker)
            Square::E4,
            Square::F4,
        ]);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_bishop_attacks_with_blockers() {
        let blockers = bitboard_from_squares(&[Square::F6, Square::B2]);
        let attacks = compute_bishop_attacks(Square::D4, blockers);
        let expected = bitboard_from_squares(&[
            // NE to F6 (stop at blocker)
            Square::E5,
            Square::F6,
            // NW until the edge
            Square::C5,
            Square::B6,
            Square::A7,
            // SE until the edge
            Square::E3,
            Square::F2,
            Square::G1,
            // SW to B2 (stop at blocker)
            Square::C3,
            Square::B2,
        ]);
        assert_eq!(attacks, expected);
    }
}
