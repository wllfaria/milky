use std::num::Wrapping;

use milky_bitboard::{Move, Pieces, Side, Square};

use crate::evaluate::{EvalContext, evaluate_position};
use crate::moves::{MoveContext, MoveKind, generate_moves, make_move, sort_moves};
use crate::time_manager::{TimeManager, TimeManagerContext};
use crate::transposition_table::{TTFlag, TranspositionTable};
use crate::zobrist::Zobrist;
use crate::{BoardState, MAX_PLY};

pub static INFINITY: i32 = 50000;
pub static MATE_UPPER_BOUND: i32 = 49000;
pub static MATE_LOWER_BOUND: i32 = 48000;

pub type HistoryMoves = [[i32; 64]; 12];
pub type KillerMoves = [[Move; 64]; 2];

fn is_repetition(ctx: &SearchContext<'_>) -> bool {
    ctx.board.repetition_table[0..ctx.board.repetition_index].contains(&ctx.zobrist.position)
}

pub struct SearchContext<'ctx> {
    pub transposition_table: &'ctx mut TranspositionTable,
    pub board: &'ctx mut BoardState,
    pub zobrist: &'ctx mut Zobrist,
    pub(crate) time_manager: TimeManager,
}

pub struct SearchState {
    pub nodes: u64,
    pub score_pv: bool,
    pub follow_pv: bool,
    pub killer_moves: KillerMoves,
    pub history_moves: HistoryMoves,
    pub pv_table: [[Move; MAX_PLY]; MAX_PLY],
    pub pv_length: [usize; MAX_PLY],

