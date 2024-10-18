// chess_compress.ts

import init, {
  wasm_compress_position,
  wasm_decompress_position,
  wasm_compress_pgn,
  wasm_decompress_pgn,
} from "@openchess/chess-compression-wasm";

let isWasmInitialized = false;

/**
 * Initializes the WASM module.
 */
export async function initializeWasm() {
  if (!isWasmInitialized) {
    try {
      await init();
      isWasmInitialized = true;
    } catch (error) {
      console.error("Failed to initialize WASM module:", error);
      throw error;
    }
  }
}

export async function compressPosition(
  fen: string,
): Promise<Uint8Array | null> {
  try {
    await initializeWasm();
    const compressed = wasm_compress_position(fen);
    return compressed;
  } catch (error) {
    console.error("Compression failed:", error);
    return null;
  }
}

export async function decompressPosition(
  compressed: Uint8Array,
): Promise<string | null> {
  try {
    await initializeWasm();
    const decompressed = wasm_decompress_position(compressed);
    return decompressed;
  } catch (error) {
    console.error("Decompression failed:", error);
    return null;
  }
}

export async function compressPgn(moves: string): Promise<Uint8Array | null> {
  try {
    await initializeWasm();
    const compressed = wasm_compress_pgn(moves);
    return compressed;
  } catch (error) {
    console.error("PGN Compression failed:", error);
    return null;
  }
}

export async function decompressPgn(
  compressed: Uint8Array,
  plies: number,
): Promise<string | null> {
  try {
    await initializeWasm();
    const decompressed = wasm_decompress_pgn(compressed, plies);
    return decompressed;
  } catch (error) {
    console.error("PGN Decompression failed:", error);
    return null;
  }
}
