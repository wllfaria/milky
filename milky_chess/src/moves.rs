use milky_bitboard::{
    BitBoard, CastlingRights, Move, MoveFlags, Pieces, PromotionPieces, Rank, Side, Square,
};

use crate::board::{get_bishop_attacks, get_queen_attacks, get_rook_attacks};
use crate::evaluate::{EvalContext, score_move};
use crate::search::SearchState;
use crate::zobrist::Zobrist;
use crate::{BoardState, CASTLING_RIGHTS, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS, attacks};

pub trait Movable {
    fn source(&self) -> Square;
    fn target(&self) -> Square;
    fn promotion(&self) -> PromotionPieces;
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub enum MoveKind {
    AllMoves,
    Captures,
}

pub struct MoveContext<'ctx> {
    pub zobrist: &'ctx mut Zobrist,
    pub search: &'ctx mut SearchState,
    pub board: &'ctx mut BoardState,
}

#[cfg(feature = "bench")]
pub fn make_move_bench(ctx: &mut MoveContext<'_>, piece_move: Move, move_kind: MoveKind) -> bool {
    make_move(ctx, piece_move, move_kind)
}

pub(crate) fn make_move(ctx: &mut MoveContext<'_>, piece_move: Move, move_kind: MoveKind) -> bool {
    match move_kind {
        MoveKind::AllMoves => {
            ctx.board.snapshot_board(ctx.zobrist);

            let source = piece_move.source();
            let target = piece_move.target();
            let piece = piece_move.piece();

            ctx.board.pieces[piece].clear_bit(source);
            ctx.board.pieces[piece].set_bit(target);

            ctx.zobrist.position ^= ctx.zobrist.pieces_table[piece][source];
            ctx.zobrist.position ^= ctx.zobrist.pieces_table[piece][target];

            if piece_move.is_capture() {
                let (start, end) = match ctx.board.side_to_move {
                    Side::White => (Pieces::BlackPawn as usize, Pieces::BlackKing as usize),
                    Side::Black => (Pieces::WhitePawn as usize, Pieces::WhiteKing as usize),
                    _ => unreachable!(),
                };

                for piece in start..=end {
                    // if there is a piece on target square, remove that piece and break out
                    if ctx.board.pieces[piece].get_bit(target).is_set() {
                        ctx.board.pieces[piece].clear_bit(target);
                        ctx.zobrist.position ^= ctx.zobrist.pieces_table[piece][target];
                        break;
                    }
                }
            }

            if piece_move.promotion().is_promoting() {
                // remove pawn from its original bitboard and move add the promoted piece to its
                // corresponding promoted piece
                let pawn_side = match ctx.board.side_to_move {
                    Side::White => Pieces::WhitePawn,
                    Side::Black => Pieces::BlackPawn,
                    _ => unreachable!(),
                };

                let promotion = piece_move.promotion();
                let promoted_piece = promotion.into_piece(ctx.board.side_to_move);

                ctx.board.pieces[pawn_side].clear_bit(target);
                ctx.board.pieces[promoted_piece].set_bit(target);
                ctx.zobrist.position ^= ctx.zobrist.pieces_table[pawn_side][target];
                ctx.zobrist.position ^= ctx.zobrist.pieces_table[promoted_piece][target];
            }

            if piece_move.is_en_passant() {
                let pawn_side = match ctx.board.side_to_move {
                    Side::White => Pieces::BlackPawn,
                    Side::Black => Pieces::WhitePawn,
                    _ => unreachable!(),
                };

                let square = match ctx.board.side_to_move {
                    Side::White => target.one_backward().unwrap(),
                    Side::Black => target.one_forward().unwrap(),
                    _ => unreachable!(),
                };

                ctx.board.pieces[pawn_side].clear_bit(square);
                ctx.zobrist.position ^= ctx.zobrist.pieces_table[pawn_side][square];
            }

            if ctx.board.en_passant.is_available() {
                ctx.zobrist.position ^= ctx.zobrist.en_passant[ctx.board.en_passant];
            }
            ctx.board.en_passant = Square::OffBoard;

            if piece_move.is_double_push() {
                ctx.board.en_passant = match ctx.board.side_to_move {
                    Side::White => target.one_backward().unwrap(),
                    Side::Black => target.one_forward().unwrap(),
                    _ => unreachable!(),
                };
                ctx.zobrist.position ^= ctx.zobrist.en_passant[ctx.board.en_passant];
            }

            if piece_move.is_castling() {
                let (piece, source, target) = match target {
                    // White castles king side
                    Square::G1 => (Pieces::WhiteRook, Square::H1, Square::F1),
                    // White castles queen side
                    Square::C1 => (Pieces::WhiteRook, Square::A1, Square::D1),
                    // Black castles king side
                    Square::G8 => (Pieces::BlackRook, Square::H8, Square::F8),
                    // Black castles queen side
                    Square::C8 => (Pieces::BlackRook, Square::A8, Square::D8),
                    _ => unreachable!(),
                };

                ctx.board.pieces[piece].clear_bit(source);
                ctx.board.pieces[piece].set_bit(target);
                ctx.zobrist.position ^= ctx.zobrist.pieces_table[piece][source];
                ctx.zobrist.position ^= ctx.zobrist.pieces_table[piece][target];
            }

            let source_rights = CASTLING_RIGHTS[source as usize];
            let target_rights = CASTLING_RIGHTS[target as usize];

            ctx.zobrist.position ^=
                ctx.zobrist.castling_rights[ctx.board.castling_rights.bits() as usize];

            ctx.board.castling_rights = ctx
                .board
                .castling_rights
                .intersection(CastlingRights::from_bits_retain(source_rights));

            ctx.board.castling_rights = ctx
                .board
                .castling_rights
                .intersection(CastlingRights::from_bits_retain(target_rights));

            ctx.zobrist.position ^=
                ctx.zobrist.castling_rights[ctx.board.castling_rights.bits() as usize];

            ctx.board.occupancies[Side::White] = BitBoard::default();
            ctx.board.occupancies[Side::Black] = BitBoard::default();
            ctx.board.occupancies[Side::Both] = BitBoard::default();

            for &board in &ctx.board.pieces[Pieces::white_pieces_range()] {
                ctx.board.occupancies[Side::White] |= board;
            }

            for &board in &ctx.board.pieces[Pieces::black_pieces_range()] {
                ctx.board.occupancies[Side::Black] |= board;
            }

            let white = ctx.board.occupancies[Side::White];
            let black = ctx.board.occupancies[Side::Black];
            ctx.board.occupancies[Side::Both] |= white;
            ctx.board.occupancies[Side::Both] |= black;

            ctx.board.side_to_move = ctx.board.side_to_move.enemy();
            ctx.zobrist.position ^= ctx.zobrist.side_key;
            let king = match ctx.board.side_to_move {
                Side::White => Pieces::BlackKing,
                Side::Black => Pieces::WhiteKing,
                _ => unreachable!(),
            };

            let king_square = ctx.board.pieces[king].trailing_zeros();
            if ctx
                .board
                .is_square_attacked(king_square, ctx.board.side_to_move)
            {
                ctx.zobrist.position = ctx.board.undo_move();
                return false;
            }

            true
        }
        MoveKind::Captures => {
            if piece_move.is_capture() {
                make_move(ctx, piece_move, MoveKind::AllMoves)
            } else {
                false
            }
        }
    }
}

