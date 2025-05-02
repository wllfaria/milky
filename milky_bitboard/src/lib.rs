bitflags::bitflags! {
    /// ┌──────┬─────┬─────────────────────────────┐
    /// │ bin  │ dec │ description                 │
    /// ├──────┼─────┼─────────────────────────────┤
    /// │ 0001 │  1  │ White can castle king side  │
    /// ├──────┼─────┼─────────────────────────────┤
    /// │ 0010 │  2  │ White can castle queen side │
    /// ├──────┼─────┼─────────────────────────────┤
    /// │ 0100 │  4  │ Black can castle king side  │
    /// ├──────┼─────┼─────────────────────────────┤
    /// │ 1000 │  8  │ Black can castle queen side │
    /// └──────┴─────┴─────────────────────────────┘
    #[repr(transparent)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
    pub struct CastlingRights: u8 {
        const WHITE_K = 0b0001;
        const WHITE_Q = 0b0010;
        const BLACK_K = 0b0100;
        const BLACK_Q = 0b1000;
    }
}

impl std::fmt::Display for CastlingRights {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let white_k = if (self.0 & Self::WHITE_K.0).0 == 0 { "-" } else { "K" };
        let white_q = if (self.0 & Self::WHITE_Q.0).0 == 0 { "-" } else { "Q" };
        let black_k = if (self.0 & Self::BLACK_K.0).0 == 0 { "-" } else { "k" };
        let black_q = if (self.0 & Self::BLACK_Q.0).0 == 0 { "-" } else { "q" };

        write!(f, "{white_k}")?;
        write!(f, "{white_q}")?;
        write!(f, "{black_k}")?;
        write!(f, "{black_q}")?;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Boards {
    WhitePawns,
    WhiteRooks,
    WhiteKnights,
    WhiteBishops,
    WhiteQueens,
    WhiteKing,
    BlackPawns,
    BlackRooks,
    BlackKnights,
    BlackBishops,
    BlackQueens,
    BlackKing,
}

impl Boards {
    pub fn white_pieces_range() -> std::ops::Range<usize> {
        0..6
    }

    pub fn black_pieces_range() -> std::ops::Range<usize> {
        6..12
    }

    pub fn range() -> std::ops::Range<usize> {
        0..12
    }

    pub fn from_usize_unchecked(value: usize) -> Self {
        match value {
            0 => Boards::WhitePawns,
            1 => Boards::WhiteRooks,
            2 => Boards::WhiteKnights,
            3 => Boards::WhiteBishops,
            4 => Boards::WhiteQueens,
            5 => Boards::WhiteKing,
            6 => Boards::BlackPawns,
            7 => Boards::BlackRooks,
            8 => Boards::BlackKnights,
            9 => Boards::BlackBishops,
            10 => Boards::BlackQueens,
            11 => Boards::BlackKing,
            _ => unreachable!(),
        }
    }
}

impl std::fmt::Display for Boards {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Boards::WhitePawns => write!(f, "P"),
            Boards::WhiteKnights => write!(f, "N"),
            Boards::WhiteBishops => write!(f, "B"),
            Boards::WhiteRooks => write!(f, "R"),
            Boards::WhiteQueens => write!(f, "Q"),
            Boards::WhiteKing => write!(f, "K"),
            Boards::BlackPawns => write!(f, "p"),
            Boards::BlackKnights => write!(f, "n"),
            Boards::BlackBishops => write!(f, "b"),
            Boards::BlackRooks => write!(f, "r"),
            Boards::BlackQueens => write!(f, "q"),
            Boards::BlackKing => write!(f, "k"),
        }
    }
}

impl std::ops::Index<Boards> for [BitBoard; 12] {
    type Output = BitBoard;

    fn index(&self, index: Boards) -> &Self::Output {
        &self[index as usize]
    }
}

impl std::ops::IndexMut<Boards> for [BitBoard; 12] {
    fn index_mut(&mut self, index: Boards) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Side {
    White,
    Black,
    Both,
}

impl std::fmt::Display for Side {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Side::White => write!(f, "white"),
            Side::Black => write!(f, "black"),
            Side::Both => write!(f, "both"),
        }
    }
}

#[rustfmt::skip]
#[repr(u64)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Square {
    A8, B8, C8, D8, E8, F8, G8, H8,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A1, B1, C1, D1, E1, F1, G1, H1,
    No,
}

