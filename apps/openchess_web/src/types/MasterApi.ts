export interface MoveStats {
  uci: string;
  san: string;
  white: number;
  draws: number;
  black: number;
}

export interface MastersApiResponse {
  white: number;
  draws: number;
  black: number;
  moves: MoveStats[];
}