pub(crate) fn sort_moves(ctx: &mut MoveContext<'_>) {
    let mut scored_moves = vec![];

    for m in ctx.search.moves.into_iter().take(ctx.search.move_count) {
        let mut eval_context = EvalContext {
            board: ctx.board,
            search: ctx.search,
        };
        scored_moves.push((score_move(&mut eval_context, m), m));
    }

    scored_moves.sort_by_key(|&(score, _)| std::cmp::Reverse(score));

    scored_moves
        .into_iter()
        .enumerate()
        .for_each(|(idx, (_, m))| ctx.search.moves[idx] = m);
}

#[cfg(feature = "bench")]
pub fn generate_moves_bench(ctx: &mut MoveContext<'_>) {
    generate_moves(ctx)
}

pub(crate) fn generate_moves(ctx: &mut MoveContext<'_>) {
    ctx.search.move_count = 0;
    for (idx, board) in ctx.board.pieces.into_iter().enumerate() {
        let piece = Pieces::from_usize_unchecked(idx);

        if piece.side() != ctx.board.side_to_move {
            continue;
        }

        match piece {
            Pieces::WhitePawn | Pieces::BlackPawn => generate_pawn_moves(ctx, board, piece),
            Pieces::WhiteKing | Pieces::BlackKing => generate_king_moves(ctx, board, piece),
            Pieces::WhiteKnight | Pieces::BlackKnight => generate_knight_moves(ctx, board, piece),
            Pieces::WhiteBishop | Pieces::BlackBishop => generate_bishop_moves(ctx, board, piece),
            Pieces::WhiteRook | Pieces::BlackRook => generate_rook_moves(ctx, board, piece),
            Pieces::WhiteQueen | Pieces::BlackQueen => generate_queen_moves(ctx, board, piece),
        }
    }
}

