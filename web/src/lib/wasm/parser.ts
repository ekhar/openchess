// src/lib/wasm/parser.ts

import {
  pgnParser,
  pgnCompressor,
  fenCompressor,
  initializeWasm,
} from "./index";
import {
  PlayerGameSchema,
  type PlayerGame,
  type SanPlus,
} from "@/types/schemas";
import { supabase } from "@/utils/supabaseClient";
import { z } from "zod";

// Ensure Buffer is available (for Node.js). In browsers, use alternative methods if necessary.
import { Buffer } from "buffer";

// Function to parse PGN data
export const parsePgnData = async (pgnData: string): Promise<PlayerGame[]> => {
  if (!pgnParser) {
    throw new Error("WASM modules not initialized.");
  }

  const json = pgnParser.parse_pgn(pgnData);
  const parsedGames = JSON.parse(json) as unknown[];

  // Validate and transform parsed games to PlayerGame[]
  const playerGames: PlayerGame[] = parsedGames.map((game) => {
    // Transform Rust's Game to PlayerGame
    const transformedGame: PlayerGame = {
      id: game.id,
      eco: game.eco,
      white_player: game.white_player,
      black_player: game.black_player,
      white_elo: game.white_elo,
      black_elo: game.black_elo,
      date: game.date,
      result: game.result,
      compressed_pgn: "", // To be filled after compression
      site: game.chess_site,
      time_control: game.speed ? speedToTimeControl(game.speed) : undefined,
      fen: game.fen,
      variant: game.variant,
      moves: game.moves,
    };
    return transformedGame;
  });

  // Validate with Zod
  PlayerGameSchema.array().parse(playerGames);

  return playerGames;
};

// Helper function to map Speed enum to time_control string
const speedToTimeControl = (
  speed: string,
): PlayerGame["time_control"] | undefined => {
  switch (speed) {
    case "ultrabullet":
      return "ultrabullet";
    case "bullet":
      return "bullet";
    case "blitz":
      return "blitz";
    case "rapid":
      return "rapid";
    case "classical":
      return "classical";
    case "correspondence":
      return "correspondence";
    default:
      return undefined;
  }
};

// Function to export and store player games
export const exportPlayerGames = async (
  playerGames: PlayerGame[],
): Promise<void> => {
  try {
    // Initialize WASM if not already done
    await initializeWasm();

    if (!pgnParser || !fenCompressor || !pgnCompressor) {
      throw new Error("WASM modules failed to initialize.");
    }

    // Iterate through each player game
    for (const game of playerGames) {
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
      if (game.moves) {
        for (let i = 0; i < game.moves.length; i++) {
          const move = game.moves[i] as SanPlus;
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
      }

      // Insert into player_games
      const playerGameInsert: Omit<
        PlayerGame,
        "id" | "compressed_pgn" | "fen" | "variant" | "moves"
      > & {
        compressed_pgn: string;
      } = {
        eco: game.eco,
        white_player: game.white_player,
        black_player: game.black_player,
        date: game.date ? new Date(game.date) : undefined,
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