    pub moves: [Move; 256],
    pub move_count: usize,
}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            nodes: 0,
            move_count: 0,
            score_pv: false,
            follow_pv: false,

            moves: [Move::default(); 256],
            history_moves: [[0; 64]; 12],
            killer_moves: [[Move::default(); 64]; 2],

            pv_length: [0; MAX_PLY],
            pv_table: [[Move::default(); MAX_PLY]; MAX_PLY],
        }
    }

    pub fn moves(&self) -> impl Iterator<Item = &Move> {
        self.moves[..self.move_count].iter()
    }

    pub fn best_move(&self) -> Move {
        self.pv_table[0][0]
    }

    pub fn enable_pv_scoring(&mut self, game_ply: usize) {
        self.follow_pv = false;

        for piece_move in self.moves.into_iter().take(self.move_count) {
            if self.pv_table[0][game_ply] == piece_move {
                self.score_pv = true;
                self.follow_pv = true;
            }
        }
    }

    pub fn push_move(&mut self, piece_move: Move) {
        self.moves[self.move_count] = piece_move;
        self.move_count += 1;
    }

    pub fn search_position(&mut self, mut ctx: SearchContext<'_>) {
        const ASPIRATION_WINDOW: i32 = 50;

        self.nodes = 0;
        self.follow_pv = false;
        self.score_pv = false;

        self.killer_moves = [[Move::default(); 64]; 2];
        self.history_moves = [[0; 64]; 12];
        self.pv_table = [[Move::default(); MAX_PLY]; MAX_PLY];
        self.pv_length = [0; MAX_PLY];

        let mut alpha = Wrapping(-INFINITY);
        let mut beta = Wrapping(INFINITY);

        let mut curr_depth = 1;

        let start = std::time::Instant::now();

        while !ctx.time_manager.should_stop(TimeManagerContext {
            depth: curr_depth,
            nodes: self.nodes,
        }) {
            self.follow_pv = true;

            let score = self.negamax(&mut ctx, alpha, beta, curr_depth);
            if score <= alpha.0 || score >= beta.0 {
                alpha = Wrapping(-INFINITY);
                beta = Wrapping(INFINITY);
                curr_depth += 1;
                continue;
            }

            alpha = Wrapping(score - ASPIRATION_WINDOW);
            beta = Wrapping(score + ASPIRATION_WINDOW);

            if score > -MATE_UPPER_BOUND && score < -MATE_LOWER_BOUND {
                print!(
                    "info score mate {} depth {curr_depth} nodes {} pv ",
                    -(score + MATE_UPPER_BOUND) / 2 - 1,
                    self.nodes,
                )
            } else if score > MATE_LOWER_BOUND && score < MATE_UPPER_BOUND {
                print!(
                    "info score mate {} depth {curr_depth} nodes {} pv ",
                    (MATE_UPPER_BOUND - score) / 2 + 1,
                    self.nodes,
                )
            } else {
                print!(
                    "info score cp {score} depth {curr_depth} nodes {} pv ",
                    self.nodes
                );
            }

            for idx in 0..self.pv_length[0] {
                print!("{} ", self.pv_table[0][idx]);
            }

            println!();

            curr_depth += 1;
        }

        println!(
            "took: {:?} nodes: {}",
            start.elapsed().as_millis(),
            self.nodes
        );

        println!("bestmove {}", self.pv_table[0][0]);
    }

    fn negamax(
        &mut self,
        ctx: &mut SearchContext<'_>,
        mut alpha: Wrapping<i32>,
        beta: Wrapping<i32>,
        mut depth: u8,
    ) -> i32 {
        const FULL_DEPTH_MOVES: i32 = 4;
        const REDUCTION_LIMIT: u8 = 3;

        let mut tt_flag = TTFlag::Alpha;

        if ctx.board.ply != 0 && is_repetition(ctx) {
            return 0;
        }

        let pv_node = beta.0 - alpha.0 > 1;

        let score = ctx.transposition_table.get(
            ctx.zobrist.position,
            alpha.0,
            beta.0,
            depth,
            ctx.board.ply,
        );

        if let (Some(score), true, true) = (score, ctx.board.ply != 0, !pv_node) {
            return score;
        }

        self.pv_length[ctx.board.ply] = ctx.board.ply;

        if depth == 0 {
            return self.quiescence(ctx, alpha, beta);
        }

        if ctx.board.ply > MAX_PLY - 1 {
            return evaluate_position(&mut EvalContext {
                board: ctx.board,
                search: self,
            });
        }

        self.nodes += 1;

        let king_square = match ctx.board.side_to_move {
            Side::White => ctx.board.pieces[Pieces::WhiteKing].trailing_zeros(),
            Side::Black => ctx.board.pieces[Pieces::BlackKing].trailing_zeros(),
            _ => unreachable!(),
        };

        let in_check = ctx
            .board
            .is_square_attacked(king_square, ctx.board.side_to_move.enemy());

        if in_check {
            // Extend the search depth if in check, this is useful to find forced mates or tactical
            // defenses in dangerous positions
            depth += 1;
        }

        if ctx.time_manager.should_stop(TimeManagerContext {
            depth,
            nodes: self.nodes,
        }) {
            return 0;
        }

        if depth >= REDUCTION_LIMIT && !in_check && ctx.board.ply != 0 {
            ctx.board.snapshot_board(ctx.zobrist);

            ctx.board.ply += 1;
            ctx.board.record_repetition(ctx.zobrist);

            if ctx.board.en_passant.is_available() {
                ctx.zobrist.position ^= ctx.zobrist.en_passant[ctx.board.en_passant];
            }

            ctx.board.en_passant = Square::OffBoard;
            ctx.board.side_to_move = ctx.board.side_to_move.enemy();

            ctx.zobrist.position ^= ctx.zobrist.side_key;

            let score = -Wrapping(self.negamax(ctx, -beta, -beta + Wrapping(1), depth - 1 - 2));
            ctx.board.ply -= 1;
            ctx.board.repetition_index -= 1;
            ctx.zobrist.position = ctx.board.undo_move();

            if score >= beta {
                return beta.0;
            }
        }

        generate_moves(&mut MoveContext {
            zobrist: ctx.zobrist,
            board: ctx.board,
            search: self,
        });

        // If move is within the PV path from the previous iteration, give it a small bonus to
        // improve its position in ordering.
        //
        // This is making the assumption that if we already have a PV, following its path is more
        // likely to have better results.
        if self.follow_pv {
            self.enable_pv_scoring(ctx.board.ply);
        }

        // Order moves by MVV-LVA score to improve pruning efficiency
        sort_moves(&mut MoveContext {
            zobrist: ctx.zobrist,
            board: ctx.board,
            search: self,
        });

        let mut legal_moves = 0;
        let mut moves_searched = 0;

        for piece_move in self.moves.into_iter().take(self.move_count) {
            ctx.board.ply += 1;
            ctx.board.record_repetition(ctx.zobrist);

            let valid_move = make_move(
                &mut MoveContext {
                    search: self,
                    board: ctx.board,
                    zobrist: ctx.zobrist,
                },
                piece_move,
                MoveKind::AllMoves,
            );

            if !valid_move {
                ctx.board.ply -= 1;
                ctx.board.repetition_index -= 1;
                continue;
            }

            legal_moves += 1;

            let score = if moves_searched == 0 {
                -Wrapping(self.negamax(ctx, -beta, -alpha, depth - 1))
            } else {
                // To apply late move reduction, a move cannot be a capture or a promotion, the
                // king must not be in check and the search must also be past the depth allowed to
                // be reduced
                let should_reduce = moves_searched >= FULL_DEPTH_MOVES
                    && depth >= REDUCTION_LIMIT
                    && !in_check
                    && !piece_move.is_capture()
                    && !piece_move.promotion().is_promoting();

                // Apply late move reduction by reducing the depth by 2 per ply
                let shallow = if should_reduce {
                    -Wrapping(self.negamax(ctx, -alpha - Wrapping(1), -alpha, depth - 2))
                } else {
                    // This move should not yet reduce, but we are also on a non-pv path, so
                    // instead of going down the search, we give it a fake score slightly above
                    // alpha that ensures it will trigger the full search below.
                    alpha + Wrapping(1)
                };

                if shallow > alpha {
                    // LMR found a better move, so we search at full depth but with a narrower
                    // window to double check if it is a better move.
                    let deeper =
                        -Wrapping(self.negamax(ctx, -alpha - Wrapping(1), -alpha, depth - 1));

                    // If the narrower window also proves to improve alpha, we do a final full
                    // depth and full width window search.
                    if deeper > alpha && deeper < beta {
                        -Wrapping(self.negamax(ctx, -beta, -alpha, depth - 1))
                    } else {
                        deeper
                    }
                } else {
                    shallow
                }
            };

            ctx.board.ply -= 1;
            ctx.board.repetition_index -= 1;
            ctx.zobrist.position = ctx.board.undo_move();
            moves_searched += 1;

            // Alpha raise
            //
            // The move is better than alpha and smaller than beta, which means it is an
            // improvement on our previously found primary variance and we want to update our
            // primary variance table
            if score > alpha {
                // since we found an exact score, update the flag used at the end
                tt_flag = TTFlag::Exact;

                // History heuristic
                //
                // Keep track of quiet moves that increases alpha by giving them a bonus based on
                // its depth, this put those moves higher on the move sorting
                if !piece_move.is_capture() {
                    self.history_moves[piece_move.piece()][piece_move.target()] += depth as i32;
                }

                alpha = score;

                // Principal variation bookkeeping, the current move is the new best move, so we
                // update the PV table at the current depth to store this move, and copy all the
                // other PV nodes from the deeper ply
                self.pv_table[ctx.board.ply][ctx.board.ply] = piece_move;
                for next_ply in ctx.board.ply + 1..self.pv_length[ctx.board.ply + 1] {
                    self.pv_table[ctx.board.ply][next_ply] =
                        self.pv_table[ctx.board.ply + 1][next_ply];
                }

                self.pv_length[ctx.board.ply] = self.pv_length[ctx.board.ply + 1];

                // Beta cutoff
                //
                // If the current move is so good it exceeds beta, there is no need to search its
                // siblings, as this move is so good the opponent would never allow it to happen.
                //
                // This is a fail-hard alpha/beta search
                if score >= beta {
                    ctx.transposition_table.set(
                        ctx.zobrist.position,
                        depth,
                        beta.0,
                        TTFlag::Beta,
                        ctx.board.ply,
                    );

                    if !piece_move.is_capture() {
                        // When a non-capture (killer move) causes a beta cutoff, we store keep track of
                        // them in order to give them a higher priority in searching when there's a
                        // similar position.
                        self.killer_moves[1][ctx.board.ply] = self.killer_moves[0][ctx.board.ply];
                        self.killer_moves[0][ctx.board.ply] = piece_move;
                    }

                    return beta.0;
                }
            }
        }

        if legal_moves == 0 {
            if in_check {
                return -MATE_UPPER_BOUND + ctx.board.ply as i32;
            } else {
                return 0;
            }
        }

        ctx.transposition_table
            .set(ctx.zobrist.position, depth, alpha.0, tt_flag, ctx.board.ply);

        alpha.0
    }

    fn quiescence(
        &mut self,
        ctx: &mut SearchContext<'_>,
        mut alpha: Wrapping<i32>,
        beta: Wrapping<i32>,
    ) -> i32 {
        self.nodes += 1;

        if ctx.board.ply > MAX_PLY - 1 {
            return evaluate_position(&mut EvalContext {
                board: ctx.board,
                search: self,
            });
        }

        let evaluation = evaluate_position(&mut EvalContext {
            board: ctx.board,
            search: self,
        });

        if evaluation >= beta.0 {
            return beta.0;
        }

        if evaluation > alpha.0 {
            alpha = Wrapping(evaluation);
        }

        generate_moves(&mut MoveContext {
            zobrist: ctx.zobrist,
            board: ctx.board,
            search: self,
        });

        sort_moves(&mut MoveContext {
            zobrist: ctx.zobrist,
            board: ctx.board,
            search: self,
        });

        for piece_move in self.moves.into_iter().take(self.move_count) {
            ctx.board.ply += 1;
            ctx.board.record_repetition(ctx.zobrist);

            if !make_move(
                &mut MoveContext {
                    search: self,
                    board: ctx.board,
                    zobrist: ctx.zobrist,
                },
                piece_move,
                MoveKind::Captures,
            ) {
                ctx.board.ply -= 1;
                ctx.board.repetition_index -= 1;
                continue;
            }

            let score = -Wrapping(self.quiescence(ctx, -beta, -alpha));

            ctx.board.ply -= 1;
            ctx.board.repetition_index -= 1;

            ctx.zobrist.position = ctx.board.undo_move();

            if score > alpha {
                alpha = score;

                if score >= beta {
                    return beta.0;
                }
            }
        }

        alpha.0
    }
}