fn generate_pawn_moves(ctx: &mut MoveContext<'_>, board: BitBoard, piece: Pieces) {
    let promotion_rank = match ctx.board.side_to_move {
        Side::White => Rank::Seventh,
        Side::Black => Rank::Second,
        _ => unreachable!(),
    };

    let initial_rank = match ctx.board.side_to_move {
        Side::White => Rank::Second,
        Side::Black => Rank::Seventh,
        _ => unreachable!(),
    };

    let promotion_options = [
        PromotionPieces::Knight,
        PromotionPieces::Bishop,
        PromotionPieces::Rook,
        PromotionPieces::Queen,
    ];

    for square in board {
        let one_forward = match ctx.board.side_to_move {
            Side::White => square.one_forward(),
            Side::Black => square.one_backward(),
            _ => unreachable!(),
        };

        // Skip if the move would leave the board
        let Some(one_forward) = one_forward else {
            continue;
        };

        if ctx.board.occupancies[Side::Both]
            .get_bit(one_forward)
            .is_empty()
        {
            if square.is_on_rank(promotion_rank) {
                for option in promotion_options {
                    ctx.search.push_move(Move::new(
                        square,
                        one_forward,
                        piece,
                        option,
                        MoveFlags::empty(),
                    ));
                }
            } else {
                ctx.search.push_move(Move::new(
                    square,
                    one_forward,
                    piece,
                    PromotionPieces::NoPromotion,
                    MoveFlags::empty(),
                ));
            }

            if square.is_on_rank(initial_rank) {
                // SAFETY: one_forward is valid (verified above)
                let two_forward = match ctx.board.side_to_move {
                    Side::White => one_forward.one_forward().unwrap(),
                    Side::Black => one_forward.one_backward().unwrap(),
                    _ => unreachable!(),
                };

                if ctx.board.occupancies[Side::Both]
                    .get_bit(two_forward)
                    .is_empty()
                {
                    ctx.search.push_move(Move::new(
                        square,
                        two_forward,
                        piece,
                        PromotionPieces::NoPromotion,
                        MoveFlags::DOUBLE_PUSH,
                    ));
                }
            }
        }

        let enemy_occupancies = ctx.board.occupancies[ctx.board.side_to_move.enemy()];
        let pawn_attacks = attacks!(PAWN_ATTACKS)[ctx.board.side_to_move][square];
        let attacks = pawn_attacks.attacked_squares(enemy_occupancies);

        for target in attacks {
            if square.is_on_rank(promotion_rank) {
                for option in promotion_options {
                    ctx.search.push_move(Move::new(
                        square,
                        target,
                        piece,
                        option,
                        MoveFlags::CAPTURE,
                    ));
                }
            } else {
                ctx.search.push_move(Move::new(
                    square,
                    target,
                    piece,
                    PromotionPieces::NoPromotion,
                    MoveFlags::CAPTURE,
                ));
            }
        }

        if ctx.board.en_passant != Square::OffBoard {
            let en_passant_attacks =
                pawn_attacks.attacked_squares(BitBoard::from_square(ctx.board.en_passant));

            if en_passant_attacks.is_set() {
                let target = en_passant_attacks.trailing_zeros();
                ctx.search.push_move(Move::new(
                    square,
                    target,
                    piece,
                    PromotionPieces::NoPromotion,
                    MoveFlags::union(MoveFlags::EN_PASSANT, MoveFlags::CAPTURE),
                ));
            }
        }
    }
}

fn generate_pre_computed_moves<F>(
    ctx: &mut MoveContext<'_>,
    piece: Pieces,
    board: BitBoard,
    get_attacks: F,
) where
    F: Fn(Square) -> BitBoard,
{
    for square in board {
        let attacks = get_attacks(square);
        let occupancies = !ctx.board.occupancies[ctx.board.side_to_move];
        let attacks = attacks.attacked_squares(occupancies);

        for target in attacks {
            let occupancies = ctx.board.occupancies[ctx.board.side_to_move.enemy()];

            if occupancies.get_bit(target).is_set() {
                ctx.search.push_move(Move::new(
                    square,
                    target,
                    piece,
                    PromotionPieces::NoPromotion,
                    MoveFlags::CAPTURE,
                ));
            } else {
                ctx.search.push_move(Move::new(
                    square,
                    target,
                    piece,
                    PromotionPieces::NoPromotion,
                    MoveFlags::empty(),
                ));
            }
        }
    }
}

fn generate_knight_moves(ctx: &mut MoveContext<'_>, board: BitBoard, piece: Pieces) {
    generate_pre_computed_moves(ctx, piece, board, |sq| attacks!(KNIGHT_ATTACKS)[sq]);
}

