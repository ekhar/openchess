// src/pgn_compress.rs
use crate::huffman_code::get_huffman_code;
use bit_vec::BitVec;
use huffman_compress::{Book, Tree};
use pgn_reader::San;
use shakmaty::{
    attacks::{self},
    san::SanPlus,
    Chess, Color, Move, Position, Role, Square,
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

// Piece-Square Table (PSQT)
static PSQT: [[i32; 64]; 6] = [
    // Pawn
    [
        0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5,
        5, 10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 21, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10,
        -31, -31, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
    ],
    // Knight
    [
        -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15,
        10, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15,
        15, 11, 5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
    ],
    // Bishop
    [
        -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5,
        0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10,
        10, 10, -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
    ],
    // Rook
    [
        0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0,
        0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0,
        -5, 0, 0, 0, 5, 5, 0, 0, 0,
    ],
    // Queen
    [
        -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0,
        -10, -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0,
        5, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
    ],
    // King
    [
        -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40,
        -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40,
        -40, -30, -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 0,
        30, 10, 0, 0, 10, 30, 0,
    ],
];

fn piece_value(board: &Chess, role: Role, square: Square) -> i32 {
    let us = board.turn();
    let index = if us == Color::White {
        square as u8
    } else {
        mirror_square(square as u8)
    };
    PSQT[role as usize - 1][index as usize]
}

fn mirror_square(sq_index: u8) -> u8 {
    sq_index ^ 56
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

            output.push(format!("{}", san));

            board = board.play(&sm.mv).ok()?;
        }

        Some(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let pgn_moves = vec![
            "e4", "c5", "Nf3", "d6", "Bb5+", "Bd7", "Bxd7+", "Nxd7", "O-O", "Ngf6", "Re1", "e6",
            "d4", "cxd4", "Nxd4", "Be7", "c4", "a6", "Nc3", "O-O", "Be3", "Rc8", "b3", "e5", "Nf5",
            "b5", "Nxd6", "Bxd6", "Qxd6", "bxc4", "b4", "a5", "a3", "Re8", "Rad1", "Re6", "Qd2",
            "Rb8", "Bg5", "Qb6", "Be3", "Qb7", "f3", "axb4", "axb4", "Qxb4", "Rb1", "Qd6", "Rxb8+",
            "Qxb8", "Rb1", "Qc7", "Nd5", "Nxd5", "exd5", "Rd6", "f4", "c3", "Qc2", "Rxd5", "fxe5",
            "Nxe5", "Qxc3", "h6",
        ];

        let encoder = Encoder::new();

        // Encode the moves
        let compressed = encoder.encode(&pgn_moves).unwrap();
        println!("Compressed data size: {} bytes", compressed.len());

        // Decode the moves
        let decoded_moves = encoder.decode(&compressed, pgn_moves.len()).unwrap();

        // Verify that the original and decoded moves match
        assert_eq!(pgn_moves, decoded_moves);
    }
}
