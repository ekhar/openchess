export interface Player {
  name: string;
  rating: number;
}

export interface Game {
  id: string;
  winner: "white" | "black" | "draw";
  white: Player;
  black: Player;
  year: number;
  month: string;
}

export interface Move {
  uci: string;
  san: string;
  averageRating: number;
  white: number;
  draws: number;
  black: number;
  game: Game | null;
}

export interface HistoryEntry {
  month: string;
  black: number;
  draws: number;
  white: number;
}

export interface MastersApiResponse {
  opening?: {
    eco: string;
    name: string;
  };
  white: number;
  draws: number;
  black: number;
  moves: Move[];
  topGames: Game[];
  recentGames: Game[];
  history: HistoryEntry[];
}

// New types for PlayerApiResponse
export interface PlayerMove {
  uci: string;
  san: string;
  averageOpponentRating: number;
  performance: number;
  white: number;
  draws: number;
  black: number;
  game: Game | null;
}

export interface PlayerApiResponse {
  white: number;
  draws: number;
  black: number;
  moves: PlayerMove[];
  recentGames: Game[];
  opening?: {
    eco: string;
    name: string;
  };
  queuePosition?: number;
}
export interface ChartDataItem {
  name: string;
  White: number;
  Draw: number;
  Black: number;
  avgElo: number | string;
}
