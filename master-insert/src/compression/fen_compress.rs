#![allow(dead_code)]
use shakmaty::{CastlingMode, CastlingSide, Chess, Color, File, Position, Rank, Role, Square};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompressedPositionError {
    #[error("Not enough nibble values for occupied squares")]
    InsufficientNibbles,
    #[error("Invalid nibble value: {0}")]
    InvalidNibbleValue(u8),
    #[error("Data too short for occupied bitboard")]
    InsufficientDataForBitboard,
    #[error("Data too short for packed_state")]
    InsufficientDataForPackedState,
    #[error("FEN parsing error: {0}")]
    FenParseError(#[from] shakmaty::fen::ParseFenError),
    #[error("Position conversion error: {0}")]
    PositionConversionError(#[from] shakmaty::PositionError<Chess>),
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CompressedPosition {
    pub compressed: [u8; 32], // 64 nibbles
}

//note - does not work on fake positions. positions must be legal!
impl CompressedPosition {
    pub fn compress(position: &Chess) -> [u8; 32] {
        let board = position.board();
        let mut compressed = [0u8; 32];
        let en_passant_squares: Vec<Square> = position
            .en_passant_moves()
            .into_iter()
            .map(|ep| ep.to())
            .collect();
        for rank in (0..8).rev() {
            for file in 0..8 {
                let square = Square::from_coords(File::new(file), Rank::new(rank));
                let nibble_index = rank * 8 + file;
                let byte_index = nibble_index / 2;
                let is_high_nibble = nibble_index % 2 != 0;

                let nibble_value = if let Some(piece) = board.piece_at(square) {
                    match (piece.color, piece.role) {
                        (Color::White, Role::Pawn) => 1,
                        (Color::Black, Role::Pawn) => 2,
                        (Color::White, Role::Knight) => 3,
                        (Color::Black, Role::Knight) => 4,
                        (Color::White, Role::Bishop) => 5,
                        (Color::Black, Role::Bishop) => 6,
                        (Color::White, Role::Rook) => 7,
                        (Color::Black, Role::Rook) => 8,
                        (Color::White, Role::Queen) => 9,
                        (Color::Black, Role::Queen) => 10,
                        (Color::White, Role::King) => 11,
                        (Color::Black, Role::King) => 12,
                    }
                } else {
                    0 // Empty square
                };

                // Special cases
                if nibble_value != 0 {
                    // Check for en passant pawns -- will error if pawn is on first or last rank
                    if ((nibble_value == 1 || nibble_value == 2)
                        && (square.rank() != Rank::First || square.rank() != Rank::Eighth))
                        && (en_passant_squares.contains(&Square::from_coords(
                            square.file(),
                            square.rank().offset(1).unwrap(),
                        )) || en_passant_squares.contains(&Square::from_coords(
                            square.file(),
                            square.rank().offset(-1).unwrap(),
                        )))
                    {
                        compressed[byte_index as usize] |=
                            13 << (if is_high_nibble { 4 } else { 0 });
                        continue;
                    }

                    // Check for rooks with castling rights
                    if nibble_value == 7 || nibble_value == 8 {
                        let castles = position.castles();
                        let color = if nibble_value == 7 {
                            Color::White
                        } else {
                            Color::Black
                        };
                        if Some(square) == castles.rook(color, CastlingSide::KingSide)
                            || Some(square) == castles.rook(color, CastlingSide::QueenSide)
                        {
                            compressed[byte_index as usize] |=
                                14 << (if is_high_nibble { 4 } else { 0 });
                            continue;
                        }
                    }

                    // Check for black king when it's black's turn
                    if nibble_value == 12 && position.turn() == Color::Black {
                        compressed[byte_index as usize] |=
                            15 << (if is_high_nibble { 4 } else { 0 });
                        continue;
                    }
                }

                // Set the nibble in the byte
                if is_high_nibble {
                    compressed[byte_index as usize] |= nibble_value << 4;
                } else {
                    compressed[byte_index as usize] |= nibble_value;
                }
            }
        }

        compressed
    }

    pub fn decompress(compressed: &[u8; 32]) -> Result<Chess, CompressedPositionError> {
        println!("Decompressing...");
        use shakmaty::fen::Fen;
        use std::fmt::Write;

        let mut fen = String::new();
        let mut side_to_move = Color::White;
        let mut castling_rights = String::new();
        let mut en_passant_square = None;

        for rank in (0..8).rev() {
            if rank != 7 {
                fen.push('/');
            }
            let mut empty_count = 0;

            for file in 0..8 {
                let nibble_index = rank * 8 + file;
                let byte_index = nibble_index / 2;
                let is_high_nibble = nibble_index % 2 != 0;

                let nibble_value = if is_high_nibble {
                    (compressed[byte_index] >> 4) & 0x0F
                } else {
                    compressed[byte_index] & 0x0F
                };

                let square = Square::from_coords(File::new(file as u32), Rank::new(rank as u32));

                match nibble_value {
                    0 => empty_count += 1,
                    1 => {
                        if empty_count > 0 {
                            write!(fen, "{}", empty_count).unwrap();
                            empty_count = 0;
                        }
                        fen.push('P');
                    }
                    2 => {
                        if empty_count > 0 {
                            write!(fen, "{}", empty_count).unwrap();
                            empty_count = 0;
                        }
                        fen.push('p');
                    }
                    3 => {
                        if empty_count > 0 {
                            write!(fen, "{}", empty_count).unwrap();
                            empty_count = 0;
                        }
                        fen.push('N');
                    }
                    4 => {
                        if empty_count > 0 {
                            write!(fen, "{}", empty_count).unwrap();
                            empty_count = 0;
                        }
                        fen.push('n');
                    }
                    5 => {
                        if empty_count > 0 {
                            write!(fen, "{}", empty_count).unwrap();
                            empty_count = 0;
                        }
                        fen.push('B');
                    }
                    6 => {
                        if empty_count > 0 {
                            write!(fen, "{}", empty_count).unwrap();
                            empty_count = 0;
                        }
                        fen.push('b');
                    }
                    7 => {
                        if empty_count > 0 {
                            write!(fen, "{}", empty_count).unwrap();
                            empty_count = 0;
                        }
                        fen.push('R');
                    }
                    8 => {
                        if empty_count > 0 {
                            write!(fen, "{}", empty_count).unwrap();
                            empty_count = 0;
                        }
                        fen.push('r');
                    }
                    9 => {
                        if empty_count > 0 {
                            write!(fen, "{}", empty_count).unwrap();
                            empty_count = 0;
                        }
                        fen.push('Q');
                    }
                    10 => {
                        if empty_count > 0 {
                            write!(fen, "{}", empty_count).unwrap();
                            empty_count = 0;
                        }
                        fen.push('q');
                    }
                    11 => {
                        if empty_count > 0 {
                            write!(fen, "{}", empty_count).unwrap();
                            empty_count = 0;
                        }
                        fen.push('K');
                    }
                    12 => {
                        if empty_count > 0 {
                            write!(fen, "{}", empty_count).unwrap();
                            empty_count = 0;
                        }
                        fen.push('k');
                    }
                    //enter special cases
                    13 => {
                        if empty_count > 0 {
                            write!(fen, "{}", empty_count).unwrap();
                            empty_count = 0;
                        }
                        fen.push(if rank >= 4 { 'p' } else { 'P' });
                        en_passant_square = Some(Square::from_coords(
                            square.file(),
                            if rank >= 4 { Rank::Sixth } else { Rank::Third },
                        ));
                    }
                    14 => {
                        if empty_count > 0 {
                            write!(fen, "{}", empty_count).unwrap();
                            empty_count = 0;
                        }
                        if rank == 0 {
                            fen.push('R');
                            castling_rights.push(if file == 0 { 'Q' } else { 'K' });
                        } else {
                            fen.push('r');
                            castling_rights.push(if file == 0 { 'q' } else { 'k' });
                        }
                    }
                    15 => {
                        if empty_count > 0 {
                            write!(fen, "{}", empty_count).unwrap();
                            empty_count = 0;
                        }
                        fen.push('k');
                        side_to_move = Color::Black;
                    }
                    _ => return Err(CompressedPositionError::InvalidNibbleValue(nibble_value)),
                }
            }
            if empty_count > 0 {
                write!(fen, "{}", empty_count).unwrap();
            }
        }

        // Side to move
        fen.push(' ');
        fen.push(if side_to_move == Color::White {
            'w'
        } else {
            'b'
        });

        // Castling rights
        fen.push(' ');
        if castling_rights.is_empty() {
            fen.push('-');
        } else {
            fen.push_str(&castling_rights);
        }

        // En passant
        fen.push(' ');
        if let Some(ep_square) = en_passant_square {
            write!(fen, "{}", ep_square).unwrap();
        } else {
            fen.push('-');
        }

        // Halfmove clock and fullmove number
        fen.push_str(" 0 1");

        // Parse the FEN string
        let position = Fen::from_ascii(fen.as_bytes())?.into_position(CastlingMode::Standard)?;
        Ok(position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::fen::Fen;

    #[test]
    fn test_compress_decompress_startpos() -> Result<(), CompressedPositionError> {
        let startpos = Chess::default();
        let compressed = CompressedPosition::compress(&startpos);
        let decompressed = CompressedPosition::decompress(&compressed)?;
        assert_eq!(startpos, decompressed);
        Ok(())
    }

    #[test]
    fn test_compress_decompress_with_en_passant() -> Result<(), CompressedPositionError> {
        let fen = "rnbqkbnr/ppp1ppp1/7p/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6";
        let position = Fen::from_ascii(fen.as_bytes())?.into_position(CastlingMode::Standard)?;
        let compressed = CompressedPosition::compress(&position);
        let decompressed = CompressedPosition::decompress(&compressed)?;
        assert_eq!(position, decompressed);
        Ok(())
    }

    #[test]
    fn test_compress_decompress_complex_position() -> Result<(), CompressedPositionError> {
        let complex_fen = "r1bqk2r/pp1nbppp/2p1pn2/3p4/2PP4/2N1PN2/PP3PPP/R1BQK2R w KQkq -";
        let position =
            Fen::from_ascii(complex_fen.as_bytes())?.into_position(CastlingMode::Standard)?;
        let compressed = CompressedPosition::compress(&position);
        let decompressed = CompressedPosition::decompress(&compressed)?;
        assert_eq!(position, decompressed);
        Ok(())
    }

    #[test]
    fn test_compress_decompress_with_castling_rights() -> Result<(), CompressedPositionError> {
        let fen_with_castling = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq -";
        let position =
            Fen::from_ascii(fen_with_castling.as_bytes())?.into_position(CastlingMode::Standard)?;
        let compressed = CompressedPosition::compress(&position);
        let decompressed = CompressedPosition::decompress(&compressed)?;
        assert_eq!(position, decompressed);
        Ok(())
    }

    #[test]
    fn test_compress_decompress_black_to_move() -> Result<(), CompressedPositionError> {
        let fen_black_to_move = "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3";
        let position =
            Fen::from_ascii(fen_black_to_move.as_bytes())?.into_position(CastlingMode::Standard)?;
        let compressed = CompressedPosition::compress(&position);
        let decompressed = CompressedPosition::decompress(&compressed)?;
        assert_eq!(position, decompressed);
        Ok(())
    }

    #[test]
    fn test_compress_decompress_no_castling_rights() -> Result<(), CompressedPositionError> {
        let fen_no_castling = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w - -";
        let position =
            Fen::from_ascii(fen_no_castling.as_bytes())?.into_position(CastlingMode::Standard)?;
        let compressed = CompressedPosition::compress(&position);
        let decompressed = CompressedPosition::decompress(&compressed)?;
        assert_eq!(position, decompressed);
        Ok(())
    }

    #[test]
    fn test_fen_parse_error() {
        let invalid_fen = "invalid fen string";
        assert!(Fen::from_ascii(invalid_fen.as_bytes()).is_err());
    }

    #[test]
    fn test_position_conversion_error() -> Result<(), CompressedPositionError> {
        // Create an invalid FEN with two white kings
        let invalid_position_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBKKBNR w KQkq -";
        let fen = Fen::from_ascii(invalid_position_fen.as_bytes())?;
        assert!(fen.into_position::<Chess>(CastlingMode::Standard).is_err());
        Ok(())
    }
}
