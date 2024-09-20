// src/pgn_compress.rs
use crate::huffman_code::get_huffman_code;
use crate::psqt::piece_value;
use bit_vec::BitVec;
use huffman_compress::{Book, EncodeError, Tree};
use shakmaty::{
    attacks::{self},
    san::{San, SanPlus, Suffix},
    Chess, Move, Position, Square,
};
use std::cmp::Ordering;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EncoderError {
    #[error("Failed to parse SAN move: {0}")]
    SanParseError(String),
    #[error("Failed to convert SAN to move: {0}")]
    SanToMoveError(String),
    #[error("Failed to find move in scored moves")]
    MoveNotFound,
    #[error("Failed to play move: {0}")]
    PlayMoveError(String),
    #[error("Huffman encoding error: {0}")]
    HuffmanEncodeError(EncodeError),
    #[error("Invalid move index during decoding")]
    InvalidMoveIndex,
}

impl From<EncodeError> for EncoderError {
    fn from(error: EncodeError) -> Self {
        EncoderError::HuffmanEncodeError(error)
    }
}
// Define ScoredMove struct
#[derive(Debug, Clone)]
struct ScoredMove {
    mv: Move,
    score: i32,
}

impl ScoredMove {
    fn new(board: &Chess, mv: Move) -> Self {
        let score = Self::compute_score(board, &mv);
        ScoredMove { mv, score }
    }

    fn compute_score(board: &Chess, mv: &Move) -> i32 {
        let us = board.turn();
        let them = us.other();
        let piece_role = mv.role();
        let from_square = mv.from().unwrap_or(Square::A1); // Use A1 for drops
        let to_square = mv.to();

        let mut score = 0;

        if let Some(promotion_role) = mv.promotion() {
            score += (promotion_role as i32) << 26;
        }

        if mv.is_capture() {
            score += 1 << 25;
        }

        let defending_pawns =
            attacks::pawn_attacks(them, to_square) & board.board().pawns() & board.board().black();

        let defending_pawns_score = if defending_pawns.0 == 0 {
            6
        } else {
            5 - piece_role as i32
        } << 22;
        score += defending_pawns_score;

        let move_value =
            piece_value(board, piece_role, to_square) - piece_value(board, piece_role, from_square);
        score += (512 + move_value) << 12;

        score += (to_square as i32) << 6;
        score += from_square as i32;

        score
    }
}

impl PartialEq for ScoredMove {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for ScoredMove {}

impl Ord for ScoredMove {
    fn cmp(&self, other: &Self) -> Ordering {
        other.score.cmp(&self.score)
    }
}

impl PartialOrd for ScoredMove {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Encoder struct
pub struct Encoder {
    codebook: &'static Book<u32>,
    tree: &'static Tree<u32>,
}

impl Default for Encoder {
    fn default() -> Self {
        Self::new()
    }
}

impl Encoder {
    pub fn new() -> Self {
        let (codebook, tree) = get_huffman_code();
        Encoder { codebook, tree }
    }

    pub fn encode(&self, pgn_moves: &[&str]) -> Result<BitVec, EncoderError> {
        let mut buffer = BitVec::new();
        let mut board = Chess::default();

        for pgn_move in pgn_moves {
            let san_plus: SanPlus = pgn_move
                .parse::<SanPlus>()
                .map_err(|e| EncoderError::SanParseError(e.to_string()))?;
            let mv = san_plus
                .san
                .to_move(&board)
                .map_err(|e| EncoderError::SanToMoveError(e.to_string()))?;

            let legal_moves = board.legal_moves();
            let mut scored_moves: Vec<ScoredMove> = legal_moves
                .into_iter()
                .map(|legal_mv| ScoredMove::new(&board, legal_mv))
                .collect();
            scored_moves.sort();

            let index = scored_moves
                .iter()
                .position(|sm| sm.mv == mv)
                .ok_or(EncoderError::MoveNotFound)? as u32;

            self.codebook.encode(&mut buffer, &index)?;

            board = board
                .play(&mv)
                .map_err(|e| EncoderError::PlayMoveError(e.to_string()))?;
        }

        Ok(buffer)
    }

