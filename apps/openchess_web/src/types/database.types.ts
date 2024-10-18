export type Json =
  | string
  | number
  | boolean
  | null
  | { [key: string]: Json | undefined }
  | Json[]

export type Database = {
  graphql_public: {
    Tables: {
      [_ in never]: never
    }
    Views: {
      [_ in never]: never
    }
    Functions: {
      graphql: {
        Args: {
          operationName?: string
          query?: string
          variables?: Json
          extensions?: Json
        }
        Returns: Json
      }
    }
    Enums: {
      [_ in never]: never
    }
    CompositeTypes: {
      [_ in never]: never
    }
  }
  public: {
    Tables: {
      games_foreign: {
        Row: {
          black_elo: number | null
          black_player: string | null
          date: string | null
          eco: string | null
          id: number
          pgn_moves: string | null
          result: Database["public"]["Enums"]["result"] | null
          time_control: Database["public"]["Enums"]["chess_speed"] | null
          white_elo: number | null
          white_player: string | null
        }
        Insert: {
          black_elo?: number | null
          black_player?: string | null
          date?: string | null
          eco?: string | null
          id?: number
          pgn_moves?: string | null
          result?: Database["public"]["Enums"]["result"] | null
          time_control?: Database["public"]["Enums"]["chess_speed"] | null
          white_elo?: number | null
          white_player?: string | null
        }
        Update: {
          black_elo?: number | null
          black_player?: string | null
          date?: string | null
          eco?: string | null
          id?: number
          pgn_moves?: string | null
          result?: Database["public"]["Enums"]["result"] | null
          time_control?: Database["public"]["Enums"]["chess_speed"] | null
          white_elo?: number | null
          white_player?: string | null
        }
        Relationships: []
      }
      live_games: {
        Row: {
          created_at: string
          current_position: string
          id: string
          moves: string | null
          players: Json
          status: Database["public"]["Enums"]["game_status"]
          turn: Database["public"]["Enums"]["turn"]
        }
        Insert: {
          created_at?: string
          current_position?: string
          id: string
          moves?: string | null
          players: Json
          status?: Database["public"]["Enums"]["game_status"]
          turn?: Database["public"]["Enums"]["turn"]
        }
        Update: {
          created_at?: string
          current_position?: string
          id?: string
          moves?: string | null
          players?: Json
          status?: Database["public"]["Enums"]["game_status"]
          turn?: Database["public"]["Enums"]["turn"]
        }
        Relationships: []
      }
      positions_foreign: {
        Row: {
          game_id: number | null
          id: number
          move_number: number | null
          position: string | null
        }
        Insert: {
          game_id?: number | null
          id?: number
          move_number?: number | null
          position?: string | null
        }
        Update: {
          game_id?: number | null
          id?: number
          move_number?: number | null
          position?: string | null
        }
        Relationships: []
      }
    }
    Views: {
      [_ in never]: never
    }
    Functions: {
      check_position_exists: {
        Args: {
          game_position: string
        }
        Returns: boolean
      }
      postgres_fdw_disconnect: {
        Args: {
          "": string
        }
        Returns: boolean
      }
      postgres_fdw_disconnect_all: {
        Args: Record<PropertyKey, never>
        Returns: boolean
      }
      postgres_fdw_get_connections: {
        Args: Record<PropertyKey, never>
        Returns: Record<string, unknown>[]
      }
      postgres_fdw_handler: {
        Args: Record<PropertyKey, never>
        Returns: unknown
      }
    }
    Enums: {
      chess_speed:
        | "UltraBullet"
        | "Bullet"
        | "Blitz"
        | "Rapid"
        | "Classical"
        | "Correspondence"
      game_status: "waiting" | "ongoing" | "finished"
      result: "white" | "black" | "draw"
      turn: "white" | "black"
    }
    CompositeTypes: {
      [_ in never]: never
    }
  }
}

type PublicSchema = Database[Extract<keyof Database, "public">]

export type Tables<
  PublicTableNameOrOptions extends
    | keyof (PublicSchema["Tables"] & PublicSchema["Views"])
    | { schema: keyof Database },
  TableName extends PublicTableNameOrOptions extends { schema: keyof Database }
    ? keyof (Database[PublicTableNameOrOptions["schema"]]["Tables"] &
        Database[PublicTableNameOrOptions["schema"]]["Views"])
    : never = never,
> = PublicTableNameOrOptions extends { schema: keyof Database }
  ? (Database[PublicTableNameOrOptions["schema"]]["Tables"] &
      Database[PublicTableNameOrOptions["schema"]]["Views"])[TableName] extends {
      Row: infer R
    }
    ? R
    : never
  : PublicTableNameOrOptions extends keyof (PublicSchema["Tables"] &
        PublicSchema["Views"])
    ? (PublicSchema["Tables"] &
        PublicSchema["Views"])[PublicTableNameOrOptions] extends {
        Row: infer R
      }
      ? R
      : never
    : never

export type TablesInsert<
  PublicTableNameOrOptions extends
    | keyof PublicSchema["Tables"]
    | { schema: keyof Database },
  TableName extends PublicTableNameOrOptions extends { schema: keyof Database }
    ? keyof Database[PublicTableNameOrOptions["schema"]]["Tables"]
    : never = never,
> = PublicTableNameOrOptions extends { schema: keyof Database }
  ? Database[PublicTableNameOrOptions["schema"]]["Tables"][TableName] extends {
      Insert: infer I
    }
    ? I
    : never
  : PublicTableNameOrOptions extends keyof PublicSchema["Tables"]
    ? PublicSchema["Tables"][PublicTableNameOrOptions] extends {
        Insert: infer I
      }
      ? I
      : never
    : never

export type TablesUpdate<
  PublicTableNameOrOptions extends
    | keyof PublicSchema["Tables"]
    | { schema: keyof Database },
  TableName extends PublicTableNameOrOptions extends { schema: keyof Database }
    ? keyof Database[PublicTableNameOrOptions["schema"]]["Tables"]
    : never = never,
> = PublicTableNameOrOptions extends { schema: keyof Database }
  ? Database[PublicTableNameOrOptions["schema"]]["Tables"][TableName] extends {
      Update: infer U
    }
    ? U
    : never
  : PublicTableNameOrOptions extends keyof PublicSchema["Tables"]
    ? PublicSchema["Tables"][PublicTableNameOrOptions] extends {
        Update: infer U
      }
      ? U
      : never
    : never

export type Enums<
  PublicEnumNameOrOptions extends
    | keyof PublicSchema["Enums"]
    | { schema: keyof Database },
  EnumName extends PublicEnumNameOrOptions extends { schema: keyof Database }
    ? keyof Database[PublicEnumNameOrOptions["schema"]]["Enums"]
    : never = never,
> = PublicEnumNameOrOptions extends { schema: keyof Database }
  ? Database[PublicEnumNameOrOptions["schema"]]["Enums"][EnumName]
  : PublicEnumNameOrOptions extends keyof PublicSchema["Enums"]
    ? PublicSchema["Enums"][PublicEnumNameOrOptions]
    : never

