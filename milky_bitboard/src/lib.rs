use std::num::Wrapping;

mod error;
mod moves;
mod square;

pub use error::Error;
pub use moves::{Move, MoveFlags, PromotionPieces};
pub use square::Square;

pub trait IntoU64 {
    fn into(self) -> u64;
}

impl IntoU64 for u64 {
    fn into(self) -> u64 {
        self
    }
}

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

#[derive(Debug)]
pub struct PiecesIter {
    iter: [Pieces; 12],
    index: usize,
}

impl Iterator for PiecesIter {
    type Item = Pieces;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > 11 {
            None
        } else {
            self.index += 1;
            Some(self.iter[self.index - 1])
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Pieces {
    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,
    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,
}

impl Pieces {
    pub fn white_pieces_range() -> std::ops::Range<usize> {
        0..6
    }

    pub fn black_pieces_range() -> std::ops::Range<usize> {
        6..12
    }

    pub fn range() -> std::ops::Range<usize> {
        0..12
    }

    pub fn side(&self) -> Side {
        match self {
            Pieces::WhitePawn
            | Pieces::WhiteKnight
            | Pieces::WhiteBishop
            | Pieces::WhiteRook
            | Pieces::WhiteQueen
            | Pieces::WhiteKing => Side::White,
            Pieces::BlackPawn
            | Pieces::BlackKnight
            | Pieces::BlackBishop
            | Pieces::BlackRook
            | Pieces::BlackQueen
            | Pieces::BlackKing => Side::Black,
        }
    }

    pub fn from_usize_unchecked(value: usize) -> Self {
        match value {
            0 => Pieces::WhitePawn,
            1 => Pieces::WhiteKnight,
            2 => Pieces::WhiteBishop,
            3 => Pieces::WhiteRook,
            4 => Pieces::WhiteQueen,
            5 => Pieces::WhiteKing,
            6 => Pieces::BlackPawn,
            7 => Pieces::BlackKnight,
            8 => Pieces::BlackBishop,
            9 => Pieces::BlackRook,
            10 => Pieces::BlackQueen,
            11 => Pieces::BlackKing,
            _ => unreachable!(),
        }
    }

    pub fn from_u8_unchecked(value: u8) -> Self {
        Pieces::from_usize_unchecked(value as usize)
    }

    pub fn iter() -> PiecesIter {
        PiecesIter {
            iter: [
                Pieces::WhitePawn,
                Pieces::WhiteKnight,
                Pieces::WhiteBishop,
                Pieces::WhiteRook,
                Pieces::WhiteQueen,
                Pieces::WhiteKing,
                Pieces::BlackPawn,
                Pieces::BlackKnight,
                Pieces::BlackBishop,
                Pieces::BlackRook,
                Pieces::BlackQueen,
                Pieces::BlackKing,
            ],
            index: 0,
        }
    }
}

impl std::fmt::Display for Pieces {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pieces::WhitePawn => write!(f, "♙"),
            Pieces::WhiteKnight => write!(f, "♘"),
            Pieces::WhiteBishop => write!(f, "♗"),
            Pieces::WhiteRook => write!(f, "♖"),
            Pieces::WhiteQueen => write!(f, "♕"),
            Pieces::WhiteKing => write!(f, "♔"),
            Pieces::BlackPawn => write!(f, "♟"),
            Pieces::BlackKnight => write!(f, "♞"),
            Pieces::BlackBishop => write!(f, "♝"),
            Pieces::BlackRook => write!(f, "♜"),
            Pieces::BlackQueen => write!(f, "♛"),
            Pieces::BlackKing => write!(f, "♚"),
        }
    }
}

impl std::ops::Index<Pieces> for [BitBoard; 12] {
    type Output = BitBoard;

    fn index(&self, index: Pieces) -> &Self::Output {
        &self[index as usize]
    }
}

impl std::ops::IndexMut<Pieces> for [BitBoard; 12] {
    fn index_mut(&mut self, index: Pieces) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl std::ops::Index<Pieces> for [[i32; 12]; 12] {
    type Output = [i32; 12];

    fn index(&self, index: Pieces) -> &Self::Output {
        &self[index as usize]
    }
}

impl std::ops::Index<Pieces> for [i32; 12] {
    type Output = i32;

