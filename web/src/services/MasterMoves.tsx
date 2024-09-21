"use server";

import { type MastersApiResponse } from "@/types/lichess-api";
import { unstable_cache } from "next/cache";

export async function master_moves(
  fen: string,
): Promise<MastersApiResponse | undefined> {
  return unstable_cache(
    async () => {
      const baseUrl = process.env.BACKEND_URL;
      if (!baseUrl) {
        throw new Error("BACKEND_URL is not defined in environment variables");
      }
      const url = `${baseUrl}/masters?${new URLSearchParams({
        fen,
      }).toString()}`;
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
        const jsonResponse: MastersApiResponse = await response.json();
        return jsonResponse;
      } catch (error) {
        console.error("Request failed:", error);
        return undefined;
      }
    },
    [`master_moves_${fen}`],
    {
      revalidate: 86400, // 24 hours in seconds
      tags: [`master_moves_${fen}`],
    },
  )();
}
