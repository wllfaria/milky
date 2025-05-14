use milky_bitboard::{BitBoard, CastlingRights, IntoU64, Pieces, Side, Square};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct ZobristKey(u64);

impl ZobristKey {
    pub fn inner(&self) -> u64 {
        self.0
    }
}

impl IntoU64 for ZobristKey {
    fn into(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for ZobristKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:X}", self.0)
    }
}

impl std::ops::BitXorAssign for ZobristKey {
    fn bitxor_assign(&mut self, rhs: Self) {
        self.0 ^= rhs.0
    }
}

impl std::ops::BitXorAssign<u64> for ZobristKey {
    fn bitxor_assign(&mut self, rhs: u64) {
        self.0 ^= rhs
    }
}

pub struct GamePosition {
    pub boards: [BitBoard; 12],
    pub side_to_move: Side,
    pub en_passant: Square,
    pub castling_rights: CastlingRights,
}

#[derive(Debug)]
pub struct Zobrist {
    pub pieces_table: [[ZobristKey; 64]; 12],
    pub en_passant: [ZobristKey; 64],
    pub castling_rights: [ZobristKey; 16],
    pub side_key: ZobristKey,
    pub position: ZobristKey,
}

impl Default for Zobrist {
    fn default() -> Self {
        Self::new()
    }
}

impl Zobrist {
    pub fn new() -> Self {
        let mut zobrist = Self {
            pieces_table: [[ZobristKey(0); 64]; 12],
            en_passant: [ZobristKey(0); 64],
            castling_rights: [ZobristKey(0); 16],
            side_key: ZobristKey(0),
            position: ZobristKey(0),
        };

        zobrist.init();

        zobrist
    }

    pub fn init(&mut self) {
        let mut rng = crate::random::Random::new();

        for piece in Pieces::iter() {
            for square in Square::iter() {
                self.pieces_table[piece][square] = ZobristKey(rng.gen_u64());
            }
        }

        for square in Square::iter() {
            self.en_passant[square] = ZobristKey(rng.gen_u64());
        }

        for idx in 0..16 {
            self.castling_rights[idx] = ZobristKey(rng.gen_u64());
        }

        self.side_key = ZobristKey(rng.gen_u64());
    }

    pub fn hash_position(&self, position: GamePosition) -> ZobristKey {
        let mut key = ZobristKey(0);

        for piece in Pieces::iter() {
            let board = position.boards[piece];

            for square in board {
                key ^= self.pieces_table[piece][square];
            }
        }

        if position.en_passant.is_available() {
            key ^= self.en_passant[position.en_passant];
        }

        key ^= self.castling_rights[position.castling_rights.bits() as usize];

        if position.side_to_move == Side::Black {
            key ^= self.side_key;
        }

        key
    }
}
