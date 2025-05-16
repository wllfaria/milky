use crate::error::{Error, Result};
use crate::{File, IntoU64, Rank};

#[derive(Debug)]
pub struct SquareIter {
    iter: [Square; 64],
    index: usize,
}

impl Iterator for SquareIter {
    type Item = Square;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > 63 {
            None
        } else {
            self.index += 1;
            Some(self.iter[self.index - 1])
        }
    }
}

#[rustfmt::skip]
#[repr(u64)]
#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Square {
    A8, B8, C8, D8, E8, F8, G8, H8,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A1, B1, C1, D1, E1, F1, G1, H1,
    #[default]
    OffBoard,
}

impl Square {
    /// SAFETY: `value` must always be 0..=63
    pub fn from_u64_unchecked(value: u64) -> Self {
        unsafe { std::mem::transmute(value) }
    }

    pub fn one_forward(&self) -> Option<Self> {
        (*self as u64)
            .checked_sub(8)
            .map(Square::from_u64_unchecked)
    }

    pub fn one_backward(&self) -> Option<Self> {
        let value = (*self as u64) + 8;
        if value > Square::H1 as u64 { None } else { Some(Square::from_u64_unchecked(value)) }
    }

    pub fn mirror(&self) -> Square {
        let index = *self as u64;
        let rank = index / 8;
        let file = index % 8;
        let mirrored_index = (7 - rank) * 8 + file;
        unsafe { std::mem::transmute(mirrored_index) }
    }

    #[rustfmt::skip]
    pub fn is_on_rank(&self, rank: Rank) -> bool {
        match rank {
            Rank::First =>   *self >= Self::A1 && *self <= Self::H1,
            Rank::Second =>  *self >= Self::A2 && *self <= Self::H2,
            Rank::Third =>   *self >= Self::A3 && *self <= Self::H3,
            Rank::Fourth =>  *self >= Self::A4 && *self <= Self::H4,
            Rank::Fifth =>   *self >= Self::A5 && *self <= Self::H5,
            Rank::Sixth =>   *self >= Self::A6 && *self <= Self::H6,
            Rank::Seventh => *self >= Self::A7 && *self <= Self::H7,
            Rank::Eighth =>  *self >= Self::A8 && *self <= Self::H8,
        }
    }

    pub fn file(&self) -> File {
        match self {
            Square::A8
            | Square::A7
            | Square::A6
            | Square::A5
            | Square::A4
            | Square::A3
            | Square::A2
            | Square::A1 => File::A,
            Square::B8
            | Square::B7
            | Square::B6
            | Square::B5
            | Square::B4
            | Square::B3
            | Square::B2
            | Square::B1 => File::B,
            Square::C8
            | Square::C7
            | Square::C6
            | Square::C5
            | Square::C4
            | Square::C3
            | Square::C2
            | Square::C1 => File::C,
            Square::D8
            | Square::D7
            | Square::D6
            | Square::D5
            | Square::D4
            | Square::D3
            | Square::D2
            | Square::D1 => File::D,
            Square::E8
            | Square::E7
            | Square::E6
            | Square::E5
            | Square::E4
            | Square::E3
            | Square::E2
            | Square::E1 => File::E,
            Square::F8
            | Square::F7
            | Square::F6
            | Square::F5
            | Square::F4
            | Square::F3
            | Square::F2
            | Square::F1 => File::F,
            Square::G8
            | Square::G7
            | Square::G6
            | Square::G5
            | Square::G4
            | Square::G3
            | Square::G2
            | Square::G1 => File::G,
            Square::H8
            | Square::H7
            | Square::H6
            | Square::H5
            | Square::H4
            | Square::H3
            | Square::H2
            | Square::H1 => File::H,
            _ => unreachable!(),
        }
    }

    pub fn is_available(&self) -> bool {
        *self != Square::OffBoard
    }

    pub fn from_algebraic_str(str: &str) -> Result<Square> {
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
            _ => Err(Error::InvalidSquare(format!("Invalid square: {str}"))),
        }
    }

    pub fn iter() -> SquareIter {
        SquareIter {
            index: 0,
            iter: [
                Square::A8,
                Square::B8,
                Square::C8,
                Square::D8,
                Square::E8,
                Square::F8,
                Square::G8,
                Square::H8,
                Square::A7,
                Square::B7,
                Square::C7,
                Square::D7,
                Square::E7,
                Square::F7,
                Square::G7,
                Square::H7,
                Square::A6,
                Square::B6,
                Square::C6,
                Square::D6,
                Square::E6,
                Square::F6,
                Square::G6,
                Square::H6,
                Square::A5,
                Square::B5,
                Square::C5,
                Square::D5,
                Square::E5,
                Square::F5,
                Square::G5,
                Square::H5,
                Square::A4,
                Square::B4,
                Square::C4,
                Square::D4,
                Square::E4,
                Square::F4,
                Square::G4,
                Square::H4,
                Square::A3,
                Square::B3,
                Square::C3,
                Square::D3,
                Square::E3,
                Square::F3,
                Square::G3,
                Square::H3,
                Square::A2,
                Square::B2,
                Square::C2,
                Square::D2,
                Square::E2,
                Square::F2,
                Square::G2,
                Square::H2,
                Square::A1,
                Square::B1,
                Square::C1,
                Square::D1,
                Square::E1,
                Square::F1,
                Square::G1,
                Square::H1,
            ],
        }
    }
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
                OffBoard => "--",
            }
        )
    }
}

impl std::ops::Shl<Square> for u64 {
    type Output = u64;

    fn shl(self, rhs: Square) -> Self::Output {
        self << rhs as u64
    }
}

impl std::ops::Index<Square> for [i32; 64] {
    type Output = i32;

    fn index(&self, index: Square) -> &Self::Output {
        &self[index as usize]
    }
}

impl std::ops::IndexMut<Square> for [i32; 64] {
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

impl<T> std::ops::Index<Square> for [T; 64]
where
    T: IntoU64,
{
    type Output = T;

    fn index(&self, index: Square) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> std::ops::IndexMut<Square> for [T; 64]
where
    T: IntoU64,
{
    fn index_mut(&mut self, index: Square) -> &mut Self::Output {
        &mut self[index as usize]
    }
}
