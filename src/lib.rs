pub mod fen_compress;
pub mod huffman_code;
pub mod import_pgn;
pub mod pgn_compress;
// src/lib.rs

pub struct CompressedPosition {
    // Define your compressed state here
    // Example: piece_placement: u64,
}

// pub fn compress_fen(fen: &str) -> Result<CompressedPosition, Box<dyn std::error::Error>> {
//     todo!("Implement compression logic")
// }
//
// pub fn decompress_position(compressed: u64) -> Result<u64, Box<dyn std::error::Error>> {
//     // Implement decompression logic here
//     todo!("Implement decompression logic")
// }
