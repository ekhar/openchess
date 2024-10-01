//! Chess compression library
//!
//! This library provides functionality for compressing and decompressing
//! chess positions and games.

pub mod fen_compress;
mod huffman_code;
pub mod pgn_compress;
mod psqt;
pub mod wasm;
pub use wasm::*;

pub use fen_compress::{CompressedPosition, CompressedPositionError};
pub use pgn_compress::{Encoder, EncoderError};

// Re-export types from shakmaty that are used in our public API
pub use shakmaty::{Chess, Position};

/// Compress a chess position
///
/// This function takes a `Chess` position and returns a `CompressedPosition`.
///
/// # Examples
///
/// ```
/// use chess_compression::{compress_position, Chess};
/// use shakmaty::Position;
///
/// let position = Chess::default();
/// let compressed = compress_position(&position);
/// ```
pub fn compress_position(position: &Chess) -> [u8; 32] {
    CompressedPosition::compress(position)
}

/// Decompress a chess position
///
/// This function takes a compressed position as a `[u8; 32]` array and returns a `Result<Chess, CompressedPositionError>`.
///
/// # Examples
///
/// ```
/// use chess_compression::{compress_position, decompress_position, Chess};
/// use shakmaty::Position;
///
/// let position = Chess::default();
/// let compressed = compress_position(&position);
/// let decompressed = decompress_position(&compressed).unwrap();
/// assert_eq!(position, decompressed);
/// ```
pub fn decompress_position(compressed: &[u8; 32]) -> Result<Chess, CompressedPositionError> {
    CompressedPosition::decompress(compressed)
}

/// Compress a sequence of chess moves (PGN)
///
/// This function takes a slice of PGN move strings and returns a `Result<Vec<u8>, EncoderError>`.
///
/// # Examples
///
/// ```
/// use chess_compression::compress_pgn;
///
/// let moves = vec!["e4".to_string(), "e5".to_string(), "Nf3".to_string(), "Nc6".to_string()];
/// let compressed = compress_pgn(&moves).unwrap();
/// ```
pub fn compress_pgn(moves: &[String]) -> Result<Vec<u8>, EncoderError> {
    let mut encoder = Encoder::new();
    for move_str in moves {
        encoder.encode_move(move_str)?;
    }
    Ok(encoder.finalize().to_bytes())
}

/// Decompress a sequence of chess moves (PGN)
///
/// This function takes a slice of compressed bytes and the number of plies,
/// and returns a `Result<Vec<String>, EncoderError>`.
///
/// # Examples
///
/// ```
/// use chess_compression::{compress_pgn, decompress_pgn};
///
/// let moves = vec!["e4".to_string(), "e5".to_string(), "Nf3".to_string(), "Nc6".to_string()];
/// let compressed = compress_pgn(&moves).unwrap();
/// let decompressed = decompress_pgn(&compressed, moves.len()).unwrap();
/// assert_eq!(moves, decompressed);
/// ```
pub fn decompress_pgn(compressed: &[u8], plies: usize) -> Result<Vec<String>, EncoderError> {
    let encoder = Encoder::new();
    encoder.decode(&bit_vec::BitVec::from_bytes(compressed), plies)
}
