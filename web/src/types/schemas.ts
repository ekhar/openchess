// src/types/schemas.ts

import { z } from "zod";

// Define the Player schema
export const PlayerSchema = z.object({
  name: z.string().optional(),
  rating: z.number().int().optional(),
});

// Define the GameResult schema
export const GameResultSchema = z.enum(["white", "black", "draw"]);

// Define the SanPlus schema
export const SanPlusSchema = z.object({
  san: z.string(),
  fen: z.string().optional(),
});

// Define the PlayerGame schema
export const PlayerGameSchema = z.object({
  eco: z.string().max(4),
  white_player: z.string(),
  black_player: z.string(),
  date: z.string().optional(),
  result: GameResultSchema,
  compressed_pgn: z.string(), // Assuming compressed_pgn is a base64 string
  site: z.enum(["chesscom", "lichess", "custom"]).optional(),
  white_elo: z.number().int(),
  black_elo: z.number().int(),
  time_control: z
    .enum([
      "ultrabullet",
      "bullet",
      "blitz",
      "rapid",
      "classical",
      "correspondence",
    ])
    .optional(),
});

// Define the PlayerGamePosition schema
export const PlayerGamePositionSchema = z.object({
  game_id: z.number(),
  position_id: z.number(),
  move_number: z.number(),
});

// Define the Position schema
export const PositionSchema = z.object({
  id: z.number(),
  compressed_fen: z.string(), // Assuming compressed_fen is a base64 string
});

// Define the MasterGame schema
export const MasterGameSchema = z.object({
  id: z.number(),
  eco: z.string().max(4),
  white_player: z.string(),
  black_player: z.string(),
  date: z.string().optional(),
  result: GameResultSchema,
  compressed_pgn: z.string(), // Assuming compressed_pgn is a base64 string
  white_elo: z.number().int(),
  black_elo: z.number().int(),
  time_control: z
    .enum([
      "ultrabullet",
      "bullet",
      "blitz",
      "rapid",
      "classical",
      "correspondence",
    ])
    .optional(),
});

// Define the MasterGamePosition schema
export const MasterGamePositionSchema = z.object({
  game_id: z.number(),
  position_id: z.number(),
  move_number: z.number(),
});

// Export TypeScript types inferred from schemas
export type PlayerGame = z.infer<typeof PlayerGameSchema>;
export type PlayerGamePosition = z.infer<typeof PlayerGamePositionSchema>;
export type MasterGame = z.infer<typeof MasterGameSchema>;
export type MasterGamePosition = z.infer<typeof MasterGamePositionSchema>;
export type Position = z.infer<typeof PositionSchema>;
