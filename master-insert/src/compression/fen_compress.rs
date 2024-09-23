#![allow(dead_code)]
use shakmaty::{
    Bitboard, CastlingMode, CastlingSide, Chess, Color, Piece, Position, Rank, Role, Square,
};
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
    pub occupied: Bitboard,
    pub packed_state: Vec<u8>,
}

impl CompressedPosition {
    // The thinking behind the encoding
    // Occupied bitboard has bits set for
    // each square with a piece on it.
    // Each packedState byte holds 2 values (nibbles).
    // First one at low bits, second one at high bits.
    // Values correspond to consecutive squares
    // in bitboard iteration order.
    // Nibble values:
    // these are the same as for Piece
    // knights, bishops, queens can just be copied
    //  0 : white pawn
    //  1 : black pawn
    //  2 : white knight
    //  3 : black knight
    //  4 : white bishop
    //  5 : black bishop
    //  6 : white rook
    //  7 : black rook
    //  8 : white queen
    //  9 : black queen
    // 10 : white king
    // 11 : black king
    //
    // these are special
    // 12 : pawn with ep square behind (white or black, depending on rank)
    // 13 : white rook with coresponding castling rights
    // 14 : black rook with coresponding castling rights
    // 15 : black king and black is side to move
    //
    // Let N be the number of bits set in occupied bitboard.
    // Only N nibbles are present. (N+1)/2 bytes are initialized.

    pub fn compress(position: &Chess) -> CompressedPosition {
        let board = position.board();
        let occupied_bitboard = board.occupied();

        let en_passant_squares: Vec<Square> = position
            .en_passant_moves()
            .into_iter()
            .map(|ep| ep.to())
            .collect();

        let mut nibble_values = Vec::new();

        for square in occupied_bitboard {
            let piece = board.piece_at(square).unwrap();

            let mut nibble_value = match piece {
                Piece {
                    color: Color::White,
                    role: Role::Pawn,
                } => 0,
                Piece {
                    color: Color::Black,
                    role: Role::Pawn,
                } => 1,
                Piece {
                    color: Color::White,
                    role: Role::Knight,
                } => 2,
                Piece {
                    color: Color::Black,
                    role: Role::Knight,
                } => 3,
                Piece {
                    color: Color::White,
                    role: Role::Bishop,
                } => 4,
                Piece {
                    color: Color::Black,
                    role: Role::Bishop,
                } => 5,
                Piece {
                    color: Color::White,
                    role: Role::Rook,
                } => 6,
                Piece {
                    color: Color::Black,
                    role: Role::Rook,
                } => 7,
                Piece {
                    color: Color::White,
                    role: Role::Queen,
                } => 8,
                Piece {
                    color: Color::Black,
                    role: Role::Queen,
                } => 9,
                Piece {
                    color: Color::White,
                    role: Role::King,
                } => 10,
                Piece {
                    color: Color::Black,
                    role: Role::King,
                } => 11,
            };

            // Check for en passant pawn
            if piece.role == Role::Pawn
                && ((piece.color == Color::White && square.rank() == Rank::Fourth)
                    || (piece.color == Color::Black && square.rank() == Rank::Fifth))
            {
                let ep_check_square = match piece.color {
                    Color::White => Square::from_coords(square.file(), Rank::Third),
                    Color::Black => Square::from_coords(square.file(), Rank::Sixth),
                };
                if en_passant_squares.contains(&ep_check_square) {
                    nibble_value = 12; // Pawn with ep square behind
                }
            }

            // Rooks with corresponding castling rights
            if piece.role == Role::Rook {
                let castles = position.castles();

                let (kingside_rook, queenside_rook) = match piece.color {
                    Color::White => (
                        castles.rook(Color::White, CastlingSide::KingSide),
                        castles.rook(Color::White, CastlingSide::QueenSide),
                    ),
                    Color::Black => (
                        castles.rook(Color::Black, CastlingSide::KingSide),
                        castles.rook(Color::Black, CastlingSide::QueenSide),
                    ),
                };

                if Some(square) == kingside_rook || Some(square) == queenside_rook {
                    nibble_value = if piece.color == Color::White { 13 } else { 14 };
                }
            }

            // Black king and black to move
            if piece.role == Role::King
                && piece.color == Color::Black
                && position.turn() == Color::Black
            {
                nibble_value = 15;
            }

            nibble_values.push(nibble_value as u8);
        }

        // Calculate the number of bytes needed
        let n = nibble_values.len();
        let packed_bytes = (n + 1) / 2;

        // Pack nibbles into bytes
        let mut packed_state = Vec::with_capacity(packed_bytes);
        for i in 0..packed_bytes {
            let low_nibble = nibble_values[2 * i];
            let high_nibble = if 2 * i + 1 < n {
                nibble_values[2 * i + 1]
            } else {
                0
            };
            packed_state.push(low_nibble | (high_nibble << 4));
        }

        CompressedPosition {
            occupied: occupied_bitboard,
            packed_state,
        }
    }

