use milky_bitboard::{Move, Pieces, Side};

use crate::board::BoardState;
use crate::search::SearchState;
use crate::{BLACK_PASSED_PAWNS_MASKS, FILE_MASKS, ISOLATED_PAWNS_MASKS, WHITE_PASSED_PAWNS_MASKS};

static PASSED_PAWN_BONUS: [i32; 8] = [0, 5, 10, 20, 35, 60, 100, 200];
static DOUBLE_PAWN_PENALTY: i32 = -10;
static ISOLATED_PAWN_PENALTY: i32 = -10;
static SEMI_OPEN_FILE_SCORE: i32 = 10;
static OPEN_FILE_SCORE: i32 = 15;

#[rustfmt::skip]
static PIECE_SCORE: [i32; 12] = [
    100,  // white pawn score
    300,  // white knight scrore
    350,  // white bishop score
    500,  // white rook score
   1000,  // white queen score
  10000,  // white king score
   -100,  // black pawn score
   -300,  // black knight scrore
   -350,  // black bishop score
   -500,  // black rook score
  -1000,  // black queen score
 -10000,  // black king score
];

/// # Most Valuable Victim / Less Valuable Attacker table
///
/// This table is used to apply a bonus to captures based on the values of the pieces. Getting a
/// better beta cuttof on alpha-beta-search.
///
/// # Ordering is:
///
/// .  P   N   B   R   Q   K
/// P 105 205 305 405 505 605
/// N 104 204 304 404 504 604
/// B 103 203 303 403 503 603
/// R 102 202 302 402 502 602
/// Q 101 201 301 401 501 601
/// K 100 200 300 400 500 600
///
/// The table contains twice the size above to enable indexing with `Pieces`.
///
#[rustfmt::skip]
static MVV_LVA: [[i32; 12]; 12] = [
    [105, 205, 305, 405, 505, 605,  105, 205, 305, 405, 505, 605],
    [104, 204, 304, 404, 504, 604,  104, 204, 304, 404, 504, 604],
    [103, 203, 303, 403, 503, 603,  103, 203, 303, 403, 503, 603],
    [102, 202, 302, 402, 502, 602,  102, 202, 302, 402, 502, 602],
    [101, 201, 301, 401, 501, 601,  101, 201, 301, 401, 501, 601],
    [100, 200, 300, 400, 500, 600,  100, 200, 300, 400, 500, 600],

    [105, 205, 305, 405, 505, 605,  105, 205, 305, 405, 505, 605],
    [104, 204, 304, 404, 504, 604,  104, 204, 304, 404, 504, 604],
    [103, 203, 303, 403, 503, 603,  103, 203, 303, 403, 503, 603],
    [102, 202, 302, 402, 502, 602,  102, 202, 302, 402, 502, 602],
    [101, 201, 301, 401, 501, 601,  101, 201, 301, 401, 501, 601],
    [100, 200, 300, 400, 500, 600,  100, 200, 300, 400, 500, 600],
];

#[rustfmt::skip]
static PAWN_POS_SCORE: [i32; 64] = [
    90,  90,  90,  90,  90,  90,  90,  90,
    30,  30,  30,  40,  40,  30,  30,  30,
    20,  20,  20,  30,  30,  30,  20,  20,
    10,  10,  10,  20,  20,  10,  10,  10,
     5,   5,  10,  20,  20,   5,   5,   5,
     0,   0,   0,   5,   5,   0,   0,   0,
     0,   0,   0, -10, -10,   0,   0,   0,
     0,   0,   0,   0,   0,   0,   0,   0,
];

#[rustfmt::skip]
static KNIGHT_POS_SCORE: [i32; 64] = [
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5,   0,   0,  10,  10,   0,   0,  -5,
    -5,   5,  20,  20,  20,  20,   5,  -5,
    -5,  10,  20,  30,  30,  20,  10,  -5,
    -5,  10,  20,  30,  30,  20,  10,  -5,
    -5,   5,  20,  10,  10,  20,   5,  -5,
    -5,   0,   0,   0,   0,   0,   0,  -5,
    -5, -10,   0,   0,   0,   0, -10,  -5,
];

