// src/pgn_compress.rs
use crate::huffman_code::get_huffman_code;
use huffman_compress::{Book, CodeBuilder, Tree};
use shakmaty::{san::SanPlus, Chess, Color, Move, Position, Role, Square};
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
            score += (role_index(promotion_role) as i32) << 26;
        }

        if mv.is_capture() {
            score += 1 << 25;
        }

        let defending_pawns =
            pawn_attacks(them, to_square) & board.board().pieces(Role::Pawn, them).to_bits();

        let defending_pawns_score = if defending_pawns == 0 {
            6
        } else {
            5 - role_index(piece_role) as i32
        } << 22;
        score += defending_pawns_score;

        let move_value =
            piece_value(board, piece_role, to_square) - piece_value(board, piece_role, from_square);
        score += (512 + move_value) << 12;

        score += (to_square.to_index() as i32) << 6;
        score += from_square.to_index() as i32;

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

// Helper functions
fn role_index(role: Role) -> usize {
    match role {
        Role::Pawn => 0,
        Role::Knight => 1,
        Role::Bishop => 2,
        Role::Rook => 3,
        Role::Queen => 4,
        Role::King => 5,
    }
}

fn pawn_attacks(color: Color, square: Square) -> u64 {
    let bitboard = 1u64 << square.to_index();
    match color {
        Color::White => {
            ((bitboard & !0x8080808080808080) << 9) | ((bitboard & !0x0101010101010101) << 7)
        }
        Color::Black => {
            ((bitboard & !0x0101010101010101) >> 9) | ((bitboard & !0x8080808080808080) >> 7)
        }
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
        square.to_index()
    } else {
        mirror_square(square.to_index())
    };
    PSQT[role_index(role)][index]
}

fn mirror_square(sq_index: usize) -> usize {
    sq_index ^ 56
}

// Encoder struct
pub struct Encoder {
    codebook: &'static Book<u32>,
    tree: &'static Tree<u32>,
}

impl Encoder {
    pub fn new() -> Self {
        let (codebook, tree) = get_huffman_code();
        Encoder { codebook, tree }
    }

    pub fn encode(&self, pgn_moves: &[&str]) -> Option<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut board = Chess::default();

        for pgn_move in pgn_moves {
            let san_plus: SanPlus = pgn_move.parse().ok()?;
            let mv = san_plus.to_move(&board).ok()?;

            let legal_moves = board.legal_moves();
            let mut scored_moves: Vec<ScoredMove> = legal_moves
                .map(|legal_mv| ScoredMove::new(&board, legal_mv))
                .collect();
            scored_moves.sort();

            let index = scored_moves.iter().position(|sm| sm.mv == mv)? as u32;

            self.codebook.encode(&mut buffer, &index).unwrap();

            board = board.play(&mv).ok()?;
        }

        Some(buffer)
    }

    pub fn decode(&self, data: &[u8], plies: usize) -> Option<Vec<String>> {
        let mut output = Vec::new();
        let mut board = Chess::default();

        let mut decoder = self.tree.decoder(data);

        for _ in 0..plies {
            let legal_moves = board.legal_moves();
            let mut scored_moves: Vec<ScoredMove> =
                legal_moves.map(|mv| ScoredMove::new(&board, mv)).collect();
            scored_moves.sort();

            let index = decoder.next().unwrap().ok()? as usize;

            if index >= scored_moves.len() {
                return None;
            }

            let sm = &scored_moves[index];

            let san = sm.mv.to_san(&board);

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
            "e4", "e5", "Nf3", "Nc6", "Bb5", "a6", "Ba4", "Nf6", "O-O", "Be7", "Re1", "b5", "Bb3",
            "d6", "c3", "O-O", "h3", "Nb8", "d4", "cxd4", "cxd4", "Nbd7", "Nc3", "Bb7", "a3", "c5",
            "d5", "c4", "Bc2", "Nc5",
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
