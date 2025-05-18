use milky_bitboard::{Pieces, Square};
use milky_fen::FenParts;

use crate::board::BoardState;
use crate::moves::MoveKind;
use crate::search::{SearchContext, SearchState};
use crate::time_manager::{IntoTimeControl, SearchLimits, TimeManager};
use crate::transposition_table::TranspositionTable;
use crate::zobrist::{GamePosition, Zobrist};
use crate::{Movable, MoveContext, generate_moves, make_move};

pub struct Milky {
    board_state: BoardState,
    zobrist: Zobrist,
    transposition_table: TranspositionTable,
    search_state: SearchState,
}

impl Default for Milky {
    fn default() -> Self {
        Self::new()
    }
}

impl Milky {
    pub fn new() -> Self {
        Self {
            board_state: BoardState::default(),
            zobrist: Zobrist::default(),
            transposition_table: TranspositionTable::default(),
            search_state: SearchState::default(),
        }
    }

    pub fn board_state(&self) -> &BoardState {
        &self.board_state
    }

    pub fn board_state_mut(&mut self) -> &mut BoardState {
        &mut self.board_state
    }

    pub fn zobrist(&self) -> &Zobrist {
        &self.zobrist
    }

    pub fn zobrist_mut(&mut self) -> &mut Zobrist {
        &mut self.zobrist
    }

    pub fn search_state(&self) -> &SearchState {
        &self.search_state
    }

    pub fn search_state_mut(&mut self) -> &mut SearchState {
        &mut self.search_state
    }

    pub fn new_game(&mut self) {
        self.transposition_table.clear();
        self.board_state.reset();
    }

    pub fn load_position(&mut self, fen_parts: FenParts) {
        let occupancies = [
            fen_parts.white_occupancy,
            fen_parts.black_occupancy,
            fen_parts.both_occupancy,
        ];

        self.board_state.pieces = fen_parts.positions;
        self.board_state.occupancies = occupancies;
        self.board_state.en_passant = fen_parts.en_passant;
        self.board_state.side_to_move = fen_parts.side_to_move;
        self.board_state.castling_rights = fen_parts.castling_rights;

        self.zobrist.position = self.zobrist.hash_position(GamePosition {
            boards: self.board_state.pieces,
            side_to_move: self.board_state.side_to_move,
            en_passant: self.board_state.en_passant,
            castling_rights: self.board_state.castling_rights,
        })
    }

    pub fn load_moves(&mut self, moves: impl Iterator<Item = impl Movable>) {
        for mv in moves {
            generate_moves(&mut MoveContext {
                board: &mut self.board_state,
                zobrist: &mut self.zobrist,
                search: &mut self.search_state,
            });

            let valid_move = self.search_state.moves().find(|m| {
                m.source() == mv.source()
                    && m.target() == mv.target()
                    && m.promotion() == mv.promotion()
            });

            let Some(&valid_move) = valid_move else {
                return;
            };

            self.board_state.record_repetition(&mut self.zobrist);
            let mut move_context = MoveContext {
                board: &mut self.board_state,
                zobrist: &mut self.zobrist,
                search: &mut self.search_state,
            };
            make_move(&mut move_context, valid_move, MoveKind::AllMoves);
        }
    }

    pub fn think(&mut self, time_control: impl IntoTimeControl) {
        let time_manager = TimeManager::new(SearchLimits::new(
            time_control.into_time_control(self.board_state.side_to_move),
        ));

        self.search_state.search_position(SearchContext {
            transposition_table: &mut self.transposition_table,
            zobrist: &mut self.zobrist,
            board: &mut self.board_state,
            time_manager,
        });
    }

    #[cfg(feature = "bench")]
    pub fn move_ctx(&mut self) -> MoveContext<'_> {
        MoveContext {
            zobrist: &mut self.zobrist,
            board: &mut self.board_state,
            search: &mut self.search_state,
        }
    }
}

impl std::fmt::Display for Milky {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f,)?;

        for rank in 0..8 {
            let mut line = String::with_capacity(20);
            line.push_str(&format!("  {} ", 8 - rank));

            for file in 0..8 {
                let square = Square::from_u64_unchecked(rank * 8 + file);
                let mut piece = String::from(".");

                for (idx, &board) in self.board_state.pieces.iter().enumerate() {
                    if !board.get_bit(square).is_empty() {
                        piece = Pieces::from_usize_unchecked(idx).to_string();
                        break;
                    }
                }

                line.push(' ');
                line.push_str(&piece);
            }

            writeln!(f, "{line}")?;
        }

        writeln!(f)?;
        writeln!(f, "     a b c d e f g h")?;
        writeln!(f)?;
        writeln!(f, "     Side:      {}", self.board_state.side_to_move)?;
        writeln!(f, "     Castling:   {}", self.board_state.castling_rights)?;
        writeln!(f, "     Enpassant:    {}", self.board_state.en_passant)?;
        writeln!(f, "     Zobrist key: {}", self.zobrist.position)?;
        writeln!(f)
    }
}
