// src/types/database.ts

import {
  type MasterGame,
  type MasterGamePosition,
  type PlayerGame,
  type PlayerGamePosition,
  type Position,
} from "@/types/schemas";

export type Json =
  | string
  | number
  | boolean
  | null
  | { [key: string]: Json }
  | Json[];

export interface Database {
  public: {
    Tables: {
      positions: {
        Row: Position;
        Insert: Omit<Position, "id">; // 'id' is auto-generated
        Update: Partial<Omit<Position, "id">>;
      };
      master_games: {
        Row: MasterGame;
        Insert: Omit<MasterGame, "id">;
        Update: Partial<Omit<MasterGame, "id">>;
      };
      master_game_positions: {
        Row: MasterGamePosition;
        Insert: MasterGamePosition;
        Update: Partial<MasterGamePosition>;
      };
      player_games: {
        Row: PlayerGame;
        Insert: Omit<PlayerGame, "id">;
        Update: Partial<Omit<PlayerGame, "id">>;
      };
      player_game_positions: {
        Row: PlayerGamePosition;
        Insert: PlayerGamePosition;
        Update: Partial<PlayerGamePosition>;
      };
    };
    Enums: {
      game_result: "white" | "black" | "draw";
      site: "chesscom" | "lichess" | "custom";
      speed:
        | "ultrabullet"
        | "bullet"
        | "blitz"
        | "rapid"
        | "classical"
        | "correspondence";
    };
    // Add other schema elements if necessary (e.g., Views, Functions)
  };
}