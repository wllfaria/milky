use crate::Square;

#[repr(transparent)]
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct BitBoard(u64);

impl BitBoard {
    pub const fn new(value: u64) -> Self {
        Self(value)
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
