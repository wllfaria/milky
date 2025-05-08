use milky_bitboard::{BitBoard, CastlingRights, Pieces, Side, Square};
use thiserror::Error;

type Result<R> = std::result::Result<R, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    MalformedFenString(String),
}

#[derive(Debug)]
struct UnparsedFenParts<'fen> {
    positions: &'fen str,
    side_to_move: &'fen str,
    castling_rights: &'fen str,
    en_passant: &'fen str,
    half_move_clock: Option<&'fen str>,
    full_move_counter: Option<&'fen str>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FenParts {
    pub positions: [BitBoard; 12],
    pub white_occupancy: BitBoard,
    pub black_occupancy: BitBoard,
    pub both_occupancy: BitBoard,
    pub side_to_move: Side,
    pub castling_rights: CastlingRights,
    pub en_passant: Square,
    pub half_move_clock: u32,
    pub full_move_counter: u32,
}

pub fn parse_fen_string(fen_string: &str) -> Result<FenParts> {
    let parts = split_fen_string(fen_string)?;

    let positions = parse_position(parts.positions);
    let side_to_move = parse_side_to_move(parts.side_to_move);
    let castling_rights = parse_castling_rights(parts.castling_rights)?;
    let en_passant = parse_en_passant(parts.en_passant)?;
    let half_move_clock = parse_half_move_clock(parts.half_move_clock)?;
    let full_move_counter = parse_full_move_counter(parts.full_move_counter)?;

    let white_occupancy = get_occupancy(positions, Side::White);
    let black_occupancy = get_occupancy(positions, Side::Black);
    let both_occupancy = get_occupancy(positions, Side::Both);

    Ok(FenParts {
        positions,
        side_to_move,
        castling_rights,
        en_passant,
        half_move_clock,
        full_move_counter,
        white_occupancy,
        black_occupancy,
        both_occupancy,
    })
}

fn split_fen_string<'fen>(fen_string: &'fen str) -> Result<UnparsedFenParts<'fen>> {
    if fen_string.is_empty() {
        return Err(Error::MalformedFenString(
            "FEN string cannot be empty".into(),
        ));
    }

    let mut parts = fen_string.trim().split(" ");

    let positions = parts
        .next()
        .ok_or(Error::MalformedFenString("Malformed FEN string".into()))?;

    let side_to_move = parts
        .next()
        .ok_or(Error::MalformedFenString("Malformed FEN string".into()))?;

    let castling_rights = parts
        .next()
        .ok_or(Error::MalformedFenString("Malformed FEN string".into()))?;

    let en_passant = parts
        .next()
        .ok_or(Error::MalformedFenString("Malformed FEN string".into()))?;

    let half_move_clock = parts.next();

    let full_move_counter = parts.next();

    Ok(UnparsedFenParts {
        positions,
        side_to_move,
        castling_rights,
        en_passant,
        half_move_clock,
        full_move_counter,
    })
}

fn parse_position(position: &str) -> [BitBoard; 12] {
    let mut boards = [BitBoard::default(); 12];

    let (mut rank, mut file) = (0, 0);

    for ch in position.chars() {
        let square = Square::from_u64_unchecked(rank * 8 + file);
        let mut skip = 1;

        match ch {
            'r' => boards[Pieces::BlackRook].set_bit(square),
            'b' => boards[Pieces::BlackBishop].set_bit(square),
            'n' => boards[Pieces::BlackKnight].set_bit(square),
            'q' => boards[Pieces::BlackQueen].set_bit(square),
            'k' => boards[Pieces::BlackKing].set_bit(square),
            'p' => boards[Pieces::BlackPawn].set_bit(square),
            'R' => boards[Pieces::WhiteRook].set_bit(square),
            'B' => boards[Pieces::WhiteBishop].set_bit(square),
            'N' => boards[Pieces::WhiteKnight].set_bit(square),
            'Q' => boards[Pieces::WhiteQueen].set_bit(square),
            'K' => boards[Pieces::WhiteKing].set_bit(square),
            'P' => boards[Pieces::WhitePawn].set_bit(square),
            '1'..='8' => skip = ch.to_digit(10).unwrap() as u64,
            '/' => {
                rank += 1;
                file = 0;
                continue;
            }
            _ => return boards,
        };

        file += skip;
    }

    boards
}

fn parse_side_to_move(side_to_move_str: &str) -> Side {
    match side_to_move_str {
        "w" => Side::White,
        "b" => Side::Black,
        _ => unreachable!(),
    }
}

fn parse_castling_rights(castling_rights_str: &str) -> Result<CastlingRights> {
    let mut castling_rights = CastlingRights::empty();

    if castling_rights_str == "-" {
        return Ok(castling_rights);
    }

    for ch in castling_rights_str.chars() {
        let side = match ch {
            'Q' => CastlingRights::WHITE_Q,
            'K' => CastlingRights::WHITE_K,
            'q' => CastlingRights::BLACK_Q,
            'k' => CastlingRights::BLACK_K,
            _ => return Err(Error::MalformedFenString("Malformed FEN string".into())),
        };

        castling_rights = castling_rights.union(side);
    }

    Ok(castling_rights)
}

