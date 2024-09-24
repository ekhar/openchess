"use client";
// src/lib/wasm/controller.ts

import { getChessGames } from "./fetchChessGames";
import { exportPlayerGames } from "./parser";
import { parsePgnToPlayerGames } from "./pgnUtils"; // We'll implement this utility
import { type PlayerGame } from "@/types/schemas";

/**
 * Controller function to fetch, parse, and store chess games.
 * @param username Chess.com username
 * @param year Year of the games
 * @param month Month of the games
 */
export const fetchAndStoreChessGames = async (
  username: string,
  year: number,
  month: number,
): Promise<void> => {
  try {
    // Fetch PGN data from Chess.com
    const pgnData = await getChessGames(username, year, month);

    // Parse PGN data into PlayerGame[] format
    const playerGames: PlayerGame[] = parsePgnToPlayerGames(pgnData);

    // Export and store player games
    await exportPlayerGames(playerGames);

    console.log("Successfully imported chess games.");
  } catch (error) {
    console.error("Error in fetchAndStoreChessGames:", error);
  }
};
