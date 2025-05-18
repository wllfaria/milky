use milky_bitboard::{Move, Pieces, Side};

use crate::board::BoardState;
use crate::search::SearchState;
use crate::{
    BISHOP_POS_SCORE, KING_POS_SCORE, KNIGHT_POS_SCORE, MVV_LVA, PAWN_POS_SCORE, PIECE_SCORE,
    ROOK_POS_SCORE,
};

pub struct EvalContext<'ctx> {
    pub board: &'ctx BoardState,
    pub search: &'ctx mut SearchState,
}

pub fn evaluate_position(ctx: &mut EvalContext<'_>) -> i32 {
    let mut score = 0;

    for (idx, board) in ctx.board.pieces.into_iter().enumerate() {
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

    match ctx.board.side_to_move {
        Side::White => score,
        Side::Black => -score,
        _ => unreachable!(),
    }
}

/// Scores a move based on the following heuristics:
///
/// - PV move
/// - Captures in MVV/LVA
/// - 1st killer move
/// - 2nd killer move
/// - History moves
/// - Unsorted moves
pub fn score_move(ctx: &mut EvalContext<'_>, piece_move: Move) -> i32 {
    const PV_MOVE_SCORE: i32 = 20_000;
    const MVV_LVA_BONUS: i32 = 10_000;
    const FIRST_KILLER_MOVE: i32 = 9_000;
    const SECOND_KILLER_MOVE: i32 = 8_000;

    if ctx.search.score_pv && ctx.search.pv_table[0][ctx.board.ply] == piece_move {
        ctx.search.score_pv = false;
        return PV_MOVE_SCORE;
    }

    if piece_move.is_capture() {
        let attacker = piece_move.piece();
        let victim_square = piece_move.target();

        let (start_piece, end_piece) = if ctx.board.side_to_move == Side::White {
            (Pieces::BlackPawn, Pieces::BlackKing)
        } else {
            (Pieces::WhitePawn, Pieces::WhiteKing)
        };

        // Victim is initialized as white pawn to make en-passant moves easier.
        //
        // Since side doesn't matter for en-passant, even when white is the attacker, white
        // pawn takes white pawn have the same score as white capturing black.
        let victim = (start_piece as usize..=end_piece as usize)
            .find(|&idx| ctx.board.pieces[idx].get_bit(victim_square).is_set())
            .map(Pieces::from_usize_unchecked)
            .unwrap_or(Pieces::WhitePawn);

        return MVV_LVA[attacker][victim] + MVV_LVA_BONUS;
    }

    if ctx.search.killer_moves[0][ctx.board.ply] == piece_move {
        FIRST_KILLER_MOVE
    } else if ctx.search.killer_moves[1][ctx.board.ply] == piece_move {
        SECOND_KILLER_MOVE
    } else {
        ctx.search.history_moves[piece_move.piece()][piece_move.target()]
    }
}
