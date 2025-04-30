#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Side {
    White,
    Black,
    Both,
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
    NoSquare,
}

impl Square {
    /// SAFETY: `value` must always be 0..=63
    #[inline]
    pub fn from_u64_unchecked(value: u64) -> Self {
        unsafe { std::mem::transmute(value) }
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
