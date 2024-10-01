"use server";

import { type PlayerApiResponse } from "@/types/lichess-api";
import { unstable_cache } from "next/cache";

export async function player_moves(
  fen: string,
  player: string,
  color: "white" | "black",
  chess_site = "chesscom",
  speeds?: string[],
  modes?: string[],
): Promise<PlayerApiResponse | undefined> {
  return unstable_cache(
    async () => {
      const baseUrl = process.env.BACKEND_URL;
      if (!baseUrl) {
        throw new Error("BACKEND_URL is not defined in environment variables");
      }
      const params = new URLSearchParams({ fen, player, color, chess_site });
      if (speeds && speeds.length > 0)
        params.append("speeds", speeds.join(","));
      if (modes && modes.length > 0) params.append("modes", modes.join(","));
      const url = `${baseUrl}/player?${params.toString()}`;

      try {
        const response = await fetch(url, {
          method: "GET",
          headers: {
            "Content-Type": "application/json",
          },
        });
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        const jsonResponse: PlayerApiResponse = await response.json();
        console.log(jsonResponse);
        return jsonResponse;
      } catch (error) {
        console.error("Request failed:", error);
        return undefined;
      }
    },
    [
      `player_moves_${fen}_${player}_${color}_${chess_site}_${speeds?.join("_")}_${modes?.join("_")}`,
    ],
    {
      revalidate: 3600, // 1 hour in seconds
      tags: [`player_moves_${fen}_${player}_${color}_${chess_site}`],
    },
  )();
}
