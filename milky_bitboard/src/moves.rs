use crate::error::Result;
use crate::{BitBoard, Error, Pieces, Side, Square};

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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum PromotedPieces {
    #[default]
    NoPromotion,
    Knight,
    Bishop,
    Rook,
    Queen,
}

impl std::fmt::Display for PromotedPieces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromotedPieces::NoPromotion => write!(f, ""),
            PromotedPieces::Knight => write!(f, "n"),
            PromotedPieces::Bishop => write!(f, "b"),
            PromotedPieces::Rook => write!(f, "r"),
            PromotedPieces::Queen => write!(f, "q"),
        }
    }
}

impl PromotedPieces {
    pub fn from_u8_unchecked(value: u8) -> Self {
        match value {
            0 => Self::NoPromotion,
            1 => Self::Knight,
            2 => Self::Bishop,
            3 => Self::Rook,
            4 => Self::Queen,
            _ => unreachable!(),
        }
    }

    pub fn from_algebraic_str(value: &str) -> Result<Self> {
        match value {
            "n" => Ok(Self::Knight),
            "b" => Ok(Self::Bishop),
            "r" => Ok(Self::Rook),
            "q" => Ok(Self::Queen),
            _ => Err(Error::InvalidPiece(format!(
                "Invalid promotion piece: {value}"
            ))),
        }
    }

    pub fn into_piece(self, side: Side) -> Pieces {
        match self {
            PromotedPieces::NoPromotion => match side {
                Side::White => Pieces::WhitePawn,
                Side::Black => Pieces::BlackPawn,
                _ => unreachable!(),
            },
            PromotedPieces::Knight => match side {
                Side::White => Pieces::WhiteKnight,
                Side::Black => Pieces::BlackKnight,
                _ => unreachable!(),
            },
            PromotedPieces::Bishop => match side {
                Side::White => Pieces::WhiteBishop,
                Side::Black => Pieces::BlackBishop,
                _ => unreachable!(),
            },
            PromotedPieces::Rook => match side {
                Side::White => Pieces::WhiteRook,
                Side::Black => Pieces::BlackRook,
                _ => unreachable!(),
            },
            PromotedPieces::Queen => match side {
                Side::White => Pieces::WhiteQueen,
                Side::Black => Pieces::BlackQueen,
                _ => unreachable!(),
            },
        }
    }

    pub fn is_promoting(&self) -> bool {
        *self != PromotedPieces::NoPromotion
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
/// │ 0000 │ 0x00 │ No promotion         │
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
    pub fn new(
        source: Square,
        target: Square,
        piece: Pieces,
        promoted: PromotedPieces,
        flags: MoveFlags,
    ) -> Self {
        let encoded = (source as u32)
            | ((target as u32) << 6)
            | ((piece as u32) << 12)
            | ((promoted as u32) << 16)
            | ((flags.bits() as u32) << 20);

        Self(encoded)
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

    pub fn promotion(&self) -> PromotedPieces {
        PromotedPieces::from_u8_unchecked(((self.0 >> 16) & 0xF) as u8)
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

impl std::ops::Deref for Move {
    type Target = u32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