#[rustfmt::skip]
static BISHOP_POS_SCORE: [i32; 64] = [
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   0,   0,  10,  10,   0,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,  10,   0,   0,   0,   0,  10,   0,
     0,  30,   0,   0,   0,   0,  30,   0,
     0,   0, -10,   0,   0, -10,   0,   0,
];

#[rustfmt::skip]
static ROOK_POS_SCORE: [i32; 64] = [
    50,  50,  50,  50,  50,  50,  50,  50,
    50,  50,  50,  50,  50,  50,  50,  50,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,  10,  20,  20,  10,   0,   0,
     0,   0,   0,  20,  20,   0,   0,   0,
];

#[rustfmt::skip]
static KING_POS_SCORE: [i32; 64] = [
     0,   0,   0,   0,   0,   0,   0,   0,
     0,   0,   5,   5,   5,   5,   0,   0,
     0,   5,   5,  10,  10,   5,   5,   0,
     0,   5,  10,  20,  20,  10,   5,   0,
     0,   5,  10,  20,  20,  10,   5,   0,
     0,   0,   5,  10,  10,   5,   0,   0,
     0,   5,   5,  -5,  -5,   0,   5,   0,
     0,   0,   5,   0, -15,   0,  10,   0,
];

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
            let is_white = piece.side() == Side::White;
            let sign = if is_white { 1 } else { -1 };
            let square_idx = if is_white { square as usize } else { square.mirror() as usize };

            match piece {
                Pieces::WhitePawn | Pieces::BlackPawn => {
                    score += sign * PAWN_POS_SCORE[square_idx];

                    // when there are more than one pawn on the same file, a small penalty is given
                    // based on how many pawns are on that file, due to doubled pawns
                    let pawns_on_file = (board & FILE_MASKS[square.file() as usize]).count_ones();
                    if pawns_on_file > 1 {
                        score += sign * DOUBLE_PAWN_PENALTY * pawns_on_file as i32;
                    }

                    // when theres no adjacent friendly pawn, a small penalty is given to the
                    // isolated pawn
                    let mask = board & ISOLATED_PAWNS_MASKS[square.file() as usize];
                    if mask.is_empty() {
                        score += sign * ISOLATED_PAWN_PENALTY;
                    }

                    // when there is no enemy pawn in the same or adjacent files in front of this
                    // pawn, it is considered a passed pawn and gains a small bonus based on how
                    // close it is from queening
                    let enemy_pawn_board = match ctx.board.side_to_move {
                        Side::White => ctx.board.pieces[Pieces::BlackPawn],
                        Side::Black => ctx.board.pieces[Pieces::WhitePawn],
                        _ => unreachable!(),
                    };
                    let passed_pawn_mask = match ctx.board.side_to_move {
                        Side::White => WHITE_PASSED_PAWNS_MASKS.get().unwrap()[square_idx],
                        Side::Black => BLACK_PASSED_PAWNS_MASKS.get().unwrap()[square_idx],
                        _ => unreachable!(),
                    };
                    let mask = enemy_pawn_board & passed_pawn_mask;
                    let bonus_rank = if is_white { square.rank() } else { square.mirror().rank() };
                    if mask.is_empty() {
                        score += sign * PASSED_PAWN_BONUS[bonus_rank as usize];
                    }
                }
                Pieces::WhiteKnight | Pieces::BlackKnight => {
                    score += sign * KNIGHT_POS_SCORE[square_idx];
                }
                Pieces::WhiteBishop | Pieces::BlackBishop => {
                    score += sign * BISHOP_POS_SCORE[square_idx];
                }
                Pieces::WhiteRook | Pieces::BlackRook => {
                    score += sign * ROOK_POS_SCORE[square_idx];
                }
                Pieces::WhiteKing | Pieces::BlackKing => {
                    score += sign * KING_POS_SCORE[square_idx];
                }
                _ => {}
            };
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
