// chess_compression_client.ts
import wbgInit, {
  InitOutput,
  wasm_compress_position,
  wasm_decompress_position,
  wasm_compress_pgn,
  wasm_decompress_pgn,
} from "@openchess/chess-compression-wasm/pkg/web/chess_compression.js";

let wasm_mod: InitOutput | null = null;
let initPromise: Promise<void> | null = null;

async function ensureInitialized(): Promise<void> {
  if (wasm_mod) return;

  if (!initPromise) {
    initPromise = wbgInit()
      .then((mod) => {
        wasm_mod = mod;
      })
      .catch((error) => {
        console.error("Failed to initialize WASM module:", error);
        initPromise = null;
        throw error;
      });
  }

  return initPromise;
}

export async function compressPosition(
  fen: string,
): Promise<Uint8Array | null> {
  try {
    await ensureInitialized();
    return wasm_compress_position(fen);
  } catch (error) {
    console.error("Compression failed:", error);
    return null;
  }
}

export async function decompressPosition(
  compressed: Uint8Array,
): Promise<string | null> {
  try {
    await ensureInitialized();
    return wasm_decompress_position(compressed);
  } catch (error) {
    console.error("Decompression failed:", error);
    return null;
  }
}

export async function compressPgn(moves: string): Promise<Uint8Array | null> {
  try {
    await ensureInitialized();
    return wasm_compress_pgn(moves);
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
    await ensureInitialized();
    return wasm_decompress_pgn(compressed, plies);
  } catch (error) {
    console.error("PGN Decompression failed:", error);
    return null;
  }
}

// Optional: export initWasm for preloading if desired
export const initWasm = ensureInitialized;
