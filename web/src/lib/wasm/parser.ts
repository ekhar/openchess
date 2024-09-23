// src/lib/wasm/parser.ts

import { pgnParser, initializeWasm } from "./index";
import { type Game, Position } from "@/types/interfaces";
import { supabase } from "@/utils/supabaseClient";

export const parseAndStoreGames = async (pgnData: string): Promise<void> => {
  try {
    // Initialize WASM if not already done
    await initializeWasm();

    // Parse PGN data to JSON string
    const parsedJson = pgnParser.parse_pgn(pgnData);

    // Deserialize JSON to Game[]
    const games: Game[] = JSON.parse(parsedJson);

    // Process each game
    for (const game of games) {
      // Compress PGN
      const compressedPgn = await pgnCompressor.compress(pgnData); // Assuming you have the original PGN for compression

      // Insert or retrieve positions
      const positionIds: number[] = [];
      for (let i = 0; i < game.moves.length; i++) {
        const move = game.moves[i];
        const fen = move.fen; // Adjust based on your `SanPlus` structure
        if (fen) {
          // Compress FEN
          const compressedFen = await fenCompressor.compress(fen);

          // Insert into positions table
          const { data: positionData, error: positionError } = await supabase
            .from("positions")
            .insert([{ compressed_fen: compressedFen }])
            .select("id")
            .single();

          if (positionError) {
            console.error("Error inserting position:", positionError.message);
            continue;
          }

          positionIds.push(positionData.id);
        }
      }

      // Insert into master_games
      const masterGame: Omit<MasterGame, "id"> = {
        eco: game.eco || "",
        white_player: game.white_player,
        black_player: game.black_player,
        date: game.date ? new Date(game.date) : null,
        result: game.result,
        compressed_pgn: compressedPgn,
        white_elo: game.white.rating || 0,
        black_elo: game.black.rating || 0,
        time_control: game.speed || null,
      };

      const { data: gameData, error: gameError } = await supabase
        .from("master_games")
        .insert([masterGame])
        .select("*")
        .single();

      if (gameError) {
        console.error("Error inserting master game:", gameError.message);
        continue;
      }

      // Insert into master_game_positions
      const gamePositions = positionIds.map((positionId, index) => ({
        game_id: gameData.id,
        position_id: positionId,
        move_number: index + 1,
      }));

      const { error: posError } = await supabase
        .from("master_game_positions")
        .insert(gamePositions);

      if (posError) {
        console.error("Error inserting game positions:", posError.message);
        continue;
      }
    }
  } catch (error) {
    console.error("Error parsing and storing games:", error);
  }
};
