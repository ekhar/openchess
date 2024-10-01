"use server";
// src/lib/wasm/fetchChessGames.ts

export async function getChessGames(
  username: string,
  year: number,
  month: number,
): Promise<string> {
  const url = `https://api.chess.com/pub/player/${username}/games/${year}/${String(month).padStart(2, "0")}/pgn`;

  try {
    const response = await fetch(url, {
      method: "GET",
      headers: {
        "User-Agent": "My Chess App",
      },
    });

    if (!response.ok) {
      throw new Error(`Request failed with status: ${response.status}`);
    }

    const pgnData = await response.text(); // Get PGN data as text
    return pgnData;
  } catch (error: any) {
    throw new Error(`Error fetching chess games: ${error.message}`);
  }
}
