use milky_bitboard::{Move, Pieces, Side};

use crate::board::{BoardState, get_bishop_attacks, get_queen_attacks};
use crate::search::SearchState;
use crate::{
    BLACK_PASSED_PAWNS_MASKS, FILE_MASKS, GamePhase, ISOLATED_PAWNS_MASKS, KING_ATTACKS,
    WHITE_PASSED_PAWNS_MASKS, attacks,
};

static PASSED_PAWN_BONUS: [i32; 8] = [0, 5, 10, 20, 35, 60, 100, 200];

static DOUBLE_PAWN_PENALTY: i32 = -10;

static ISOLATED_PAWN_PENALTY: i32 = -10;
static SEMI_OPEN_FILE_SCORE: i32 = 10;
static OPEN_FILE_SCORE: i32 = 15;
static KING_SAFETY_BONUS: i32 = 5;

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
static MVV_LVA: [[i32; 12]; 6] = [
    [105, 205, 305, 405, 505, 605,  105, 205, 305, 405, 505, 605],
    [104, 204, 304, 404, 504, 604,  104, 204, 304, 404, 504, 604],
    [103, 203, 303, 403, 503, 603,  103, 203, 303, 403, 503, 603],
    [102, 202, 302, 402, 502, 602,  102, 202, 302, 402, 502, 602],
    [101, 201, 301, 401, 501, 601,  101, 201, 301, 401, 501, 601],
    [100, 200, 300, 400, 500, 600,  100, 200, 300, 400, 500, 600],
];

pub static ENDGAME_SCORE: i32 = 518;
pub static OPENING_SCORE_THRESHOLD: i32 = 6192;

#[rustfmt::skip]
static MATERIAL_SCORE: [[i32; 12]; 2] = [
    [82, 337, 365, 477, 1025, 12000, -82, -337, -365, -477, -1025, -12000],
    [94, 281, 297, 512, 936, 12000, -94, -281, -297, -512, -936, -12000],
];

#[derive(Debug)]
pub struct PositionalScore {
    early: [i32; 64],
    late: [i32; 64],
}

impl std::ops::Index<GamePhase> for PositionalScore {
    type Output = [i32; 64];

    fn index(&self, index: GamePhase) -> &Self::Output {
        match index {
            GamePhase::Opening => &self.early,
            GamePhase::Endgame => &self.late,
            GamePhase::Midgame => unreachable!(),
        }
    }
}

#[rustfmt::skip]
static TAPERED_PAWN_SCORE: PositionalScore = PositionalScore {
    early: [
          0,   0,   0,   0,   0,   0,  0,   0,
         98, 134,  61,  95,  68, 126, 34, -11,
         -6,   7,  26,  31,  65,  56, 25, -20,
        -14,  13,   6,  21,  23,  12, 17, -23,
        -27,  -2,  -5,  12,  17,   6, 10, -25,
        -26,  -4,  -4, -10,   3,   3, 33, -12,
        -35,  -1, -20, -23, -15,  24, 38, -22,
          0,   0,   0,   0,   0,   0,  0,   0,
    ],
    late: [
          0,   0,   0,   0,   0,   0,   0,   0,
        178, 173, 158, 134, 147, 132, 165, 187,
         94, 100,  85,  67,  56,  53,  82,  84,
         32,  24,  13,   5,  -2,   4,  17,  17,
         13,   9,  -3,  -7,  -7,  -8,   3,  -1,
          4,   7,  -6,   1,   0,  -5,  -1,  -8,
         13,   8,   8,  10,  13,   0,   2,  -7,
          0,   0,   0,   0,   0,   0,   0,   0,
    ],
};