    fn index(&self, index: Pieces) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> std::ops::Index<Pieces> for [[T; 64]; 12]
where
    T: IntoU64,
{
    type Output = [T; 64];

    fn index(&self, index: Pieces) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> std::ops::IndexMut<Pieces> for [[T; 64]; 12]
where
    T: IntoU64,
{
    fn index_mut(&mut self, index: Pieces) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl std::ops::Index<Pieces> for [[i32; 64]; 12] {
    type Output = [i32; 64];

    fn index(&self, index: Pieces) -> &Self::Output {
        &self[index as usize]
    }
}

impl std::ops::IndexMut<Pieces> for [[i32; 64]; 12] {
    fn index_mut(&mut self, index: Pieces) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Side {
    White,
    Black,
    Both,
}

impl Side {
    pub fn enemy(&self) -> Self {
        match self {
            Self::White => Self::Black,
            Self::Black => Self::White,
            Self::Both => unreachable!(),
        }
    }
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

#[repr(u64)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Rank {
    First,
    Second,
    Third,
    Fourth,
    Fifth,
    Sixth,
    Seventh,
    Eighth,
}

#[repr(u64)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum File {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
}

#[repr(transparent)]
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct BitBoard(Wrapping<u64>);

impl BitBoard {
    pub const fn new(value: u64) -> Self {
        Self(Wrapping(value))
    }

    pub const fn empty() -> Self {
        Self(Wrapping(0))
    }

    pub const fn from_square(square: Square) -> Self {
        BitBoard(Wrapping(1 << square as u64))
    }

    pub fn get_bit(&self, square: Square) -> Self {
        *self & (1 << square as u64)
    }

    pub fn set_bit(&mut self, square: Square) {
        *self |= 1 << square as u64
    }

    pub fn clear_bit(&mut self, square: Square) {
        *self &= !(1 << square as u64);
    }

    pub fn is_empty(self) -> bool {
        self.0 == Wrapping(0)
    }

    pub fn is_set(self) -> bool {
        self.0 != Wrapping(0)
    }

    pub fn attacked_squares(&self, other: Self) -> Self {
        *self & other
    }

    pub fn is_attacked(&self, other: Self) -> bool {
        !self.attacked_squares(other).is_empty()
    }

    pub fn trailing_zeros(&self) -> Square {
        Square::from_u64_unchecked(self.0.0.trailing_zeros() as u64)
    }
}

impl Iterator for BitBoard {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.is_empty() {
            None
        } else {
            let square = self.trailing_zeros();
            self.clear_bit(square);
            Some(square)
        }
    }
}

impl From<u64> for BitBoard {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}

impl From<&[Square]> for BitBoard {
    fn from(squares: &[Square]) -> Self {
        let mut bitboard = BitBoard::default();
        squares.iter().for_each(|square| bitboard.set_bit(*square));
        bitboard
    }
}

impl std::ops::Deref for BitBoard {
    type Target = u64;

    fn deref(&self) -> &Self::Target {
        &self.0.0
    }
}

impl std::ops::Index<Square> for [BitBoard; 64] {
    type Output = BitBoard;

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

    fn index(&self, index: Side) -> &Self::Output {
        &self[index as usize]
    }
}

impl<const SIZE: usize> std::ops::IndexMut<Side> for [BitBoard; SIZE] {
    fn index_mut(&mut self, index: Side) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl std::ops::Not for BitBoard {
    type Output = BitBoard;

    fn not(self) -> Self::Output {
        BitBoard::new(!self.0.0)
    }
}

impl std::ops::Mul for BitBoard {
    type Output = BitBoard;

    fn mul(self, rhs: Self) -> Self::Output {
        BitBoard(self.0 * rhs.0)
    }
}

impl std::ops::Mul<u64> for BitBoard {
    type Output = BitBoard;

    fn mul(self, rhs: u64) -> Self::Output {
        BitBoard(self.0 * Wrapping(rhs))
    }
}

impl std::ops::MulAssign for BitBoard {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0
    }
}

impl std::ops::MulAssign<u64> for BitBoard {
    fn mul_assign(&mut self, rhs: u64) {
        self.0 *= Wrapping(rhs)
    }
}

impl std::ops::BitOr for BitBoard {
    type Output = BitBoard;

    fn bitor(self, rhs: Self) -> Self::Output {
        BitBoard::new(*self | *rhs)
    }
}

impl std::ops::BitOr<u64> for BitBoard {
    type Output = BitBoard;

    fn bitor(self, rhs: u64) -> Self::Output {
        BitBoard::new(*self | rhs)
    }
}

impl std::ops::BitOrAssign for BitBoard {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs
    }
}

impl std::ops::BitOrAssign<u64> for BitBoard {
    fn bitor_assign(&mut self, rhs: u64) {
        *self = *self | rhs
    }
}

impl std::ops::BitAnd for BitBoard {
    type Output = BitBoard;

    fn bitand(self, rhs: Self) -> Self::Output {
        BitBoard::new(*self & *rhs)
    }
}

impl std::ops::BitAnd<u64> for BitBoard {
    type Output = BitBoard;

    fn bitand(self, rhs: u64) -> Self::Output {
        BitBoard::new(*self & rhs)
    }
}

impl std::ops::BitAndAssign for BitBoard {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs
    }
}

impl std::ops::BitAndAssign<u64> for BitBoard {
    fn bitand_assign(&mut self, rhs: u64) {
        *self = *self & rhs
    }
}

impl std::ops::Shl for BitBoard {
    type Output = BitBoard;

    fn shl(self, rhs: Self) -> Self::Output {
        BitBoard::new(*self << *rhs)
    }
}

impl std::ops::Shl<u64> for BitBoard {
    type Output = BitBoard;

    fn shl(self, rhs: u64) -> Self::Output {
        BitBoard::new(*self << rhs)
    }
}

impl std::ops::ShlAssign for BitBoard {
    fn shl_assign(&mut self, rhs: Self) {
        *self = *self << rhs
    }
}

impl std::ops::ShlAssign<u64> for BitBoard {
    fn shl_assign(&mut self, rhs: u64) {
        *self = *self << rhs
    }
}

impl std::ops::Shr for BitBoard {
    type Output = BitBoard;

    fn shr(self, rhs: Self) -> Self::Output {
        BitBoard::new(*self >> *rhs)
    }
}

impl std::ops::Shr<u64> for BitBoard {
    type Output = BitBoard;

    fn shr(self, rhs: u64) -> Self::Output {
        BitBoard::new(*self >> rhs)
    }
}

impl std::ops::ShrAssign for BitBoard {
    fn shr_assign(&mut self, rhs: Self) {
        *self = *self >> rhs
    }
}

impl std::ops::ShrAssign<u64> for BitBoard {
    fn shr_assign(&mut self, rhs: u64) {
        *self = *self >> rhs
    }
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