fn parse_en_passant(en_passant_str: &str) -> Result<Square> {
    if en_passant_str == "-" {
        return Ok(Square::OffBoard);
    }

    Square::from_algebraic_str(en_passant_str).map_err(|e| Error::MalformedFenString(e.to_string()))
}

fn parse_half_move_clock(half_move_clock_str: Option<&str>) -> Result<u32> {
    let Some(value) = half_move_clock_str else {
        return Ok(0);
    };

    value
        .parse::<u32>()
        .map_err(|_| Error::MalformedFenString(format!("Invalid half move clock value: {value}")))
}

fn parse_full_move_counter(full_move_counter_str: Option<&str>) -> Result<u32> {
    let Some(value) = full_move_counter_str else {
        return Ok(1);
    };

    value
        .parse::<u32>()
        .map_err(|_| Error::MalformedFenString(format!("Invalid full move counter value: {value}")))
}

fn get_occupancy(positions: [BitBoard; 12], side: Side) -> BitBoard {
    let mut occupancy = BitBoard::default();

    match side {
        Side::White => {
            for &board in &positions[Pieces::white_pieces_range()] {
                occupancy |= board;
            }
        }
        Side::Black => {
            for &board in &positions[Pieces::black_pieces_range()] {
                occupancy |= board;
            }
        }
        Side::Both => {
            for &board in &positions[Pieces::range()] {
                occupancy |= board;
            }
        }
    };

    occupancy
}

#[cfg(test)]
mod tests {
    use std::fmt::{Display, Write};

    use super::*;

    static INITIAL_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    static POS_B: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 ";
    static POS_C: &str = "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1";

    struct FenStringSnapshot {
        fen_str: &'static str,
        board: String,
        white_occupancy: BitBoard,
        black_occupancy: BitBoard,
        both_occupancy: BitBoard,
        side_to_move: Side,
        castling_rights: CastlingRights,
        en_passant: Square,
        half_move_clock: u32,
        full_move_counter: u32,
    }

    impl FenStringSnapshot {
        fn from_fen(fen_str: &'static str, fen_parts: FenParts) -> Self {
            Self {
                fen_str,
                white_occupancy: fen_parts.white_occupancy,
                black_occupancy: fen_parts.black_occupancy,
                both_occupancy: fen_parts.both_occupancy,
                board: print_board(&fen_parts.positions),
                side_to_move: fen_parts.side_to_move,
                castling_rights: fen_parts.castling_rights,
                en_passant: fen_parts.en_passant,
                half_move_clock: fen_parts.half_move_clock,
                full_move_counter: fen_parts.full_move_counter,
            }
        }
    }

    impl Display for FenStringSnapshot {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            writeln!(f, "FEN: {}", self.fen_str)?;
            writeln!(f)?;
            writeln!(f, "{}", self.board)?;
            writeln!(f)?;
            writeln!(f, "Side: {}", self.side_to_move)?;
            writeln!(f, "Castling rights: {}", self.castling_rights)?;
            writeln!(f, "En passant: {}", self.en_passant)?;
            writeln!(f, "Half move: {}", self.half_move_clock)?;
            writeln!(f, "Full move: {}", self.full_move_counter)?;
            writeln!(f)?;
            writeln!(f, "White occupancy")?;
            writeln!(f, "{}", self.white_occupancy)?;
            writeln!(f)?;
            writeln!(f, "Black occupancy")?;
            writeln!(f, "{}", self.black_occupancy)?;
            writeln!(f)?;
            writeln!(f, "Both occupancy")?;
            writeln!(f, "{}", self.both_occupancy)?;

            Ok(())
        }
    }

    fn print_board(boards: &[BitBoard; 12]) -> String {
        let mut buffer = String::new();
        writeln!(buffer).unwrap();

        for rank in 0..8 {
            let mut line = String::with_capacity(20);
            line.push_str(&format!("  {} ", 8 - rank));

            for file in 0..8 {
                let square = Square::from_u64_unchecked(rank * 8 + file);
                let mut piece = String::from(".");

                for (idx, &board) in boards.iter().enumerate() {
                    if !board.get_bit(square).is_empty() {
                        piece = Pieces::from_usize_unchecked(idx).to_string();
                        break;
                    }
                }

                line.push(' ');
                line.push_str(&piece);
            }

            writeln!(buffer, "{line}").unwrap();
        }

        writeln!(buffer).unwrap();
        writeln!(buffer, "     a b c d e f g h").unwrap();
        buffer
    }

    #[test]
    fn test_initial_position() {
        let result = FenStringSnapshot::from_fen(
            INITIAL_POSITION,
            parse_fen_string(INITIAL_POSITION).unwrap(),
        );
        insta::assert_snapshot!(result)
    }

    #[test]
    fn test_position_b() {
        let result = FenStringSnapshot::from_fen(POS_B, parse_fen_string(POS_B).unwrap());
        insta::assert_snapshot!(result)
    }

    #[test]
    fn test_position_c() {
        let result = FenStringSnapshot::from_fen(POS_C, parse_fen_string(POS_C).unwrap());
        insta::assert_snapshot!(result)
    }
}