#[rustfmt::skip]
static TAPERED_KNIGHT_SCORE: PositionalScore = PositionalScore {
    early: [
        -167, -89, -34, -49,  61, -97, -15, -107,
         -73, -41,  72,  36,  23,  62,   7,  -17,
         -47,  60,  37,  65,  84, 129,  73,   44,
          -9,  17,  19,  53,  37,  69,  18,   22,
         -13,   4,  16,  13,  28,  19,  21,   -8,
         -23,  -9,  12,  10,  19,  17,  25,  -16,
         -29, -53, -12,  -3,  -1,  18, -14,  -19,
        -105, -21, -58, -33, -17, -28, -19,  -23,
    ],
    late: [
        -58, -38, -13, -28, -31, -27, -63, -99,
        -25,  -8, -25,  -2,  -9, -25, -24, -52,
        -24, -20,  10,   9,  -1,  -9, -19, -41,
        -17,   3,  22,  22,  22,  11,   8, -18,
        -18,  -6,  16,  25,  16,  17,   4, -18,
        -23,  -3,  -1,  15,  10,  -3, -20, -22,
        -42, -20, -10,  -5,  -2, -20, -23, -44,
        -29, -51, -23, -15, -22, -18, -50, -64,
    ],
};

#[rustfmt::skip]
static TAPERED_BISHOP_SCORE: PositionalScore = PositionalScore {
    early: [
        -29,   4, -82, -37, -25, -42,   7,  -8,
        -26,  16, -18, -13,  30,  59,  18, -47,
        -16,  37,  43,  40,  35,  50,  37,  -2,
         -4,   5,  19,  50,  37,  37,   7,  -2,
         -6,  13,  13,  26,  34,  12,  10,   4,
          0,  15,  15,  15,  14,  27,  18,  10,
          4,  15,  16,   0,   7,  21,  33,   1,
        -33,  -3, -14, -21, -13, -12, -39, -21,
    ],
    late: [
        -14, -21, -11,  -8, -7,  -9, -17, -24,
         -8,  -4,   7, -12, -3, -13,  -4, -14,
          2,  -8,   0,  -1, -2,   6,   0,   4,
         -3,   9,  12,   9, 14,  10,   3,   2,
         -6,   3,  13,  19,  7,  10,  -3,  -9,
        -12,  -3,   8,  10, 13,   3,  -7, -15,
        -14, -18,  -7,  -1,  4,  -9, -15, -27,
        -23,  -9, -23,  -5, -9, -16,  -5, -17,
    ],
};

#[rustfmt::skip]
static TAPERED_ROOK_SCORE: PositionalScore = PositionalScore {
    early: [
         32,  42,  32,  51, 63,  9,  31,  43,
         27,  32,  58,  62, 80, 67,  26,  44,
         -5,  19,  26,  36, 17, 45,  61,  16,
        -24, -11,   7,  26, 24, 35,  -8, -20,
        -36, -26, -12,  -1,  9, -7,   6, -23,
        -45, -25, -16, -17,  3,  0,  -5, -33,
        -44, -16, -20,  -9, -1, 11,  -6, -71,
        -19, -13,   1,  17, 16,  7, -37, -26,
    ],
    late: [
        13, 10, 18, 15, 12,  12,   8,   5,
        11, 13, 13, 11, -3,   3,   8,   3,
         7,  7,  7,  5,  4,  -3,  -5,  -3,
         4,  3, 13,  1,  2,   1,  -1,   2,
         3,  5,  8,  4, -5,  -6,  -8, -11,
        -4,  0, -5, -1, -7, -12,  -8, -16,
        -6, -6,  0,  2, -9,  -9, -11,  -3,
        -9,  2,  3, -1, -5, -13,   4, -20,
    ],
};

