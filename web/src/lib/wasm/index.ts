// src/lib/wasm/index.ts

import init, { PgnParser, FenCompressor, PgnCompressor } from "./pkg/wasm_test"; // Adjust the path as necessary

let pgnParser: PgnParser;
let fenCompressor: FenCompressor;
let pgnCompressor: PgnCompressor;

export const initializeWasm = async () => {
  await init(); // Initialize the WASM module
  pgnParser = new PgnParser();
  fenCompressor = new FenCompressor();
  pgnCompressor = new PgnCompressor();
};

export { pgnParser, fenCompressor, pgnCompressor };
