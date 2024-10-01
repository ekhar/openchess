// src/lib/wasm/compress.ts

import { fenCompressor } from "./index";

export const compressFEN = async (fen: string): Promise<Uint8Array | null> => {
  try {
    const compressed = fenCompressor.compress(fen);
    return new Uint8Array(compressed);
  } catch (error) {
    console.error("Error compressing FEN:", error);
    return null;
  }
};

// src/lib/wasm/compress.ts

import { pgnCompressor } from "./index";

export const compressPGN = async (pgn: string): Promise<Uint8Array | null> => {
  try {
    const compressed = pgnCompressor.compress(pgn);
    return new Uint8Array(compressed);
  } catch (error) {
    console.error("Error compressing PGN:", error);
    return null;
  }
};