#[rustfmt::skip]
impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Square::*;

        write!(
            f,
            "{}",
            match self {
                A8 => "a8", B8 => "b8", C8 => "c8", D8 => "d8", E8 => "e8", F8 => "f8", G8 => "g8", H8 => "h8",
                A7 => "a7", B7 => "b7", C7 => "c7", D7 => "d7", E7 => "e7", F7 => "f7", G7 => "g7", H7 => "h7",
                A6 => "a6", B6 => "b6", C6 => "c6", D6 => "d6", E6 => "e6", F6 => "f6", G6 => "g6", H6 => "h6",
                A5 => "a5", B5 => "b5", C5 => "c5", D5 => "d5", E5 => "e5", F5 => "f5", G5 => "g5", H5 => "h5",
                A4 => "a4", B4 => "b4", C4 => "c4", D4 => "d4", E4 => "e4", F4 => "f4", G4 => "g4", H4 => "h4",
                A3 => "a3", B3 => "b3", C3 => "c3", D3 => "d3", E3 => "e3", F3 => "f3", G3 => "g3", H3 => "h3",
                A2 => "a2", B2 => "b2", C2 => "c2", D2 => "d2", E2 => "e2", F2 => "f2", G2 => "g2", H2 => "h2",
                A1 => "a1", B1 => "b1", C1 => "c1", D1 => "d1", E1 => "e1", F1 => "f1", G1 => "g1", H1 => "h1",
                No => "--",
            }
        )
    }
}

impl Square {
    /// SAFETY: `value` must always be 0..=63
    #[inline]
    pub fn from_u64_unchecked(value: u64) -> Self {
        unsafe { std::mem::transmute(value) }
    }

    pub fn from_algebraic_str(str: &str) -> Result<Square, String> {
        match str {
            "a1" => Ok(Square::A1),
            "a2" => Ok(Square::A2),
            "a3" => Ok(Square::A3),
            "a4" => Ok(Square::A4),
            "a5" => Ok(Square::A5),
            "a6" => Ok(Square::A6),
            "a7" => Ok(Square::A7),
            "a8" => Ok(Square::A8),
            "b1" => Ok(Square::B1),
            "b2" => Ok(Square::B2),
            "b3" => Ok(Square::B3),
            "b4" => Ok(Square::B4),
            "b5" => Ok(Square::B5),
            "b6" => Ok(Square::B6),
            "b7" => Ok(Square::B7),
            "b8" => Ok(Square::B8),
            "c1" => Ok(Square::C1),
            "c2" => Ok(Square::C2),
            "c3" => Ok(Square::C3),
            "c4" => Ok(Square::C4),
            "c5" => Ok(Square::C5),
            "c6" => Ok(Square::C6),
            "c7" => Ok(Square::C7),
            "c8" => Ok(Square::C8),
            "d1" => Ok(Square::D1),
            "d2" => Ok(Square::D2),
            "d3" => Ok(Square::D3),
            "d4" => Ok(Square::D4),
            "d5" => Ok(Square::D5),
            "d6" => Ok(Square::D6),
            "d7" => Ok(Square::D7),
            "d8" => Ok(Square::D8),
            "e1" => Ok(Square::E1),
            "e2" => Ok(Square::E2),
            "e3" => Ok(Square::E3),
            "e4" => Ok(Square::E4),
            "e5" => Ok(Square::E5),
            "e6" => Ok(Square::E6),
            "e7" => Ok(Square::E7),
            "e8" => Ok(Square::E8),
            "f1" => Ok(Square::F1),
            "f2" => Ok(Square::F2),
            "f3" => Ok(Square::F3),
            "f4" => Ok(Square::F4),
            "f5" => Ok(Square::F5),
            "f6" => Ok(Square::F6),
            "f7" => Ok(Square::F7),
            "f8" => Ok(Square::F8),
            "g1" => Ok(Square::G1),
            "g2" => Ok(Square::G2),
            "g3" => Ok(Square::G3),
            "g4" => Ok(Square::G4),
            "g5" => Ok(Square::G5),
            "g6" => Ok(Square::G6),
            "g7" => Ok(Square::G7),
            "g8" => Ok(Square::G8),
            "h1" => Ok(Square::H1),
            "h2" => Ok(Square::H2),
            "h3" => Ok(Square::H3),
            "h4" => Ok(Square::H4),
            "h5" => Ok(Square::H5),
            "h6" => Ok(Square::H6),
            "h7" => Ok(Square::H7),
            "h8" => Ok(Square::H8),
            _ => Err(format!("Invalid square: {str}")),
        }
    }
}

impl std::ops::Shl<Square> for u64 {
    type Output = u64;

    #[inline]
    fn shl(self, rhs: Square) -> Self::Output {
        self << rhs as u64
    }
}

#[repr(transparent)]
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct BitBoard(u64);

impl BitBoard {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn from_square(square: Square) -> Self {
        BitBoard(1 << square as u64)
    }

    pub fn get_bit(&self, square: Square) -> BitBoard {
        *self & (1 << square as u64)
    }

