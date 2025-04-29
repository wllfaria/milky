use bitboard::BitBoard;

mod bitboard;

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

/// Every bit is set except for the bits on the A file
static EMPTY_A_FILE: BitBoard = BitBoard::new(18374403900871474942);

/// Every bit is set except for the bits on the H file
static EMPTY_H_FILE: BitBoard = BitBoard::new(9187201950435737471);

/// Every bit is set except for the bits on the GH files
static EMPTY_GH_FILE: BitBoard = BitBoard::new(4557430888798830399);

/// Every bit is set except for the bits on the AB files
static EMPTY_AB_FILE: BitBoard = BitBoard::new(18229723555195321596);

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum Side {
    White,
    Black,
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

#[derive(Debug)]
struct Milky {
    pawn_attacks: [[BitBoard; 64]; 2],
    knight_attacks: [BitBoard; 64],
    king_attacks: [BitBoard; 64],
}

impl Milky {
    pub fn new() -> Self {
        Self {
            pawn_attacks: [[BitBoard::default(); 64]; 2],
            knight_attacks: [BitBoard::default(); 64],
            king_attacks: [BitBoard::default(); 64],
        }
    }

    pub fn init_leaper_piece_attacks(&mut self) {
        for square in 0..64 {
            let square = Square::from_u64_unchecked(square);

            self.pawn_attacks[Side::White][square] = self.compute_pawn_attacks(Side::White, square);
            self.pawn_attacks[Side::Black][square] = self.compute_pawn_attacks(Side::Black, square);
            self.knight_attacks[square] = self.compute_knight_attacks(square);
            self.king_attacks[square] = self.compute_king_attacks(square);
        }
    }

    pub fn compute_pawn_attacks(&self, side: Side, square: Square) -> BitBoard {
        let bitboard = BitBoard::from_square(square);

        match side {
            Side::White => ((bitboard >> 7) & EMPTY_A_FILE) | ((bitboard >> 9) & EMPTY_H_FILE),
            Side::Black => ((bitboard << 7) & EMPTY_H_FILE) | ((bitboard << 9) & EMPTY_A_FILE),
        }
    }

    fn compute_knight_attacks(&self, square: Square) -> BitBoard {
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

    fn compute_king_attacks(&self, square: Square) -> BitBoard {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bitboard_from_squares(squares: &[Square]) -> BitBoard {
        BitBoard::from(squares)
    }

    #[test]
    fn test_white_pawn_attacks_center() {
        let engine = Milky::new();
        let attacks = engine.compute_pawn_attacks(Side::White, Square::D4);
        let expected = bitboard_from_squares(&[Square::C5, Square::E5]);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_white_pawn_attacks_edge() {
        let engine = Milky::new();
        let attacks = engine.compute_pawn_attacks(Side::White, Square::A2);
        let expected = bitboard_from_squares(&[Square::B3]);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_black_pawn_attacks_center() {
        let engine = Milky::new();
        let attacks = engine.compute_pawn_attacks(Side::Black, Square::D5);
        let expected = bitboard_from_squares(&[Square::C4, Square::E4]);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_black_pawn_attacks_edge() {
        let engine = Milky::new();
        let attacks = engine.compute_pawn_attacks(Side::Black, Square::H7);
        let expected = bitboard_from_squares(&[Square::G6]);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_knight_attacks_center() {
        let engine = Milky::new();
        let attacks = engine.compute_knight_attacks(Square::D4);
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
        let engine = Milky::new();
        let attacks = engine.compute_knight_attacks(Square::A1);
        let expected = bitboard_from_squares(&[Square::B3, Square::C2]);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_king_attacks_center() {
        let engine = Milky::new();
        let attacks = engine.compute_king_attacks(Square::D4);
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
        let engine = Milky::new();
        let attacks = engine.compute_king_attacks(Square::A8);
        let expected = bitboard_from_squares(&[Square::A7, Square::B7, Square::B8]);
        assert_eq!(attacks, expected);
    }

    #[test]
    fn test_king_attacks_edge() {
        let engine = Milky::new();
        let attacks = engine.compute_king_attacks(Square::A4);
        let expected =
            bitboard_from_squares(&[Square::A3, Square::A5, Square::B5, Square::B4, Square::B3]);
        assert_eq!(attacks, expected);
    }
}
