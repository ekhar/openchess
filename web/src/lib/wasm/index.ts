// src/lib/wasm/index.ts

import init, { PgnParser, FenCompressor, PgnCompressor } from "./pkg/wasm_test"; // Adjust the path as necessary

let pgnParser: PgnParser | null = null;
let fenCompressor: FenCompressor | null = null;
let pgnCompressor: PgnCompressor | null = null;
let wasmInitialized = false;

export const initializeWasm = async () => {
  if (!wasmInitialized) {
    await init(); // Initialize the WASM module
    pgnParser = new PgnParser();
    fenCompressor = new FenCompressor();
    pgnCompressor = new PgnCompressor();
    wasmInitialized = true;
  }
};

export { pgnParser, fenCompressor, pgnCompressor };
