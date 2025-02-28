// filename: apps/openchess_web/src/services/MasterMoves.tsx
"use client";

import { supabase } from "@/utils/supabase/supabaseClient";
import { compressPosition, decompressPgn } from "@/lib/client_chess_compression";
import { Chess } from "chess.js";

export interface MastersApiResponse {
  white: number;
  black: number;
  draws: number;
  moves: MoveStats[];
  total?: number;
}

export interface MoveStats {
  uci: string;
  san: string;
  white: number;
  black: number;
  draws: number;
  averageRating?: number;
}

export async function master_moves(fen: string): Promise<MastersApiResponse | undefined> {
  try {
    console.log("Analyzing position:", fen);

    // Step 1: Compress the FEN position
    const compressed = await compressPosition(fen);
    if (!compressed) {
      console.error("Failed to compress FEN");
      return undefined;
    }

    // Convert compressed Uint8Array to base64 for database query
    const binaryString = Array.from(compressed).map(byte => String.fromCharCode(byte)).join('');
    const base64Encoded = btoa(binaryString);

    console.log("Position compressed successfully");

    // Step 2: Query positions where the compressed position matches
    const { data: positions, error: positionsError } = await supabase
      .from("positions_foreign")
      .select("game_id, move_number")
      .eq("position", base64Encoded);

    if (positionsError) {
      console.error("Error querying positions:", positionsError);

      // Handle the specific FDW user mapping error
      if (positionsError.code === '42704' && positionsError.message?.includes('user mapping not found')) {
        console.error("FDW user mapping error detected. The anonymous user doesn't have permission to access foreign tables.");
        return {
          white: 0,
          black: 0,
          draws: 0,
          moves: [],
          total: 0,
          error: "Database connection error: The user doesn't have proper access to the master games database."
        };
      }

      return undefined;
    }

    if (!positions || positions.length === 0) {
      console.log("No positions found for the given FEN");
      return {
        white: 0,
        black: 0,
        draws: 0,
        moves: []
      };
    }

    console.log(`Found ${positions.length} matching positions`);

    // Extract unique game IDs
    const gameIds = [...new Set(positions.map(pos => pos.game_id))];

    // Map of game_id to move_number for the next move lookup
    const positionMap = new Map<number, number>();
    positions.forEach(pos => {
      if (pos.game_id !== null && pos.move_number !== null) {
        positionMap.set(pos.game_id, pos.move_number);
      }
    });

    // Step 3: Query games for the collected game IDs
    const { data: games, error: gamesError } = await supabase
      .from("games_foreign")
      .select("id, result, white_elo, black_elo, pgn_moves")
      .in("id", gameIds);

    if (gamesError) {
      console.error("Error querying games:", gamesError);
      return undefined;
    }

    if (!games || games.length === 0) {
      console.log("No games found for the given positions");
      return {
        white: 0,
        black: 0,
        draws: 0,
        moves: []
      };
    }

    console.log(`Retrieved ${games.length} games`);

    // Initialize counters
    let whiteWins = 0;
    let blackWins = 0;
    let draws = 0;

    // Track total Elo per move for calculating average
    const moveStatsMap = new Map<string, {
      uci: string;
      san: string;
      white: number;
      black: number;
      draws: number;
      totalElo: number;
      gameCount: number;
    }>();

    // Process each game to collect results and next moves
    for (const game of games) {
      const gameId = game.id;
      const moveNumber = positionMap.get(gameId);

      if (moveNumber === undefined) continue;

      // Count the overall results
      if (game.result === "white") {
        whiteWins++;
      } else if (game.result === "black") {
        blackWins++;
      } else if (game.result === "draw") {
        draws++;
      }

      if (!game.pgn_moves) continue;

      // Calculate average Elo for the game
      const gameElo = Math.round((game.white_elo + game.black_elo) / 2);

      try {
        // Convert base64 string to Uint8Array
        const binaryPgn = atob(game.pgn_moves);
        const uint8Array = new Uint8Array(binaryPgn.length);
        for (let i = 0; i < binaryPgn.length; i++) {
          uint8Array[i] = binaryPgn.charCodeAt(i);
        }

        // Decompress the PGN moves
        const decompressedPgn = await decompressPgn(uint8Array, 500);
        if (!decompressedPgn) {
          console.error("Failed to decompress PGN for game", gameId);
          continue;
        }

        const moves = decompressedPgn.trim().split(" ");
        if (moveNumber >= moves.length) {
          // No next move available
          continue;
        }

        // Initialize a chess instance and play moves up to the current position
        const chess = new Chess();
        for (let i = 0; i < moveNumber; i++) {
          try {
            chess.move(moves[i]);
          } catch (e) {
            console.error(`Invalid move ${moves[i]} at position ${i} in game ${gameId}`);
            break;
          }
        }

        // Get the next move
        const nextMoveSan = moves[moveNumber];
        if (!nextMoveSan) continue;

        try {
          const nextMoveResult = chess.move(nextMoveSan);
          if (!nextMoveResult) {
            console.error(`Invalid next move ${nextMoveSan} at move ${moveNumber} in game ${gameId}`);
            continue;
          }

          // Create UCI notation
          const nextMoveUci = `${nextMoveResult.from}${nextMoveResult.to}${nextMoveResult.promotion || ''}`;

          // Update or create move statistics
          let moveStat = moveStatsMap.get(nextMoveSan);
          if (!moveStat) {
            moveStat = {
              uci: nextMoveUci,
              san: nextMoveSan,
              white: 0,
              draws: 0,
              black: 0,
              totalElo: 0,
              gameCount: 0
            };
            moveStatsMap.set(nextMoveSan, moveStat);
          }

          // Increment move statistics based on the result
          if (game.result === "white") {
            moveStat.white++;
          } else if (game.result === "black") {
            moveStat.black++;
          } else if (game.result === "draw") {
            moveStat.draws++;
          }

          // Add to Elo tracking
          moveStat.totalElo += gameElo;
          moveStat.gameCount++;
        } catch (e) {
          console.error(`Error processing move ${nextMoveSan} for game ${gameId}:`, e);
        }
      } catch (e) {
        console.error(`Error processing game ${gameId}:`, e);
      }
    }

    // Build the final array of move stats with calculated average Elo
    const movesArray = Array.from(moveStatsMap.values()).map(({ uci, san, white, black, draws, totalElo, gameCount }) => ({
      uci,
      san,
      white,
      black,
      draws,
      averageRating: gameCount > 0 ? Math.round(totalElo / gameCount) : 0
    }));

    // Sort moves by popularity (total games)
    movesArray.sort((a, b) => {
      const totalA = a.white + a.black + a.draws;
      const totalB = b.white + b.black + b.draws;
      return totalB - totalA;
    });

    const response: MastersApiResponse = {
      white: whiteWins,
      black: blackWins,
      draws: draws,
      moves: movesArray,
      total: whiteWins + blackWins + draws
    };

    console.log(`Analysis complete: ${response.total} total games, ${response.moves.length} distinct moves`);
    return response;

  } catch (error) {
    console.error("Error in master_moves:", error);
    return undefined;
  }
}
