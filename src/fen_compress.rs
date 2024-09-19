use shakmaty::{
    Bitboard, CastlingMode, CastlingSide, Chess, Color, Piece, Position, Rank, Role, Square,
};
#[derive(Debug, Clone, Copy)]
struct CompressedPosition {
    occupied: Bitboard,
    packed_state: [u8; 16],
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

    fn compress(position: &Chess) -> CompressedPosition {
        let mut cp = CompressedPosition {
            occupied: Bitboard::EMPTY,
            packed_state: [0u8; 16],
        };

        let board = position.board();
        let occupied_bitboard = board.occupied();
        cp.occupied = occupied_bitboard;

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

            //check for en passant pawn
            if piece.role == Role::Pawn
                && ((piece.color == Color::White && square.rank() == Rank::Third)
                    || (piece.color == Color::Black && square.rank() == Rank::Sixth))
            {
                let ep_check_square = match piece.color {
                    Color::White => Square::from_coords(square.file(), Rank::Second),
                    Color::Black => Square::from_coords(square.file(), Rank::Seventh),
                };
                if en_passant_squares.contains(&ep_check_square) {
                    nibble_value = 12; // Pawn with ep square behind
                }
            }

            // Rooks with corresponding castling rights
            // Rooks with corresponding castling rights
            if piece.role == Role::Rook {
                let castles = position.castles();
                let rook_square = Square::from(square);

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

                if Some(rook_square) == kingside_rook || Some(rook_square) == queenside_rook {
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

        // Pack nibbles into bytes
        let n = nibble_values.len();
        for i in 0..((n + 1) / 2) {
            let low_nibble = nibble_values[2 * i];
            let high_nibble = if 2 * i + 1 < n {
                nibble_values[2 * i + 1]
            } else {
                0
            };
            cp.packed_state[i] = low_nibble | (high_nibble << 4);
        }

        cp
    }

    /// Decompresses a `CompressedPosition` into a `Chess` position.
    fn decompress(&self) -> Chess {
        let occupied_bitboard = self.occupied;
        let n = occupied_bitboard.count();

        // Extract nibbles from packed_state
        let mut nibble_values = Vec::with_capacity(n);
        for i in 0..((n + 1) / 2) {
            let byte = self.packed_state[i];
            let low_nibble = byte & 0x0F;
            let high_nibble = (byte >> 4) & 0x0F;
            nibble_values.push(low_nibble);
            if nibble_values.len() < n {
                nibble_values.push(high_nibble);
            }
        }

        let mut board_builder = BoardBuilder::new();
        let mut en_passant_square = None;
        let mut white_castling_rights = CastlingRights::default();
        let mut black_castling_rights = CastlingRights::default();
        let mut side_to_move = Color::White;

        let mut nibble_iter = nibble_values.into_iter();

        for square in occupied_bitboard {
            let nibble_value = nibble_iter.next().unwrap();

            let (piece, color) = match nibble_value {
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
                    let color = if square.rank() as u8 >= 4 {
                        Color::Black
                    } else {
                        Color::White
                    };
                    let ep_square = match color {
                        Color::White => square.backward().unwrap(),
                        Color::Black => square.forward().unwrap(),
                    };
                    en_passant_square = Some(ep_square);
                    (Role::Pawn, color)
                }
                13 => {
                    // White rook with corresponding castling rights
                    if square == Square::A1 {
                        white_castling_rights.queenside = true;
                    } else if square == Square::H1 {
                        white_castling_rights.kingside = true;
                    }
                    (Role::Rook, Color::White)
                }
                14 => {
                    // Black rook with corresponding castling rights
                    if square == Square::A8 {
                        black_castling_rights.queenside = true;
                    } else if square == Square::H8 {
                        black_castling_rights.kingside = true;
                    }
                    (Role::Rook, Color::Black)
                }
                15 => {
                    // Black king and black to move
                    side_to_move = Color::Black;
                    (Role::King, Color::Black)
                }
                _ => panic!("Invalid nibble value: {}", nibble_value),
            };

            board_builder.set_piece_at(square, piece, color);
        }

        let board = board_builder.build().expect("Invalid board");

        let mut castles = CastlingMode::Standard.to_state();
        castles.white = white_castling_rights;
        castles.black = black_castling_rights;

        let en_passant = en_passant_square.map(|sq| shakmaty::EnPassant { to: sq });

        Chess {
            board,
            turn: side_to_move,
            castles,
            ep_square: en_passant,
            halfmove_clock: 0,
            fullmove_number: 1,
        }
    }

    /// Reads a `CompressedPosition` from a big-endian byte slice.
    fn read_from_big_endian(data: &[u8]) -> CompressedPosition {
        assert!(data.len() >= 24, "Data too short");
        let occupied = u64::from_be_bytes(data[0..8].try_into().unwrap());
        let mut packed_state = [0u8; 16];
        packed_state.copy_from_slice(&data[8..24]);
        CompressedPosition {
            occupied: Bitboard(occupied),
            packed_state,
        }
    }

    /// Writes the `CompressedPosition` to a mutable big-endian byte slice.
    fn write_to_big_endian(&self, data: &mut [u8]) {
        assert!(data.len() >= 24, "Data buffer too small");
        data[0..8].copy_from_slice(&self.occupied.0.to_be_bytes());
        data[8..24].copy_from_slice(&self.packed_state);
    }
}

use std::cmp::Ordering;

impl PartialEq for CompressedPosition {
    fn eq(&self, other: &Self) -> bool {
        self.occupied == other.occupied && self.packed_state == other.packed_state
    }
}

impl Eq for CompressedPosition {}

impl PartialOrd for CompressedPosition {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CompressedPosition {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.occupied.0.cmp(&other.occupied.0) {
            Ordering::Equal => self.packed_state.cmp(&other.packed_state),
            other => other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::{fen::Fen, Chess, Position};

    #[test]
    fn test_compress_decompress_startpos() {
        let startpos = Chess::default();
        let cp = CompressedPosition::compress(&startpos);
        let decompressed = cp.decompress();
        assert_eq!(startpos, decompressed);
    }

    #[test]
    fn test_compress_decompress_with_en_passant() {
        let fen = "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR b KQkq e6 0 2";
        let position: Chess = Fen::from_ascii(fen.as_bytes()).unwrap().position().unwrap();
        let cp = CompressedPosition::compress(&position);
        let decompressed = cp.decompress();
        assert_eq!(position, decompressed);
    }

    #[test]
    fn test_compress_decompress_with_castling_rights() {
        let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
        let position: Chess = Fen::from_ascii(fen.as_bytes()).unwrap().position().unwrap();
        let cp = CompressedPosition::compress(&position);
        let decompressed = cp.decompress();
        assert_eq!(position, decompressed);
    }

    #[test]
    fn test_compress_decompress_black_to_move() {
        let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR b KQkq - 0 1";
        let position: Chess = Fen::from_ascii(fen.as_bytes()).unwrap().position().unwrap();
        let cp = CompressedPosition::compress(&position);
        let decompressed = cp.decompress();
        assert_eq!(position, decompressed);
    }

    #[test]
    fn test_read_write_big_endian() {
        let startpos = Chess::default();
        let cp = CompressedPosition::compress(&startpos);

        let mut data = [0u8; 24];
        cp.write_to_big_endian(&mut data);
        let cp_read = CompressedPosition::read_from_big_endian(&data);

        assert_eq!(cp, cp_read);
    }
}
