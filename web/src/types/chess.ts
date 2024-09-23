// src/types/chess.ts

export type GameResult = "white" | "black" | "draw";
export type Site = "chesscom" | "lichess" | "custom";
export type Speed =
  | "ultrabullet"
  | "bullet"
  | "blitz"
  | "rapid"
  | "classical"
  | "correspondence";

export interface Position {
  id: number;
  compressed_fen: Uint8Array; // Compressed FEN data
}

export interface MasterGame {
  id: number;
  eco: string;
  white_player: string;
  black_player: string;
  date: string; // ISO date string
  result: GameResult;
  compressed_pgn: Uint8Array; // Compressed PGN data
  white_elo: number;
  black_elo: number;
  time_control: Speed;
}

export interface PlayerGame extends MasterGame {
  site: Site;
}

export interface MasterGamePosition {
  game_id: number;
  position_id: number;
  move_number: number;
}

export interface PlayerGamePosition extends MasterGamePosition {}