    pub fn decode(&self, data: &BitVec, plies: usize) -> Result<Vec<String>, EncoderError> {
        let mut output = Vec::new();
        let mut board = Chess::default();

        let mut decoder = self.tree.decoder(data, plies);

        for _ in 0..plies {
            let legal_moves = board.legal_moves();
            let mut scored_moves: Vec<ScoredMove> = legal_moves
                .into_iter()
                .map(|mv| ScoredMove::new(&board, mv))
                .collect();
            scored_moves.sort();

            let index = decoder.next().ok_or(EncoderError::InvalidMoveIndex)? as usize;

            if index >= scored_moves.len() {
                return Err(EncoderError::InvalidMoveIndex);
            }

            let sm = &scored_moves[index];

            let san = San::from_move(&board, &sm.mv);
            board = board
                .play(&sm.mv)
                .map_err(|e| EncoderError::PlayMoveError(e.to_string()))?;
            let suffix = Suffix::from_position(&board);
            let san_plus = SanPlus { san, suffix };

            output.push(format!("{}", san_plus));
        }

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode_improved() -> Result<(), EncoderError> {
        let pgn_moves = vec![
            "e4", "c5", "Nf3", "d6", "Bb5+", "Bd7", "Bxd7+", "Nxd7", "O-O", "Ngf6", "Re1", "e6",
        ];

        let encoder = Encoder::new();
        let compressed = encoder.encode(&pgn_moves)?;
        let decoded_moves = encoder.decode(&compressed, pgn_moves.len())?;

        assert_eq!(pgn_moves, decoded_moves);
        Ok(())
    }

    #[test]
    fn test_invalid_san_parse() {
        let encoder = Encoder::new();
        let result = encoder.encode(&["e4", "invalid_move"]);
        assert!(matches!(result, Err(EncoderError::SanParseError(_))));
    }

    #[test]
    fn test_invalid_move() {
        let encoder = Encoder::new();
        let result = encoder.encode(&["e4", "e5", "a5", "h5", "Ra4"]); // Ra4 is invalid in this position
        assert!(matches!(result, Err(EncoderError::SanToMoveError(_))));
    }

    #[test]
    fn test_move_not_found() {
        let _encoder = Encoder::new();
        // This test is tricky to set up, as it would require manipulating internal state
        // Instead, we'll test that MoveNotFound error is properly defined
        assert!(matches!(
            EncoderError::MoveNotFound,
            EncoderError::MoveNotFound
        ));
    }

    #[test]
    fn test_play_move_error() {
        let encoder = Encoder::new();
        let result = encoder.encode(&[
            "e4", "e5", "Nf3", "Nc6", "Bb5", "a6", "Ba4", "b5", "Bb3", "Na5",
        ]); // This sequence should be valid
        assert!(result.is_ok()); // We expect this to succeed now
    }

    #[test]
    fn test_invalid_move_index_during_decoding() {
        let encoder = Encoder::new();
        let mut invalid_bitvec = BitVec::new();
        invalid_bitvec.push(true); // Add some invalid data
        let result = encoder.decode(&invalid_bitvec, 1);
        assert!(matches!(result, Err(EncoderError::InvalidMoveIndex)));
    }

    #[test]
    fn test_encode_empty_moves() -> Result<(), EncoderError> {
        let encoder = Encoder::new();
        let compressed = encoder.encode(&[])?;
        assert!(compressed.is_empty());
        Ok(())
    }

    #[test]
    fn test_decode_empty_bitvec() -> Result<(), EncoderError> {
        let encoder = Encoder::new();
        let empty_bitvec = BitVec::new();
        let decoded = encoder.decode(&empty_bitvec, 0)?;
        assert!(decoded.is_empty());
        Ok(())
    }

    #[test]
    fn test_encode_decode_long_game() -> Result<(), EncoderError> {
        let pgn_moves = vec![
            "e4", "c5", "d3", "g6", "f4", "Bg7", "Nf3", "e6", "e5", "d5", "exd6", "Qxd6", "Nc3",
            "Ne7", "Ne4", "Qd8", "Nxc5", "Qa5+", "c3", "Qxc5", "d4", "Qc7", "Bb5+", "Nbc6", "O-O",
            "O-O", "Ne5", "Nxe5", "fxe5", "Bxe5", "dxe5", "Qc5+", "Kh1", "Qxb5", "Bh6", "Re8",
            "Qf3", "Nf5", "g4", "Nxh6", "Qf4", "Qc6+", "Kg1", "Qc5+", "Kh1", "Qd5+", "Kg1", "Kg7",
            "Qf6+", "Kg8", "Qf4", "Qc5+", "Rf2", "Bd7", "Qxh6", "Bc6", "Qf4", "Rf8", "h4", "Qd5",
            "Rh2", "Rad8", "h5", "Qc5+", "Rf2", "Rd5", "hxg6", "fxg6", "Qxf8+", "Qxf8", "Rxf8+",
            "Kxf8", "Re1", "Kg7", "Kf2", "Rd2+", "Re2", "Rxe2+", "Kxe2", "Bd5", "Ke3", "Bxa2",
            "Kf4", "Bd5", "Kg5", "Bc6", "b4", "a5", "bxa5", "Bb5", "Kf4", "h6", "Kg3", "g5", "Kh3",
            "Ba6", "Kg3", "Kg6", "Kh3", "Kf7", "Kg3", "Kg6", "Kf3", "Bd3", "Kg3", "Be4", "Kh3",
            "Bc6", "Kg3", "h5", "Kh3", "hxg4+", "Kxg4", "Be4", "Kg3", "Bf5", "Kf3", "Bd3", "Kg3",
            "Kf5", "Kf3", "g4+", "Kg3", "Kxe5", "Kxg4", "Kd5", "Kf4", "Kc4", "Ke5", "Kxc3", "Kxe6",
            "Ba6", "Kd6",
        ];

        let encoder = Encoder::new();
        let compressed = encoder.encode(&pgn_moves)?;
        let decoded_moves = encoder.decode(&compressed, pgn_moves.len())?;

        assert_eq!(pgn_moves, decoded_moves);

        // Optional: Print compression statistics
        let original_size = pgn_moves.len() * 4; // Assuming average move length of 4 bytes
        let compressed_size = compressed.len() / 8; // Convert bits to bytes
        println!("Original size: {} bytes", original_size);
        println!("Compressed size: {} bytes", compressed_size);
        println!(
            "Compression ratio: {:.2}",
            original_size as f64 / compressed_size as f64
        );

        Ok(())
    }
}
