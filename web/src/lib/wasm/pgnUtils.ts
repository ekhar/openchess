// src/lib/wasm/pgnUtils.ts

import { pgnParser, initializeWasm } from "./index";
import { type PlayerGame } from "@/types/schemas";

/**
 * Parses raw PGN data into an array of PlayerGame objects.
 * @param pgnData Raw PGN data as string
 * @returns Array of PlayerGame objects
 */
export const parsePgnToPlayerGames = async (
  pgnData: string,
): Promise<PlayerGame[]> => {
  try {
    // Initialize WASM if not already done
    await initializeWasm();

    if (!pgnParser) {
      throw new Error("PgnParser is not initialized.");
    }

    // Parse PGN data to JSON string
    const parsedJson = pgnParser.parse_pgn(pgnData);
    if (!parsedJson) {
      throw new Error("Failed to parse PGN data.");
    }
    console.log("Parsed PGN data:", parsedJson);

    // Deserialize JSON to PlayerGame[]
    const playerGames: PlayerGame[] = JSON.parse(parsedJson) as PlayerGame[];

    return playerGames;
  } catch (error) {
    console.error("Error parsing PGN to PlayerGames:", error);
    return [];
  }
};
