// src/main.rs
use fen_compression::{compress_fen, decompress_position};

fn main() {
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    match compress_fen(fen) {
        Ok(compressed) => {
            println!("FEN compressed successfully");
            // Further processing of compressed position
        }
        Err(e) => eprintln!("Error compressing FEN: {}", e),
    }
}
