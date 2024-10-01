use std::str::FromStr;

use crate::{compress_pgn, compress_position, decompress_pgn, decompress_position};
use js_sys::Uint8Array; // Use this type for better TS compatibility
use wasm_bindgen::prelude::*;

use shakmaty::fen::Fen;
use shakmaty::{CastlingMode, EnPassantMode};

#[wasm_bindgen]
pub fn wasm_compress_position(fen: &str) -> Result<Uint8Array, JsValue> {
    // Parse the FEN string
    let fen = Fen::from_str(fen).map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Convert FEN to Position
    let position = fen
        .into_position(CastlingMode::Standard)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Compress the position
    let compressed = compress_position(&position);

    // Convert Rust Vec<u8> to JS-compatible Uint8Array
    Ok(Uint8Array::from(compressed.as_slice()))
}

#[wasm_bindgen]
pub fn wasm_decompress_position(compressed: &[u8]) -> Result<String, JsValue> {
    let position = decompress_position(compressed.try_into().unwrap())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(Fen::from_position(position, EnPassantMode::Legal).to_string())
}

#[wasm_bindgen]
pub fn wasm_compress_pgn(moves: &str) -> Result<Uint8Array, JsValue> {
    let moves: Vec<String> = moves.split_whitespace().map(String::from).collect();
    let compressed = compress_pgn(&moves).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(Uint8Array::from(compressed.as_slice()))
}

#[wasm_bindgen]
pub fn wasm_decompress_pgn(compressed: &[u8], plies: usize) -> Result<String, JsValue> {
    let moves = decompress_pgn(compressed, plies).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(moves.join(" "))
}
