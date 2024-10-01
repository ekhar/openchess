use wasm_bindgen::prelude::*;

use crate::{compress_pgn, compress_position, decompress_pgn, decompress_position};
use shakmaty::Chess;

#[wasm_bindgen]
pub fn wasm_compress_position(fen: &str) -> Result<Vec<u8>, JsValue> {
    let position = Chess::from_fen(fen).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(compress_position(&position).to_vec())
}

#[wasm_bindgen]
pub fn wasm_decompress_position(compressed: &[u8]) -> Result<String, JsValue> {
    let position = decompress_position(compressed.try_into().unwrap())
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(position.to_string())
}

#[wasm_bindgen]
pub fn wasm_compress_pgn(moves: &str) -> Result<Vec<u8>, JsValue> {
    let moves: Vec<String> = moves.split_whitespace().map(String::from).collect();
    compress_pgn(&moves).map_err(|e| JsValue::from_str(&e.to_string()))
}

#[wasm_bindgen]
pub fn wasm_decompress_pgn(compressed: &[u8], plies: usize) -> Result<String, JsValue> {
    let moves = decompress_pgn(compressed, plies).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(moves.join(" "))
}