    pub fn decompress(&self) -> Result<Chess, CompressedPositionError> {
        use shakmaty::fen::Fen;
        use std::collections::HashMap;
        use std::fmt::Write;

        let occupied_bitboard = self.occupied;
        let n = occupied_bitboard.count();

        // Extract nibbles from packed_state
        let mut nibble_values = Vec::with_capacity(n);
        for byte in &self.packed_state {
            let low_nibble = byte & 0x0F;
            let high_nibble = (byte >> 4) & 0x0F;
            nibble_values.push(low_nibble);
            if nibble_values.len() < n {
                nibble_values.push(high_nibble);
            }
        }

        let mut nibble_iter = nibble_values.into_iter();

        // Map squares to nibble values
        let mut square_nibbles = HashMap::new();
        for square in occupied_bitboard {
            if let Some(nibble_value) = nibble_iter.next() {
                square_nibbles.insert(square, nibble_value);
            } else {
                return Err(CompressedPositionError::InsufficientNibbles);
            }
        }

        let mut side_to_move = Color::White;
        let mut castling_rights = String::new();
        let mut en_passant_square = None;

        // Build the FEN string
        let mut fen = String::new();

        for rank in (0..8).rev() {
            if rank != 7 {
                fen.push('/');
            }
            let mut empty_count = 0;

            for file in 0..8 {
                let square_index = rank * 8u32 + file;
                let square = Square::new(square_index);
                if let Some(&nibble_value) = square_nibbles.get(&square) {
                    if empty_count > 0 {
                        write!(&mut fen, "{}", empty_count).unwrap();
                        empty_count = 0;
                    }

                    let (role, color) = match nibble_value {
                        0 => (Role::Pawn, Color::White),
                        1 => (Role::Pawn, Color::Black),
                        2 => (Role::Knight, Color::White),
                        3 => (Role::Knight, Color::Black),
                        4 => (Role::Bishop, Color::White),
                        5 => (Role::Bishop, Color::Black),
                        6 => (Role::Rook, Color::White),
                        7 => (Role::Rook, Color::Black),
                        8 => (Role::Queen, Color::White),
                        9 => (Role::Queen, Color::Black),
                        10 => (Role::King, Color::White),
                        11 => (Role::King, Color::Black),
                        12 => {
                            // Pawn with en passant square behind
                            let color = if rank >= 4 {
                                Color::Black
                            } else {
                                Color::White
                            };
                            let ep_square = match color {
                                Color::White => Square::from_coords(square.file(), Rank::Third),
                                Color::Black => Square::from_coords(square.file(), Rank::Sixth),
                            };
                            en_passant_square = Some(ep_square);
                            (Role::Pawn, color)
                        }
                        13 => {
                            // White rook with corresponding castling rights
                            if square == Square::A1 {
                                castling_rights.push('Q');
                            } else if square == Square::H1 {
                                castling_rights.push('K');
                            }
                            (Role::Rook, Color::White)
                        }
                        14 => {
                            // Black rook with corresponding castling rights
                            if square == Square::A8 {
                                castling_rights.push('q');
                            } else if square == Square::H8 {
                                castling_rights.push('k');
                            }
                            (Role::Rook, Color::Black)
                        }
                        15 => {
                            // Black king and black to move
                            side_to_move = Color::Black;
                            (Role::King, Color::Black)
                        }
                        _ => return Err(CompressedPositionError::InvalidNibbleValue(nibble_value)),
                    };

                    let piece_char = match (role, color) {
                        (Role::Pawn, Color::White) => 'P',
                        (Role::Pawn, Color::Black) => 'p',
                        (Role::Knight, Color::White) => 'N',
                        (Role::Knight, Color::Black) => 'n',
                        (Role::Bishop, Color::White) => 'B',
                        (Role::Bishop, Color::Black) => 'b',
                        (Role::Rook, Color::White) => 'R',
                        (Role::Rook, Color::Black) => 'r',
                        (Role::Queen, Color::White) => 'Q',
                        (Role::Queen, Color::Black) => 'q',
                        (Role::King, Color::White) => 'K',
                        (Role::King, Color::Black) => 'k',
                    };

                    fen.push(piece_char);
                } else {
                    empty_count += 1;
                }
            }
            if empty_count > 0 {
                write!(&mut fen, "{}", empty_count).unwrap();
            }
        }

        // Side to move
        fen.push(' ');
        fen.push(match side_to_move {
            Color::White => 'w',
            Color::Black => 'b',
        });

        // Castling rights
        if castling_rights.is_empty() {
            castling_rights.push('-');
        }

        fen.push(' ');
        fen.push_str(&castling_rights);

        // En passant
        fen.push(' ');
        if let Some(ep_square) = en_passant_square {
            write!(fen, "{}", ep_square).unwrap();
        }

        // Halfmove clock and fullmove number
        fen.push_str(" 0 1");

        // Parse the FEN string
        let position = Fen::from_ascii(fen.as_bytes())?.into_position(CastlingMode::Standard)?;
        Ok(position)
    }

