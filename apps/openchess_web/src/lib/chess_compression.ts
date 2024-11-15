const wasm_mod = await import("@openchess/chess-compression-wasm");

export function compressPosition(fen: string): Uint8Array | null {
  try {
    console.log("Compressing position:", fen);
    const compressed = wasm_mod.wasm_compress_position(fen);
    return compressed;
  } catch (error) {
    console.error("Compression failed:", error);
    return null;
  }
}

export function decompressPosition(compressed: Uint8Array): string | null {
  try {
    const decompressed = wasm_mod.wasm_decompress_position(compressed);
    return decompressed;
  } catch (error) {
    console.error("Decompression failed:", error);
    return null;
  }
}

export function compressPgn(moves: string): Uint8Array | null {
  try {
    const compressed = wasm_mod.wasm_compress_pgn(moves);
    return compressed;
  } catch (error) {
    console.error("PGN Compression failed:", error);
    return null;
  }
}

export function decompressPgn(
  compressed: Uint8Array,
  plies: number,
): string | null {
  try {
    const decompressed = wasm_mod.wasm_decompress_pgn(compressed, plies);
    return decompressed;
  } catch (error) {
    console.error("PGN Decompression failed:", error);
    return null;
  }
}
