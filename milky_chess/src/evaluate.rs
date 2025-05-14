use milky_bitboard::{BitBoard, Pieces, Side};

use crate::{
    BISHOP_POS_SCORE, KING_POS_SCORE, KNIGHT_POS_SCORE, PAWN_POS_SCORE, PIECE_SCORE, ROOK_POS_SCORE,
};

pub fn evaluate_position(side_to_move: Side, boards: [BitBoard; 12]) -> i32 {
    let mut score = 0;

    for (idx, board) in boards.into_iter().enumerate() {
        let piece = Pieces::from_usize_unchecked(idx);

        for square in board {
            score += PIECE_SCORE[idx];

            match piece {
                Pieces::WhitePawn => score += PAWN_POS_SCORE[square as usize],
                Pieces::WhiteKnight => score += KNIGHT_POS_SCORE[square as usize],
                Pieces::WhiteBishop => score += BISHOP_POS_SCORE[square as usize],
                Pieces::WhiteRook => score += ROOK_POS_SCORE[square as usize],
                Pieces::WhiteKing => score += KING_POS_SCORE[square as usize],

                Pieces::BlackPawn => score -= PAWN_POS_SCORE[square.mirror() as usize],
                Pieces::BlackKnight => score -= KNIGHT_POS_SCORE[square.mirror() as usize],
                Pieces::BlackBishop => score -= BISHOP_POS_SCORE[square.mirror() as usize],
                Pieces::BlackRook => score -= ROOK_POS_SCORE[square.mirror() as usize],
                Pieces::BlackKing => score -= KING_POS_SCORE[square.mirror() as usize],
                _ => {}
            }
        }
    }

    match side_to_move {
        Side::White => score,
        Side::Black => -score,
        _ => unreachable!(),
    }
}