#[rustfmt::skip]
static TAPERED_QUEEN_SCORE: PositionalScore = PositionalScore {
    early: [
        -28,   0,  29,  12,  59,  44,  43,  45,
        -24, -39,  -5,   1, -16,  57,  28,  54,
        -13, -17,   7,   8,  29,  56,  47,  57,
        -27, -27, -16, -16,  -1,  17,  -2,   1,
         -9, -26,  -9, -10,  -2,  -4,   3,  -3,
        -14,   2, -11,  -2,  -5,   2,  14,   5,
        -35,  -8,  11,   2,   8,  15,  -3,   1,
         -1, -18,  -9,  10, -15, -25, -31, -50,
    ],
    late: [
         -9,  22,  22,  27,  27,  19,  10,  20,
        -17,  20,  32,  41,  58,  25,  30,   0,
        -20,   6,   9,  49,  47,  35,  19,   9,
          3,  22,  24,  45,  57,  40,  57,  36,
        -18,  28,  19,  47,  31,  34,  39,  23,
        -16, -27,  15,   6,   9,  17,  10,   5,
        -22, -23, -30, -16, -16, -23, -36, -32,
        -33, -28, -22, -43,  -5, -32, -20, -41,
    ],
};

#[rustfmt::skip]
static TAPERED_KING_SCORE: PositionalScore = PositionalScore {
    early: [
        -65,  23,  16, -15, -56, -34,   2,  13,
         29,  -1, -20,  -7,  -8,  -4, -38, -29,
         -9,  24,   2, -16, -20,   6,  22, -22,
        -17, -20, -12, -27, -30, -25, -14, -36,
        -49,  -1, -27, -39, -46, -44, -33, -51,
        -14, -14, -22, -46, -44, -30, -15, -27,
          1,   7,  -8, -64, -43, -16,   9,   8,
        -15,  36,  12, -54,   8, -28,  24,  14,
    ],
    late: [
        -74, -35, -18, -18, -11,  15,   4, -17,
        -12,  17,  14,  17,  17,  38,  23,  11,
         10,  17,  23,  15,  20,  45,  44,  13,
         -8,  22,  24,  27,  26,  33,  26,   3,
        -18,  -4,  21,  24,  27,  23,   9, -11,
        -19,  -3,  11,  21,  23,  16,   7,  -9,
        -27, -11,   4,  13,  14,   4,  -5, -17,
        -53, -34, -21, -11, -28, -14, -24, -430
    ],
};

pub struct EvalContext<'ctx> {
    pub board: &'ctx BoardState,
    pub search: &'ctx mut SearchState,
}

pub fn get_game_phase_score(ctx: &mut EvalContext<'_>) -> i32 {
    let mut white_pieces_score = 0;
    let mut black_pieces_score = 0;

    // skip pawns (0th index) and king (last index)
    for piece_idx in Pieces::white_pieces_range().skip(1).take(4) {
        let piece_amount = ctx.board.pieces[piece_idx].count_ones() as i32;
        white_pieces_score += piece_amount * MATERIAL_SCORE[0][piece_idx];
    }

    // skip pawns (0th index) and king (last index)
    for piece_idx in Pieces::black_pieces_range().skip(1).take(4) {
        let piece_amount = ctx.board.pieces[piece_idx].count_ones() as i32;
        black_pieces_score += piece_amount * -MATERIAL_SCORE[0][piece_idx];
    }

    white_pieces_score + black_pieces_score
}

fn interpolate_score(
    game_phase: GamePhase,
    opening_score: i32,
    endgame_score: i32,
    game_phase_score: i32,
) -> i32 {
    match game_phase {
        GamePhase::Opening => opening_score,
        GamePhase::Endgame => endgame_score,
        GamePhase::Midgame => {
            (opening_score * game_phase_score
                + endgame_score * (OPENING_SCORE_THRESHOLD - game_phase_score))
                / OPENING_SCORE_THRESHOLD
        }
    }
}