    pub fn read_from_big_endian(
        data: &[u8],
    ) -> Result<CompressedPosition, CompressedPositionError> {
        if data.len() < 8 {
            return Err(CompressedPositionError::InsufficientDataForBitboard);
        }

        // Read the first 8 bytes as the occupied bitboard
        let occupied = u64::from_be_bytes(data[0..8].try_into().unwrap());
        let bitboard = Bitboard(occupied);

        // Count the number of bits set to determine the number of nibbles
        let n = bitboard.count();
        let packed_bytes = (n + 1) / 2;

        if data.len() < 8 + packed_bytes {
            return Err(CompressedPositionError::InsufficientDataForPackedState);
        }

        // Read the packed_state bytes
        let packed_state = data[8..8 + packed_bytes].to_vec();

        Ok(CompressedPosition {
            occupied: bitboard,
            packed_state,
        })
    }

    pub fn write_to_big_endian(&self, data: &mut Vec<u8>) {
        // Write the occupied bitboard
        data.extend_from_slice(&self.occupied.0.to_be_bytes());

        // Write the packed_state bytes
        data.extend_from_slice(&self.packed_state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::fen::Fen;

    #[test]
    fn test_compress_decompress_startpos() -> Result<(), CompressedPositionError> {
        let startpos = Chess::default();
        let cp = CompressedPosition::compress(&startpos);
        let decompressed = cp.decompress()?;
        assert_eq!(startpos, decompressed);
        Ok(())
    }

    #[test]
    fn test_compress_decompress_with_en_passant() -> Result<(), CompressedPositionError> {
        let fen = "rnbqkbnr/ppp1ppp1/7p/3pP3/8/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 3";
        let position = Fen::from_ascii(fen.as_bytes())?.into_position(CastlingMode::Standard)?;
        let cp = CompressedPosition::compress(&position);
        let decompressed = cp.decompress()?;
        assert_eq!(position, decompressed);
        Ok(())
    }

    #[test]
    fn test_read_write_big_endian() -> Result<(), CompressedPositionError> {
        let startpos = Chess::default();
        let cp = CompressedPosition::compress(&startpos);

        let mut data = Vec::new();
        cp.write_to_big_endian(&mut data);
        let cp_read = CompressedPosition::read_from_big_endian(&data)?;

        assert_eq!(cp, cp_read);
        Ok(())
    }

    #[test]
    fn test_insufficient_data_for_bitboard() {
        let data = vec![1, 2, 3]; // Not enough data for bitboard
        assert!(matches!(
            CompressedPosition::read_from_big_endian(&data),
            Err(CompressedPositionError::InsufficientDataForBitboard)
        ));
    }

    #[test]
    fn test_insufficient_data_for_packed_state() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9]; // Enough for bitboard, not enough for packed state
        assert!(matches!(
            CompressedPosition::read_from_big_endian(&data),
            Err(CompressedPositionError::InsufficientDataForPackedState)
        ));
    }

    #[test]
    fn test_insufficient_nibbles() -> Result<(), CompressedPositionError> {
        let mut cp = CompressedPosition::compress(&Chess::default());
        // Remove last byte to create insufficient nibbles
        cp.packed_state.pop();
        assert!(matches!(
            cp.decompress(),
            Err(CompressedPositionError::InsufficientNibbles)
        ));
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
        let invalid_position_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBKKBNR w KQkq - 0 1";
        let fen = Fen::from_ascii(invalid_position_fen.as_bytes())?;
        assert!(fen.into_position::<Chess>(CastlingMode::Standard).is_err());
        Ok(())
    }

    #[test]
    fn test_compress_decompress_complex_position() -> Result<(), CompressedPositionError> {
        let complex_fen = "r1bqk2r/pp1nbppp/2p1pn2/3p4/2PP4/2N1PN2/PP3PPP/R1BQK2R w KQkq - 0 7";
        let position =
            Fen::from_ascii(complex_fen.as_bytes())?.into_position(CastlingMode::Standard)?;
        let cp = CompressedPosition::compress(&position);
        let decompressed = cp.decompress()?;
        assert_eq!(position, decompressed);
        Ok(())
    }

    #[test]
    fn test_compress_decompress_with_castling_rights() -> Result<(), CompressedPositionError> {
        let fen_with_castling = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
        let position =
            Fen::from_ascii(fen_with_castling.as_bytes())?.into_position(CastlingMode::Standard)?;
        let cp = CompressedPosition::compress(&position);
        let decompressed = cp.decompress()?;
        assert_eq!(position, decompressed);
        Ok(())
    }
}
