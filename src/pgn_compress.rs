// src/pgn_compress.rs
use crate::huffman_code::get_huffman_code;
use crate::psqt::piece_value;
use bit_vec::BitVec;
use huffman_compress::{Book, Tree};
use shakmaty::{
    attacks::{self},
    san::{San, SanPlus, Suffix},
    Chess, Move, Position, Square,
};
use std::cmp::Ordering;

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

    pub fn encode(&self, pgn_moves: &[&str]) -> Option<BitVec> {
        let mut buffer = BitVec::new();
        let mut board = Chess::default();

        for pgn_move in pgn_moves {
            let san_plus: SanPlus = pgn_move.parse().ok()?;
            let mv = san_plus.san.to_move(&board).ok()?;

            let legal_moves = board.legal_moves();
            let mut scored_moves: Vec<ScoredMove> = legal_moves
                .into_iter()
                .map(|legal_mv| ScoredMove::new(&board, legal_mv))
                .collect();
            scored_moves.sort();

            let index = scored_moves.iter().position(|sm| sm.mv == mv)? as u32;

            self.codebook.encode(&mut buffer, &index).unwrap();

            board = board.play(&mv).ok()?;
        }

        Some(buffer)
    }

    pub fn decode(&self, data: &BitVec, plies: usize) -> Option<Vec<String>> {
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

            let index = decoder.next()? as usize;

            if index >= scored_moves.len() {
                return None;
            }

            let sm = &scored_moves[index];

            let san = San::from_move(&board, &sm.mv);
            board = board.play(&sm.mv).ok()?;
            let suffix = Suffix::from_position(&board);
            let san_plus = SanPlus { san, suffix };

            output.push(format!("{}", san_plus));
        }

        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_encode_decode_improved() {
        let pgn_moves = vec![
            "e4", "c5", "Nf3", "d6", "Bb5+", "Bd7", "Bxd7+", "Nxd7", "O-O", "Ngf6", "Re1", "e6",
            "d4", "cxd4", "Nxd4", "Be7", "c4", "a6", "Nc3", "O-O", "Be3", "Rc8", "b3", "e5", "Nf5",
            "b5", "Nxd6", "Bxd6", "Qxd6", "bxc4", "b4", "a5", "a3", "Re8", "Rad1", "Re6", "Qd2",
            "Rb8", "Bg5", "Qb6", "Be3", "Qb7", "f3", "axb4", "axb4", "Qxb4", "Rb1", "Qd6", "Rxb8+",
            "Qxb8", "Rb1", "Qc7", "Nd5", "Nxd5", "exd5", "Rd6", "f4", "c3", "Qc2", "Rxd5", "fxe5",
            "Nxe5", "Qxc3", "h6",
        ];

        // Step 1: Calculate the original size of the PGN moves in bytes
        // Join the moves with spaces to simulate a PGN string
        let original_pgn = pgn_moves.join(" ");
        let original_size_bytes = original_pgn.len();
        println!("Original PGN size: {} bytes", original_size_bytes);

        // Step 2: Initialize the encoder
        let encoder = Encoder::new();

        // Step 3: Measure the time taken to encode the moves
        let encode_start = Instant::now();
        let compressed = encoder.encode(&pgn_moves).expect("Encoding failed");
        let encode_duration = encode_start.elapsed();
        println!("Encoding time: {:.2?}", encode_duration);

        // Step 4: Calculate the size of the compressed data in bits and bytes
        let compressed_size_bits = compressed.len();
        let compressed_size_bytes = (compressed_size_bits + 7) / 8; // Round up to nearest byte
        println!(
            "Compressed data size: {} bits ({} bytes)",
            compressed_size_bits, compressed_size_bytes
        );

        // Step 5: Calculate the compression ratio
        let compression_ratio = (original_size_bytes as f64) / (compressed_size_bytes as f64);
        println!("Compression ratio: {:.2}%", compression_ratio * 100.0);

        // Step 6: Measure the time taken to decode the moves
        let decode_start = Instant::now();
        let decoded_moves = encoder
            .decode(&compressed, pgn_moves.len())
            .expect("Decoding failed");
        let decode_duration = decode_start.elapsed();
        println!("Decoding time: {:.2?}", decode_duration);

        // Step 7: Verify that the original and decoded moves match
        assert_eq!(pgn_moves, decoded_moves);

        // Optional: Ensure that the compressed size is indeed smaller than the original
        assert!(
            compressed_size_bits < original_size_bytes * 8,
            "Compression did not reduce the size"
        );
    }
}