    pub fn set_bit(&mut self, square: Square) {
        *self |= 1 << square as u64
    }

    pub fn clear_bit(&mut self, square: Square) {
        *self &= !(1 << square as u64);
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    #[inline]
    pub fn is_attacked(&self, other: Self) -> bool {
        !(*self & other).is_empty()
    }
}

impl std::ops::Index<Square> for [BitBoard; 64] {
    type Output = BitBoard;

    #[inline]
    fn index(&self, index: Square) -> &Self::Output {
        &self[index as usize]
    }
}

impl std::ops::IndexMut<Square> for [BitBoard; 64] {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl<const SIZE: usize> std::ops::Index<Side> for [[BitBoard; SIZE]; 2] {
    type Output = [BitBoard; SIZE];

    #[inline]
    fn index(&self, index: Side) -> &Self::Output {
        &self[index as usize]
    }
}

impl<const SIZE: usize> std::ops::IndexMut<Side> for [[BitBoard; SIZE]; 2] {
    fn index_mut(&mut self, index: Side) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl<const SIZE: usize> std::ops::Index<Side> for [BitBoard; SIZE] {
    type Output = BitBoard;

    #[inline]
    fn index(&self, index: Side) -> &Self::Output {
        &self[index as usize]
    }
}

impl<const SIZE: usize> std::ops::IndexMut<Side> for [BitBoard; SIZE] {
    fn index_mut(&mut self, index: Side) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl std::ops::Deref for BitBoard {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<u64> for BitBoard {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<&[Square]> for BitBoard {
    fn from(squares: &[Square]) -> Self {
        let mut bitboard = BitBoard::default();
        squares.iter().for_each(|square| bitboard.set_bit(*square));
        bitboard
    }
}

impl std::ops::BitOrAssign for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl std::ops::BitOrAssign<u64> for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, rhs: u64) {
        *self = *self | rhs
    }
}

impl std::ops::BitAndAssign for BitBoard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl std::ops::BitAndAssign<u64> for BitBoard {
    #[inline]
    fn bitand_assign(&mut self, rhs: u64) {
        *self = *self & rhs
    }
}

impl std::ops::ShlAssign for BitBoard {
    #[inline]
    fn shl_assign(&mut self, rhs: Self) {
        *self = *self << rhs
    }
}

impl std::ops::ShlAssign<u64> for BitBoard {
    #[inline]
    fn shl_assign(&mut self, rhs: u64) {
        *self = *self << rhs
    }
}

impl std::ops::ShrAssign for BitBoard {
    #[inline]
    fn shr_assign(&mut self, rhs: Self) {
        *self = *self >> rhs
    }
}

impl std::ops::ShrAssign<u64> for BitBoard {
    #[inline]
    fn shr_assign(&mut self, rhs: u64) {
        *self = *self >> rhs
    }
}

impl std::ops::MulAssign for BitBoard {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = BitBoard(self.wrapping_mul(*rhs))
    }
}

impl std::ops::MulAssign<u64> for BitBoard {
    #[inline]
    fn mul_assign(&mut self, rhs: u64) {
        *self = BitBoard(self.wrapping_mul(rhs))
    }
}

macro_rules! impl_bit_ops {
    ($trait:ident, $fn:ident, $op:tt) => {
        impl std::ops::$trait for BitBoard {
            type Output = BitBoard;

            fn $fn(self, rhs: Self) -> Self::Output {
                BitBoard(self.0 $op rhs.0)
            }
        }

        impl std::ops::$trait<u64> for BitBoard {
            type Output = BitBoard;

            fn $fn(self, rhs: u64) -> Self::Output {
                BitBoard(self.0 $op rhs)
            }
        }
    };
}

impl std::fmt::Display for BitBoard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f)?;

        for rank in 0..8 {
            let mut line = String::with_capacity(20);
            line.push_str(&format!("  {} ", 8 - rank));

            for file in 0..8 {
                let square = Square::from_u64_unchecked(rank * 8 + file);
                let bit = if !self.get_bit(square).is_empty() { '1' } else { '0' };
                line.push(' ');
                line.push(bit);
            }

            writeln!(f, "{line}")?;
        }

        writeln!(f)?;
        writeln!(f, "     a b c d e f g h")?;
        writeln!(f)?;
        writeln!(f, "     Bitboard: {}", self.0)?;

        Ok(())
    }
}

impl_bit_ops!(BitAnd, bitand, &);
impl_bit_ops!(BitOr,  bitor,  |);
impl_bit_ops!(BitXor, bitxor, ^);
impl_bit_ops!(Shl,    shl,    <<);
impl_bit_ops!(Shr,    shr,    >>);
impl_bit_ops!(Mul,    mul,    *);