pub fn evaluate_position(ctx: &mut EvalContext<'_>) -> i32 {
    let game_phase_score = get_game_phase_score(ctx);
    let game_phase = GamePhase::from_score(game_phase_score);

    let mut score_opening = 0;
    let mut score_endgame = 0;

    for (idx, board) in ctx.board.pieces.into_iter().enumerate() {
        let piece = Pieces::from_usize_unchecked(idx);

        for square in board {
            let is_white = piece.side() == Side::White;
            let sign = if is_white { 1 } else { -1 };
            let square_idx = if is_white { square as usize } else { square.mirror() as usize };

            score_opening += MATERIAL_SCORE[GamePhase::Opening as usize][piece];
            score_endgame += MATERIAL_SCORE[GamePhase::Endgame as usize][piece];

            match piece {
                Pieces::WhitePawn | Pieces::BlackPawn => {
                    score_opening += sign * TAPERED_PAWN_SCORE[GamePhase::Opening][square_idx];
                    score_endgame += sign * TAPERED_PAWN_SCORE[GamePhase::Endgame][square_idx];

                    // when there are more than one pawn on the same file, a small penalty is given
                    // based on how many pawns are on that file, due to doubled pawns
                    // let pawns_on_file = (board & FILE_MASKS[square.file() as usize]).count_ones();
                    // if pawns_on_file > 1 {
                    //     score += sign * DOUBLE_PAWN_PENALTY * pawns_on_file as i32;
                    // }

                    // when theres no adjacent friendly pawn, a small penalty is given to the
                    // isolated pawn
                    // let mask = board & ISOLATED_PAWNS_MASKS[square.file() as usize];
                    // if mask.is_empty() {
                    //     score += sign * ISOLATED_PAWN_PENALTY;
                    // }

                    // when there is no enemy pawn in the same or adjacent files in front of this
                    // pawn, it is considered a passed pawn and gains a small bonus based on how
                    // close it is from queening
                    // let enemy_pawn_board = match ctx.board.side_to_move {
                    //     Side::White => ctx.board.pieces[Pieces::BlackPawn],
                    //     Side::Black => ctx.board.pieces[Pieces::WhitePawn],
                    //     _ => unreachable!(),
                    // };
                    // let passed_pawn_mask = match ctx.board.side_to_move {
                    //     Side::White => WHITE_PASSED_PAWNS_MASKS.get().unwrap()[square_idx],
                    //     Side::Black => BLACK_PASSED_PAWNS_MASKS.get().unwrap()[square_idx],
                    //     _ => unreachable!(),
                    // };
                    // let mask = enemy_pawn_board & passed_pawn_mask;
                    // let bonus_rank = if is_white { square.rank() } else { square.mirror().rank() };
                    // if mask.is_empty() {
                    //     score += sign * PASSED_PAWN_BONUS[bonus_rank as usize];
                    // }
                }
                Pieces::WhiteKnight | Pieces::BlackKnight => {
                    score_opening += sign * TAPERED_KNIGHT_SCORE[GamePhase::Opening][square_idx];
                    score_endgame += sign * TAPERED_KNIGHT_SCORE[GamePhase::Endgame][square_idx];
                }
                Pieces::WhiteBishop | Pieces::BlackBishop => {
                    score_opening += sign * TAPERED_BISHOP_SCORE[GamePhase::Opening][square_idx];
                    score_endgame += sign * TAPERED_BISHOP_SCORE[GamePhase::Endgame][square_idx];

                    // small bonus to bishop mobility based on the amount of squares it control.
                    // let occupancies = ctx.board.occupancies[Side::Both];
                    // let available_squares = get_bishop_attacks(square, occupancies).count_ones();
                    // score += sign * available_squares as i32;
                }
                Pieces::WhiteQueen | Pieces::BlackQueen => {
                    score_opening += sign * TAPERED_QUEEN_SCORE[GamePhase::Opening][square_idx];
                    score_endgame += sign * TAPERED_QUEEN_SCORE[GamePhase::Endgame][square_idx];
                    // small bonus to queen mobility based on the amount of squares it control.
                    // let occupancies = ctx.board.occupancies[Side::Both];
                    // let available_squares = get_queen_attacks(square, occupancies).count_ones();
                    // score += sign * available_squares as i32;
                }
                Pieces::WhiteRook | Pieces::BlackRook => {
                    score_opening += sign * TAPERED_ROOK_SCORE[GamePhase::Opening][square_idx];
                    score_endgame += sign * TAPERED_ROOK_SCORE[GamePhase::Endgame][square_idx];

                    // let pawn_board = match ctx.board.side_to_move {
                    //     Side::White => ctx.board.pieces[Pieces::WhitePawn],
                    //     Side::Black => ctx.board.pieces[Pieces::BlackPawn],
                    //     _ => unreachable!(),
                    // };
                    // let enemy_pawn_board = match ctx.board.side_to_move {
                    //     Side::White => ctx.board.pieces[Pieces::BlackPawn],
                    //     Side::Black => ctx.board.pieces[Pieces::WhitePawn],
                    //     _ => unreachable!(),
                    // };
                    //
                    // // when there is no friendly pawn in front of a rook, we consider it a
                    // // semi-open file, and give it a small bonus
                    // let mask = pawn_board & FILE_MASKS[square.file() as usize];
                    // if mask.is_empty() {
                    //     score += sign * SEMI_OPEN_FILE_SCORE;
                    // }
                    //
                    // // when there is no pawn (enemy or friendly) in front of a rook, we consider it
                    // // a full-open file, and give it a slight bigger bonus
                    // let mask = (pawn_board | enemy_pawn_board) & FILE_MASKS[square.file() as usize];
                    // if mask.is_empty() {
                    //     score += sign * OPEN_FILE_SCORE;
                    // }
                }
                Pieces::WhiteKing | Pieces::BlackKing => {
                    score_opening += sign * TAPERED_KING_SCORE[GamePhase::Opening][square_idx];
                    score_endgame += sign * TAPERED_KING_SCORE[GamePhase::Endgame][square_idx];

                    // let pawn_board = match piece.side() {
                    //     Side::White => ctx.board.pieces[Pieces::WhitePawn],
                    //     Side::Black => ctx.board.pieces[Pieces::BlackPawn],
                    //     _ => unreachable!(),
                    // };
                    // let enemy_pawn_board = match piece.side() {
                    //     Side::White => ctx.board.pieces[Pieces::BlackPawn],
                    //     Side::Black => ctx.board.pieces[Pieces::WhitePawn],
                    //     _ => unreachable!(),
                    // };
                    //
                    // // semi-open and open files for kings works in the same way as they do for
                    // // rooks. Except they are penalties for kings.
                    // let mask = pawn_board & FILE_MASKS[square.file() as usize];
                    // if mask.is_empty() {
                    //     score -= sign * SEMI_OPEN_FILE_SCORE;
                    // }
                    //
                    // let mask = (pawn_board | enemy_pawn_board) & FILE_MASKS[square.file() as usize];
                    // if mask.is_empty() {
                    //     score -= sign * OPEN_FILE_SCORE;
                    // }
                    //
                    // let occupancies = match piece.side() {
                    //     Side::White => ctx.board.occupancies[Side::White],
                    //     Side::Black => ctx.board.occupancies[Side::Black],
                    //     _ => unreachable!(),
                    // };
                    // let shield_size = (attacks!(KING_ATTACKS)[square] & occupancies).count_ones();
                    // score += sign * (shield_size as i32 * KING_SAFETY_BONUS);
                }
            };
        }
    }

    let score = interpolate_score(game_phase, score_opening, score_endgame, game_phase_score);

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

        return MVV_LVA[attacker.kind()][victim] + MVV_LVA_BONUS;
    }

    if ctx.search.killer_moves[0][ctx.board.ply] == piece_move {
        FIRST_KILLER_MOVE
    } else if ctx.search.killer_moves[1][ctx.board.ply] == piece_move {
        SECOND_KILLER_MOVE
    } else {
        ctx.search.history_moves[piece_move.piece()][piece_move.target()]
    }
}
