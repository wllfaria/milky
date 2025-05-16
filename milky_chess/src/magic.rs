use milky_bitboard::{BitBoard, Square};

use crate::random::Random;
use crate::{
    BISHOP_RELEVANT_OCCUPANCIES, ROOK_RELEVANT_OCCUPANCIES, SliderPieceKind,
    compute_bishop_attacks, compute_bishop_blockers, compute_rook_attacks, compute_rook_blockers,
    set_occupancy,
};

#[allow(dead_code)]
pub fn init_magic_numbers() {
    let mut rng = Random::new();

    for square in 0..64 {
        let square = Square::from_u64_unchecked(square);
        let rook_magic = find_magic_number(
            square,
            ROOK_RELEVANT_OCCUPANCIES[square as usize],
            SliderPieceKind::Rook,
            &mut rng,
        );
        println!("0x{rook_magic:X},");
    }

    println!();
    println!();

    for square in 0..64 {
        let square = Square::from_u64_unchecked(square);
        let bishop_magic = find_magic_number(
            square,
            BISHOP_RELEVANT_OCCUPANCIES[square as usize],
            SliderPieceKind::Bishop,
            &mut rng,
        );
        println!("0x{bishop_magic:X},");
    }
}

#[allow(dead_code)]
fn find_magic_number(
    square: Square,
    relevant_bits: u32,
    kind: SliderPieceKind,
    rng: &mut Random,
) -> u64 {
    let mut occupancies = [BitBoard::default(); 4096];
    let mut attacks = [BitBoard::default(); 4096];
    let mut used_attacks = [BitBoard::default(); 4096];

    let blockers = match kind {
        SliderPieceKind::Rook => compute_rook_blockers(square),
        SliderPieceKind::Bishop => compute_bishop_blockers(square),
    };

    let occupancy_idx = 1 << relevant_bits;

    for index in 0..occupancy_idx {
        occupancies[index] = set_occupancy(index, relevant_bits, blockers);

        attacks[index] = match kind {
            SliderPieceKind::Rook => compute_rook_attacks(square, occupancies[index]),
            SliderPieceKind::Bishop => compute_bishop_attacks(square, occupancies[index]),
        }
    }

    'search: for _ in 0..100_000_000 {
        let magic_number = rng.gen_u64() & rng.gen_u64() & rng.gen_u64();
        if ((blockers.wrapping_mul(magic_number)) & 0xFF00_0000_0000_0000).count_ones() < 6 {
            continue;
        }

        for attack in used_attacks.iter_mut().take(occupancy_idx) {
            *attack = BitBoard::default()
        }

        for index in 0..occupancy_idx {
            let magic_index =
                ((occupancies[index].wrapping_mul(magic_number)) >> (64 - relevant_bits)) as usize;

            if used_attacks[magic_index].is_empty() {
                used_attacks[magic_index] = attacks[index];
            } else if used_attacks[magic_index] != attacks[index] {
                continue 'search;
            }
        }

        return magic_number;
    }

    0
}
