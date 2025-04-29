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

    fn set_occupancies(index: u32, bits_in_mask: u32, mut attackers: BitBoard) -> BitBoard {
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

    fn init_leaper_piece_attacks(&mut self) {
        for square in 0..64 {
            let square = Square::from_u64_unchecked(square);

            self.pawn_attacks[Side::White][square] = self.compute_pawn_attacks(Side::White, square);
            self.pawn_attacks[Side::Black][square] = self.compute_pawn_attacks(Side::Black, square);
            self.knight_attacks[square] = self.compute_knight_attacks(square);
            self.king_attacks[square] = self.compute_king_attacks(square);
        }
    }

    fn compute_pawn_attacks(&self, side: Side, square: Square) -> BitBoard {
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

    fn compute_bishop_attacks(&self, square: Square, blockers: BitBoard) -> BitBoard {
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

    fn compute_rook_attacks(&self, square: Square, blockers: BitBoard) -> BitBoard {
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

    fn compute_bishop_blockers(&self, square: Square) -> BitBoard {
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

    fn compute_rook_blockers(&self, square: Square) -> BitBoard {
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

    #[test]
    fn test_rook_blockers_center() {
        let engine = Milky::new();
        let blockers = engine.compute_rook_blockers(Square::D4);
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
        let engine = Milky::new();
        let blockers = engine.compute_bishop_blockers(Square::D4);
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
        let engine = Milky::new();
        let blockers = bitboard_from_squares(&[Square::D6, Square::F4, Square::F3]);
        let attacks = engine.compute_rook_attacks(Square::D4, blockers);
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
        let engine = Milky::new();
        let blockers = bitboard_from_squares(&[Square::F6, Square::B2]);
        let attacks = engine.compute_bishop_attacks(Square::D4, blockers);
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