fn generate_bishop_moves(ctx: &mut MoveContext<'_>, board: BitBoard, piece: Pieces) {
    let occupancies = ctx.board.occupancies[Side::Both];
    generate_pre_computed_moves(ctx, piece, board, |sq| get_bishop_attacks(sq, occupancies));
}

fn generate_rook_moves(ctx: &mut MoveContext<'_>, board: BitBoard, piece: Pieces) {
    let occupancies = ctx.board.occupancies[Side::Both];
    generate_pre_computed_moves(ctx, piece, board, |sq| get_rook_attacks(sq, occupancies));
}

fn generate_queen_moves(ctx: &mut MoveContext<'_>, board: BitBoard, piece: Pieces) {
    let occupancies = ctx.board.occupancies[Side::Both];
    generate_pre_computed_moves(ctx, piece, board, |sq| get_queen_attacks(sq, occupancies));
}

fn generate_king_moves(ctx: &mut MoveContext<'_>, board: BitBoard, piece: Pieces) {
    let king_side = match ctx.board.side_to_move {
        Side::White => CastlingRights::WHITE_K,
        Side::Black => CastlingRights::BLACK_K,
        _ => unreachable!(),
    };

    let queen_side = match ctx.board.side_to_move {
        Side::White => CastlingRights::WHITE_Q,
        Side::Black => CastlingRights::BLACK_Q,
        _ => unreachable!(),
    };

    let king_square = match ctx.board.side_to_move {
        Side::White => Square::E1,
        Side::Black => Square::E8,
        _ => unreachable!(),
    };

    // Check whether white king can castle to the king's side
    if ctx.board.castling_rights.contains(king_side) {
        let required_free_squares = match ctx.board.side_to_move {
            Side::White => (Square::F1, Square::G1),
            Side::Black => (Square::F8, Square::G8),
            _ => unreachable!(),
        };

        // When castling king's side, the squares between the king and king's rook must be
        // empty. That is, for white, squares f1 and g1, and for black, squares f8 and g8.
        let first = ctx.board.occupancies[Side::Both].get_bit(required_free_squares.0);
        let second = ctx.board.occupancies[Side::Both].get_bit(required_free_squares.1);

        // king cannot be in check and the square next to the king  cannot be attacked. That
        // is, for white, squares e1 and f1, and for black, squares e8 and f8.
        let is_king_attacked = ctx
            .board
            .is_square_attacked(king_square, ctx.board.side_to_move.enemy());
        let is_next_attacked = ctx
            .board
            .is_square_attacked(required_free_squares.0, ctx.board.side_to_move.enemy());

        if first.is_empty() && second.is_empty() && !is_king_attacked && !is_next_attacked {
            ctx.search.push_move(Move::new(
                king_square,
                required_free_squares.1,
                piece,
                PromotionPieces::NoPromotion,
                MoveFlags::CASTLING,
            ))
        }
    }

    // Check whether white king can castle to the queen's side
    if ctx.board.castling_rights.contains(queen_side) {
        let required_free_squares = match ctx.board.side_to_move {
            Side::White => (Square::D1, Square::C1, Square::B1),
            Side::Black => (Square::D8, Square::C8, Square::B8),
            _ => unreachable!(),
        };

        // When castling queen's side, the squares between the king and queen's rook must be
        // empty. That is, for white, squares d1, c1 and b1, and for black, squares d8, c8 and
        // b8.
        let first = ctx.board.occupancies[Side::Both].get_bit(required_free_squares.0);
        let second = ctx.board.occupancies[Side::Both].get_bit(required_free_squares.1);
        let third = ctx.board.occupancies[Side::Both].get_bit(required_free_squares.2);

        // king cannot be in check and the square next to the king  cannot be attacked. That
        // is, for white, squares e1 and f1, and for black, squares e8 and f8.
        let is_king_attacked = ctx
            .board
            .is_square_attacked(king_square, ctx.board.side_to_move.enemy());
        let is_next_attacked = ctx
            .board
            .is_square_attacked(required_free_squares.0, ctx.board.side_to_move.enemy());

        if first.is_empty()
            && second.is_empty()
            && third.is_empty()
            && !is_king_attacked
            && !is_next_attacked
        {
            ctx.search.push_move(Move::new(
                king_square,
                required_free_squares.1,
                piece,
                PromotionPieces::NoPromotion,
                MoveFlags::CASTLING,
            ))
        }
    }

    generate_pre_computed_moves(ctx, piece, board, |square| attacks!(KING_ATTACKS)[square]);
}
