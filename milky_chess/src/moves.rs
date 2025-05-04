use milky_bitboard::{Pieces, Square};

bitflags::bitflags! {
    /// ┌──────┬──────────────────┐
    /// │ bin  │ Capture flag     │
    /// ├──────┼──────────────────┤
    /// │ 0001 │ Capture flag     │
    /// ├──────┼──────────────────┤
    /// │ 0010 │ Double push flag │
    /// ├──────┼──────────────────┤
    /// │ 0100 │ En passant flag  │
    /// ├──────┼──────────────────┤
    /// │ 1000 │ Castling flag    │
    /// └──────┴──────────────────┘
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    #[rustfmt::skip]
    pub struct MoveFlags: u8 {
        const CAPTURE     = 0b0001;
        const DOUBLE_PUSH = 0b0010;
        const EN_PASSANT  = 0b0100;
        const CASTLING    = 0b1000;
    }
}

/// Piece move encoding
/// ┌─────┬──────────────────┐
/// │ Bit │ Description      │
/// ├─────┼──────────────────┤
/// │  6  │ Source square    │
/// ├─────┼──────────────────┤
/// │  6  │ Target square    │
/// ├─────┼──────────────────┤
/// │  4  │ Piece moved      │
/// ├─────┼──────────────────┤
/// │  4  │ Promoted piece   │
/// ├─────┼──────────────────┤
/// │  1  │ Capture flag     │
/// ├─────┼──────────────────┤
/// │  1  │ Double push flag │
/// ├─────┼──────────────────┤
/// │  1  │ En passant flag  │
/// ├─────┼──────────────────┤
/// │  1  │ Castling flag    │
/// └─────┴──────────────────┘
///
/// 0 0 0 0 0000 0000 000000 000000
/// ▲ ▲ ▲ ▲ ▲▲▲▲ ▲▲▲▲ ▲▲▲▲▲▲ ▲▲▲▲▲▲
/// │ │ │ │  │    │     │     └─────▶ source square
/// │ │ │ │  │    │     └───────────▶ target square
/// │ │ │ │  │    └─────────────────▶ piece moved
/// │ │ │ │  └──────────────────────▶ promoted piece
/// │ │ │ └─────────────────────────▶ capture flag
/// │ │ └───────────────────────────▶ double push flag
/// │ └─────────────────────────────▶ en passant flag
/// └───────────────────────────────▶ castling flag
///
/// ┌────────────────────────────────────┐
/// │ Binary piece representation        │
/// ├──────┬──────┬──────────────────────┤
/// │ Bit  │ Hex  │ Description          │
/// ├──────┼──────┼──────────────────────┤
/// │ 0000 │ 0x00 │ White Pawn           │
/// │ 0001 │ 0x01 │ White Rook           │
/// │ 0010 │ 0x02 │ White Knight         │
/// │ 0011 │ 0x03 │ White Bishop         │
/// │ 0100 │ 0x04 │ White Queen          │
/// │ 0101 │ 0x05 │ White King           │
/// │ 0110 │ 0x06 │ Black Pawn           │
/// │ 0111 │ 0x07 │ Black Rook           │
/// │ 1000 │ 0x08 │ Black Knight         │
/// │ 1001 │ 0x09 │ Black Bishop         │
/// │ 1010 │ 0x0A │ Black Queen          │
/// │ 1011 │ 0x0B │ Black King           │
/// └──────┴──────┴──────────────────────┘
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Move(u32);

impl Move {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn source(&self) -> Square {
        Square::from_u64_unchecked((self.0 & 0x3F) as u64)
    }

    pub fn target(&self) -> Square {
        Square::from_u64_unchecked(((self.0 >> 6) & 0x3F) as u64)
    }

    pub fn piece(&self) -> Pieces {
        Pieces::from_u8_unchecked(((self.0 >> 12) & 0xF) as u8)
    }

    pub fn promotion(&self) -> Pieces {
        Pieces::from_u8_unchecked(((self.0 >> 16) & 0xF) as u8)
    }

    pub fn is_capture(&self) -> bool {
        (self.0 & 0x100000) != 0
    }

    pub fn is_double_push(&self) -> bool {
        (self.0 & 0x200000) != 0
    }

    pub fn is_en_passant(&self) -> bool {
        (self.0 & 0x400000) != 0
    }

    pub fn is_castling(&self) -> bool {
        (self.0 & 0x800000) != 0
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}",
            self.source(),
            self.target(),
            self.promotion().to_string().to_lowercase()
        )
    }
}

#[derive(Debug)]
pub struct MoveList {
    moves: [Move; 256],
    count: usize,
}

impl Default for MoveList {
    fn default() -> Self {
        Self {
            moves: [Move::default(); 256],
            count: 0,
        }
    }
}

impl MoveList {
    pub fn push_move(&mut self, piece_move: Move) {
        self.moves[self.count] = piece_move;
        self.count += 1;
    }
}

impl std::fmt::Display for MoveList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;
        writeln!(
            f,
            "move     piece    capture    double    en passant    castling"
        )?;

        for piece_move in self.moves.iter().take(self.count) {
            writeln!(
                f,
                "{piece_move}    {}        {}       {}      {}          {}",
                piece_move.piece(),
                piece_move.is_capture(),
                piece_move.is_double_push(),
                piece_move.is_en_passant(),
                piece_move.is_castling(),
            )?;
        }

        writeln!(f)?;
        writeln!(f, "total moves: {}", self.count)?;

        Ok(())
    }
}

#[macro_export]
macro_rules! encode_move {
    (
        $source:expr,
        $target:expr,
        $piece:expr,
        $promoted:expr,
        $flags:expr
        $(,)?
    ) => {{
        $crate::moves::Move::new(
            ($source as u32)
                | (($target as u32) << 6)
                | (($piece as u32) << 12)
                | (($promoted as u32) << 16)
                | (($flags.bits() as u32) << 20),
        )
    }};
}
