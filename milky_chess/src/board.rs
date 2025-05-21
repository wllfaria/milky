use milky_bitboard::{BitBoard, CastlingRights, Pieces, Side, Square};

use crate::zobrist::{Zobrist, ZobristKey};
use crate::{
    BISHOP_ATTACKS, BISHOP_BLOCKERS, BISHOP_MAGIC_BITBOARDS, BISHOP_RELEVANT_OCCUPANCIES,
    KING_ATTACKS, KNIGHT_ATTACKS, MAX_REPETITIONS, PAWN_ATTACKS, ROOK_ATTACKS, ROOK_BLOCKERS,
    ROOK_MAGIC_BITBOARDS, ROOK_RELEVANT_OCCUPANCIES, attacks,
};

pub fn get_bishop_attacks(square: Square, mut occupancy: BitBoard) -> BitBoard {
    occupancy &= BISHOP_BLOCKERS.get().unwrap()[square];
    occupancy *= BISHOP_MAGIC_BITBOARDS[square];
    occupancy >>= (64 - BISHOP_RELEVANT_OCCUPANCIES[square as usize]) as u64;

    attacks!(BISHOP_ATTACKS)[square as usize][*occupancy as usize]
}

pub fn get_rook_attacks(square: Square, mut occupancy: BitBoard) -> BitBoard {
    occupancy &= ROOK_BLOCKERS.get().unwrap()[square];
    occupancy *= ROOK_MAGIC_BITBOARDS[square];
    occupancy >>= (64 - ROOK_RELEVANT_OCCUPANCIES[square as usize]) as u64;

    attacks!(ROOK_ATTACKS)[square as usize][*occupancy as usize]
}

pub fn get_queen_attacks(square: Square, occupancy: BitBoard) -> BitBoard {
    let bishop_occupancies = occupancy;
    let rook_occupancies = occupancy;

    let mut queen_attacks = get_bishop_attacks(square, bishop_occupancies);
    queen_attacks |= get_rook_attacks(square, rook_occupancies);

    queen_attacks
}

#[derive(Debug)]
pub struct BoardSnapshot {
    pub boards: [BitBoard; 12],
    pub occupancies: [BitBoard; 3],
    pub side_to_move: Side,
    pub en_passant: Square,
    pub castling_rights: CastlingRights,
    pub position_key: ZobristKey,
    pub fifty_move_counter: u8,
}

impl Default for BoardSnapshot {
    fn default() -> Self {
        Self {
            boards: [BitBoard::default(); 12],
            occupancies: [BitBoard::default(); 3],
            side_to_move: Side::White,
            en_passant: Square::OffBoard,
            castling_rights: CastlingRights::all(),
            position_key: ZobristKey::default(),
            fifty_move_counter: 0,
        }
    }
}

pub struct BoardState {
    pub pieces: [BitBoard; 12],
    pub occupancies: [BitBoard; 3],
    pub side_to_move: Side,
    pub en_passant: Square,
    pub castling_rights: CastlingRights,
    pub snapshots: Vec<BoardSnapshot>,
    pub fifty_move_counter: u8,
    pub ply: usize,
    pub repetition_table: [ZobristKey; MAX_REPETITIONS],
    pub repetition_index: usize,
}

impl Default for BoardState {
    fn default() -> Self {
        Self::new()
    }
}

impl BoardState {
    pub fn new() -> Self {
        Self {
            pieces: [BitBoard::default(); 12],
            occupancies: [BitBoard::default(); 3],
            side_to_move: Side::White,
            castling_rights: CastlingRights::all(),
            en_passant: Square::OffBoard,
            snapshots: vec![],
            ply: 0,
            repetition_table: [ZobristKey::default(); MAX_REPETITIONS],
            repetition_index: 0,
            fifty_move_counter: 0,
        }
    }

    pub fn snapshot_board(&mut self, zobrist: &mut Zobrist) {
        self.snapshots.push(BoardSnapshot {
            boards: self.pieces,
            occupancies: self.occupancies,
            side_to_move: self.side_to_move,
            en_passant: self.en_passant,
            castling_rights: self.castling_rights,
            position_key: zobrist.position,
            fifty_move_counter: self.fifty_move_counter,
        });
    }

    pub fn undo_move(&mut self) -> ZobristKey {
        let Some(snapshot) = self.snapshots.pop() else {
            panic!("Tried to undo_move with no snapshots on stack!");
        };

        self.pieces = snapshot.boards;
        self.occupancies = snapshot.occupancies;
        self.side_to_move = snapshot.side_to_move;
        self.en_passant = snapshot.en_passant;
        self.castling_rights = snapshot.castling_rights;
        self.fifty_move_counter = snapshot.fifty_move_counter;
        snapshot.position_key
    }

    pub fn record_repetition(&mut self, zobrist: &mut Zobrist) {
        self.repetition_index += 1;
        self.repetition_table[self.repetition_index] = zobrist.position;
    }

    pub fn reset(&mut self) {
        self.ply = 0;
        self.repetition_table = [ZobristKey::default(); MAX_REPETITIONS];
        self.repetition_index = 0;
    }

    pub fn is_square_attacked(&self, square: Square, side: Side) -> bool {
        let (
            pawn_side,
            pawn_board,
            knight_board,
            king_board,
            bishop_board,
            rook_board,
            queen_board,
        ) = match side {
            Side::White => (
                Side::Black,
                self.pieces[Pieces::WhitePawn],
                self.pieces[Pieces::WhiteKnight],
                self.pieces[Pieces::WhiteKing],
                self.pieces[Pieces::WhiteBishop],
                self.pieces[Pieces::WhiteRook],
                self.pieces[Pieces::WhiteQueen],
            ),
            Side::Black => (
                Side::White,
                self.pieces[Pieces::BlackPawn],
                self.pieces[Pieces::BlackKnight],
                self.pieces[Pieces::BlackKing],
                self.pieces[Pieces::BlackBishop],
                self.pieces[Pieces::BlackRook],
                self.pieces[Pieces::BlackQueen],
            ),
            _ => unreachable!(),
        };

        if attacks!(PAWN_ATTACKS)[pawn_side][square].is_attacked(pawn_board) {
            return true;
        }

        if attacks!(KNIGHT_ATTACKS)[square].is_attacked(knight_board) {
            return true;
        }

        if attacks!(KING_ATTACKS)[square].is_attacked(king_board) {
            return true;
        }

        let occupancy = self.occupancies[Side::Both];

        if get_bishop_attacks(square, occupancy).is_attacked(bishop_board) {
            return true;
        }

        if get_rook_attacks(square, occupancy).is_attacked(rook_board) {
            return true;
        }

        if get_queen_attacks(square, occupancy).is_attacked(queen_board) {
            return true;
        }

        false
    }
}
