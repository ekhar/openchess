"use server";

import { createClient } from "@supabase/supabase-js";
import { unstable_cache } from "next/cache";
import { Chess } from "chess.js";
import {
  compressPosition,
  decompressPgn,
} from "@/lib/server_chess_compression";
import { type Database } from "@/types/database.types";
import { type MastersApiResponse } from "@/types/MasterApi";
import { supabase } from "@/utils/supabase/supabaseClient";

// Initialize the Supabase client
const supabaseUrl = process.env.NEXT_PUBLIC_SUPABASE_URL!;
const supabaseKey = process.env.SUPABASE_SERVICE_ROLE_KEY!; // Use service role key for admin privileges
// const supabase = createClient<Database>(supabaseUrl, supabaseKey);

export async function master_moves(fen: string): Promise<string | undefined> {
  console.log("Compressing FEN:", fen);
  console.log("Compressed FEN:", compressPosition(fen));
  return String(await compressPosition(fen));
}

export async function master_movestest(
  fen: string,
): Promise<MastersApiResponse | undefined> {
  return unstable_cache(
    async () => {
      // Initialize the WASM module
      const compressed = await compressPosition(fen);
      if (!compressed) {
        console.error("Failed to compress FEN");
        return undefined;
      }
      const compressedBase64 = Buffer.from(compressed).toString("base64");

      // Query positions where the compressed position matches
      const { data: positions, error: positionsError } = await supabase
        .from("positions_foreign")
        .select("*")
        .eq("position", compressedBase64);

      if (positionsError) {
        console.error("Error querying positions:", positionsError);
        return undefined;
      }

      if (!positions || positions.length === 0) {
        console.log("No positions found for the given FEN");
        return undefined;
      }

      // Map of game_id to move_number
      const positionMap = new Map<number, number>();
      positions.forEach((pos) => {
        if (pos.game_id !== null && pos.move_number !== null) {
          positionMap.set(pos.game_id, pos.move_number);
        }
      });

      const gameIds = Array.from(positionMap.keys());

      // Query games for the collected game IDs
      const { data: games, error: gamesError } = await supabase
        .from("games_foreign")
        .select("*")
        .in("id", gameIds);

      if (gamesError) {
        console.error("Error querying games:", gamesError);
        return undefined;
      }

      if (!games || games.length === 0) {
        console.log("No games found for the given positions");
        return undefined;
      }

      // Initialize counters
      let whiteWins = 0;
      let blackWins = 0;
      let draws = 0;

      interface MoveStats {
        uci: string;
        san: string;
        white: number;
        draws: number;
        black: number;
      }

      const moveStatsMap = new Map<string, MoveStats>();

      // Process each game to collect results and next moves
      for (const game of games) {
        const gameId = game.id;
        const moveNumber = positionMap.get(gameId);
        if (moveNumber === undefined) {
          continue;
        }

        const result = game.result;

        // Count the overall results
        if (result === "white") {
          whiteWins += 1;
        } else if (result === "black") {
          blackWins += 1;
        } else if (result === "draw") {
          draws += 1;
        }

        if (!game.pgn_moves) {
          continue;
        }

        // Decompress the PGN moves
        const compressedPgnBuffer = Buffer.from(game.pgn_moves, "base64");
        const decompressedPgn = await decompressPgn(compressedPgnBuffer, 500);
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
          const sanMove = moves[i];
          const moveResult = chess.move("");
          if (moveResult === null) {
            console.error(
              `Invalid move ${sanMove} at move ${i} in game ${gameId}`,
            );
            break;
          }
        }

        // Get the next move in SAN and UCI notation
        const nextMoveSan = moves[moveNumber];
        if (!nextMoveSan) {
          continue;
        }

        const nextMoveResult = chess.move(nextMoveSan);
        if (!nextMoveResult) {
          console.error(
            `Invalid next move ${nextMoveSan} at move ${moveNumber} in game ${gameId}`,
          );
          continue;
        }
        const nextMoveUci =
          nextMoveResult.from +
          nextMoveResult.to +
          (nextMoveResult.promotion || "");

        const san = nextMoveSan;
        const uci = nextMoveUci;

        let moveStat = moveStatsMap.get(san);
        if (!moveStat) {
          moveStat = { uci, san, white: 0, draws: 0, black: 0 };
          moveStatsMap.set(san, moveStat);
        }

        // Increment move statistics based on the result
        if (result === "white") {
          moveStat.white += 1;
        } else if (result === "black") {
          moveStat.black += 1;
        } else if (result === "draw") {
          moveStat.draws += 1;
        }
      }

      // Build the response object
      const response: MastersApiResponse = {
        white: whiteWins,
        black: blackWins,
        draws: draws,
        moves: Array.from(moveStatsMap.values()),
      };

      return response;
    },
    [`master_moves_${fen}`],
    {
      revalidate: 1, // Revalidate every 24 hours
      tags: [`master_moves_${fen}`],
    },
  )();
}
