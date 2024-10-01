// src/types/interfaces.ts

import { type GameResult, type Site, type Speed } from "./enums";

// -----------------------
// Positions Table
// -----------------------

export interface Position {
  id: number;
  compressed_fen: Uint8Array; // BYTEA in PostgreSQL maps to Uint8Array in TypeScript
}

// -----------------------
// Master Games Table
// -----------------------

export interface MasterGame {
  id: number;
  eco: string; // VARCHAR(4)
  white_player: string;
  black_player: string;
  date: string | null; // DATE
  result: GameResult;
  compressed_pgn: Uint8Array;
  white_elo: number;
  black_elo: number;
  time_control: Speed | null;

  // Unique Constraint: (compressed_pgn, white_player, black_player, date)
}

// -----------------------
// Master Game Positions Table
// -----------------------

export interface MasterGamePosition {
  game_id: number; // References master_games(id)
  position_id: number; // References positions(id)
  move_number: number;
}

// -----------------------
// Player Games Table
// -----------------------

export interface PlayerGame {
  id: number;
  eco: string; // VARCHAR(4)
  white_player: string;
  black_player: string;
  date: string | null; // DATE
  result: GameResult;
  compressed_pgn: Uint8Array;
  site: Site | null;
  white_elo: number;
  black_elo: number;
  time_control: Speed | null;

  // Unique Constraint: (compressed_pgn, white_player, black_player, date)
}

// -----------------------
// Player Game Positions Table
// -----------------------

export interface PlayerGamePosition {
  game_id: number; // References player_games(id)
  position_id: number; // References positions(id)
  move_number: number;
}
