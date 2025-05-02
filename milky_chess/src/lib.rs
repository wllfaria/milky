use std::sync::OnceLock;

use milky_bitboard::{BitBoard, Boards, CastlingRights, Side, Square};
use milky_fen::FenParts;
use random::Random;

mod random;

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
                    let magic = occupancy.wrapping_mul(*BISHOP_MAGIC_BITBOARDS[index as usize]);
                    let shift = 64 - BISHOP_RELEVANT_OCCUPANCIES[index as usize] as u64;
                    magic >> shift
                }
                SliderPieceKind::Rook => {
                    let magic = occupancy.wrapping_mul(*ROOK_MAGIC_BITBOARDS[index as usize]);
                    let shift = 64 - ROOK_RELEVANT_OCCUPANCIES[index as usize] as u64;
                    magic >> shift
                }
            };

            match kind {
                SliderPieceKind::Bishop => {
                    bishop_attacks[square as usize][magic_index as usize] =
                        compute_bishop_attacks(square, occupancy);
                }
                SliderPieceKind::Rook => {
                    rook_attacks[square as usize][magic_index as usize] =
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
        let square = Square::from_u64_unchecked(attackers.trailing_zeros() as u64);
        attackers.clear_bit(square);

        if index & (1 << count) != 0 {
            occupancy.set_bit(square);
        }
    }

    occupancy
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
enum SliderPieceKind {
    Rook,
    Bishop,
}

#[derive(Debug)]
struct Milky {
    rng: Random,
    boards: [BitBoard; 12],
    occupancies: [BitBoard; 3],
    side_to_move: Side,
    en_passant: Square,
    castling_rights: CastlingRights,
}

impl Milky {
    pub fn new() -> Self {
        Self {
            rng: Random::default(),
            boards: [BitBoard::empty(); 12],
            occupancies: [BitBoard::empty(); 3],
            side_to_move: Side::White,
            en_passant: Square::No,
            castling_rights: CastlingRights::all(),
        }
    }

    #[inline]
    fn get_bishop_attacks(&self, square: Square, mut occupancy: BitBoard) -> BitBoard {
        occupancy &= BISHOP_BLOCKERS.get().unwrap()[square];
        occupancy *= BISHOP_MAGIC_BITBOARDS[square];
        occupancy >>= (64 - BISHOP_RELEVANT_OCCUPANCIES[square as usize]) as u64;

        BISHOP_ATTACKS.get().unwrap()[square as usize][*occupancy as usize]
    }

    #[inline]
    fn get_rook_attacks(&self, square: Square, mut occupancy: BitBoard) -> BitBoard {
        occupancy &= ROOK_BLOCKERS.get().unwrap()[square];
        occupancy *= ROOK_MAGIC_BITBOARDS[square];
        occupancy >>= (64 - ROOK_RELEVANT_OCCUPANCIES[square as usize]) as u64;

        ROOK_ATTACKS.get().unwrap()[square as usize][*occupancy as usize]
    }

    #[inline]
    fn get_queen_attacks(&self, square: Square, occupancy: BitBoard) -> BitBoard {
        let mut queen_attacks = BitBoard::default();
        let bishop_occupancies = occupancy;
        let rook_occupancies = occupancy;

        queen_attacks = self.get_bishop_attacks(square, bishop_occupancies);
        queen_attacks |= self.get_rook_attacks(square, rook_occupancies);

        queen_attacks
    }

    fn load_fen(&mut self, fen_parts: FenParts) {
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
    }

    fn print_board(&self) {
        println!();

        for rank in 0..8 {
            let mut line = String::with_capacity(20);
            line.push_str(&format!("  {} ", 8 - rank));

            for file in 0..8 {
                let square = Square::from_u64_unchecked(rank * 8 + file);
                let mut piece = String::from(".");

                for (idx, &board) in self.boards.iter().enumerate() {
                    if !board.get_bit(square).is_empty() {
                        piece = Boards::from_usize_unchecked(idx).to_string();
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
    }

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
            let magic_number = self.rng.gen_magic_number_candidate();
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
                self.boards[Boards::WhitePawns],
                self.boards[Boards::WhiteKnights],
                self.boards[Boards::WhiteKing],
                self.boards[Boards::WhiteBishops],
                self.boards[Boards::WhiteRooks],
                self.boards[Boards::WhiteQueens],
            ),
            Side::Black => (
                Side::White,
                self.boards[Boards::BlackPawns],
                self.boards[Boards::BlackKnights],
                self.boards[Boards::BlackKing],
                self.boards[Boards::BlackBishops],
                self.boards[Boards::BlackRooks],
                self.boards[Boards::BlackQueens],
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

        if self
            .get_bishop_attacks(square, occupancy)
            .is_attacked(bishop_board)
        {
            return true;
        }

        if self
            .get_rook_attacks(square, occupancy)
            .is_attacked(rook_board)
        {
            return true;
        }

        if self
            .get_queen_attacks(square, occupancy)
            .is_attacked(queen_board)
        {
            return true;
        }

        false
    }

    fn print_attacked_squares(&self, side: Side) {
        println!();

        for rank in 0..8 {
            let mut line = String::with_capacity(20);
            line.push_str(&format!("  {} ", 8 - rank));

            for file in 0..8 {
                let square = Square::from_u64_unchecked(rank * 8 + file);
                let bit = if self.is_square_attacked(square, side) { '1' } else { '0' };

                line.push(' ');
                line.push(bit);
            }

            println!("{line}");
        }

        println!();
        println!("     a b c d e f g h");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        init_attack_tables();

        let fen_parts =
            milky_fen::parse_fen_string("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - ")
                .unwrap();

        let mut engine = Milky::new();

        engine.load_fen(fen_parts);
        engine.print_board();
        engine.print_attacked_squares(Side::Black);

        panic!();
    }

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
