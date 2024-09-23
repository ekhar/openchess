// src/lib/wasm/parser.ts

import {
  pgnParser,
  pgnCompressor,
  fenCompressor,
  initializeWasm,
} from "./index";
import {
  MasterGameSchema,
  PlayerGameSchema,
  type MasterGame,
  type PlayerGame,
} from "@/types/schemas";
import { supabase } from "@/utils/supabaseClient";
import { z } from "zod";

// Ensure Buffer is available (for Node.js). In browsers, use alternative methods if necessary.
import { Buffer } from "buffer";

/**
 * Imports master games from Supabase, decompresses PGN, and returns them.
 * @returns Array of MasterGame objects.
 */
export const importMasterGames = async (): Promise<MasterGame[]> => {
  try {
    // Initialize WASM if not already done
    await initializeWasm();

    if (!pgnParser || !fenCompressor || !pgnCompressor) {
      throw new Error("WASM modules failed to initialize.");
    }

    // Fetch master games from Supabase
    const { data: masterGamesData, error: masterGamesError } = await supabase
      .from("master_games")
      .select("*");

    if (masterGamesError) {
      throw new Error(
        `Error fetching master games: ${masterGamesError.message}`,
      );
    }

    // Validate and parse master games data
    const masterGames = MasterGameSchema.array().parse(masterGamesData);

    // Decompress PGN for each game
    const decompressedGames: MasterGame[] = masterGames.map((game) => {
      // Decompress PGN from base64
      const compressedPgnBytes = Buffer.from(game.compressed_pgn, "base64");
      const decompressedPgn = pgnCompressor.decompress(compressedPgnBytes);

      return {
        ...game,
        // Add decompressed_pgn if needed
        // decompressed_pgn: decompressedPgn,
      } as MasterGame;
    });

    // Return decompressed games or handle as needed
    return decompressedGames;
  } catch (error) {
    console.error("Error importing master games:", error);
    return [];
  }
};

/**
 * Exports and stores player games by compressing PGN and FEN,
 * inserting into Supabase, and storing uncompressed PGN in local storage.
 * @param playerGames Array of PlayerGame objects to export and store.
 */
export const exportPlayerGames = async (
  playerGames: PlayerGame[],
): Promise<void> => {
  try {
    // Initialize WASM if not already done
    await initializeWasm();

    if (!pgnParser || !fenCompressor || !pgnCompressor) {
      throw new Error("WASM modules failed to initialize.");
    }

    // Validate player games data
    const validatedGames = PlayerGameSchema.array().parse(playerGames);

    // Iterate through each player game
    for (const game of validatedGames) {
      // Compress PGN
      const compressedPgnBytes = pgnCompressor.compress(game.compressed_pgn);
      if (!compressedPgnBytes) {
        console.error("Failed to compress PGN data.");
        continue;
      }

      // Convert Uint8Array to base64 string for storage
      const compressedPgn = Buffer.from(compressedPgnBytes).toString("base64");

      // Compress FENs and collect positions
      const positionIds: number[] = [];
      for (let i = 0; i < game.moves.length; i++) {
        const move = game.moves[i];
        const fen = move.fen; // Ensure your `SanPlus` includes `fen`
        if (fen) {
          // Compress FEN
          const compressedFenBytes = fenCompressor.compress(fen);
          if (!compressedFenBytes) {
            console.error(`Failed to compress FEN: ${fen}`);
            continue;
          }

          // Convert Uint8Array to base64 string for storage
          const compressedFen =
            Buffer.from(compressedFenBytes).toString("base64");

          // Insert into positions table (using upsert to avoid duplicates)
          const { data: positionData, error: positionError } = await supabase
            .from("positions")
            .upsert([{ compressed_fen: compressedFen }], {
              onConflict: "compressed_fen",
            })
            .select("id")
            .single();

          if (positionError) {
            console.error("Error inserting position:", positionError.message);
            continue;
          }

          positionIds.push(positionData.id);
        }
      }

      // Insert into player_games
      const playerGameInsert: Omit<PlayerGame, "id"> = {
        eco: game.eco,
        white_player: game.white_player,
        black_player: game.black_player,
        date: game.date ? new Date(game.date) : null,
        result: game.result,
        compressed_pgn: compressedPgn,
        site: game.site,
        white_elo: game.white_elo,
        black_elo: game.black_elo,
        time_control: game.time_control,
      };

      const { data: gameData, error: gameError } = await supabase
        .from("player_games")
        .insert([playerGameInsert])
        .select("*")
        .single();

      if (gameError) {
        console.error("Error inserting player game:", gameError.message);
        continue;
      }

      // Insert into player_game_positions
      const gamePositions = positionIds.map((positionId, index) => ({
        game_id: gameData.id,
        position_id: positionId,
        move_number: index + 1,
      }));

      const { error: posError } = await supabase
        .from("player_game_positions")
        .insert(gamePositions);

      if (posError) {
        console.error("Error inserting game positions:", posError.message);
        continue;
      }

      // Store uncompressed PGN in local storage
      localStorage.setItem(
        `player_game_${gameData.id}_pgn`,
        game.compressed_pgn,
      );
    }
  } catch (error) {
    console.error("Error exporting and storing player games:", error);
  }
};
