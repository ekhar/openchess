// filename: apps/supabase/functions/import-user-games/index.ts
// Follow this setup guide to integrate the Deno language server with your editor:
// https://deno.land/manual/getting_started/setup_your_environment
// This enables autocomplete, go to definition, etc.

// Setup type definitions for built-in Supabase Runtime APIs
import "jsr:@supabase/functions-js/edge-runtime.d.ts";

/**
 * Fetches chess games in PGN format for a specific user, year, and month
 * @param {string} username - Chess.com username
 * @param {number} year - Year of games
 * @param {number} month - Month of games (1-12)
 * @returns {Promise<Uint8Array>} - PGN data as bytes
 */
async function getChessGames(
  username: string,
  year: number,
  month: number,
): Promise<Uint8Array> {
  // Format the URL with padding for month (e.g., 01, 02, etc.)
  const url = `https://api.chess.com/pub/player/${username}/games/${year}/${
    String(month).padStart(2, "0")
  }/pgn`;

  // Create headers with User-Agent
  const headers = {
    "User-Agent": "My Chess App",
  };

  // Make the request
  const response = await fetch(url, { headers });

  // Check if the request was successful
  if (response.ok) {
    // Get the raw bytes
    const arrayBuffer = await response.arrayBuffer();
    return new Uint8Array(arrayBuffer);
  } else {
    throw new Error(`Request failed with status: ${response.status}`);
  }
}

/**
 * Processes PGN data to extract game information
 * @param {Uint8Array} pgnData - Raw PGN data
 * @returns {any} - Processed game data
 */
function processPgnData(pgnData: Uint8Array): any {
  // Convert binary data to string
  const pgnText = new TextDecoder().decode(pgnData);

  // Split into individual games
  const games = pgnText
    .split(/\n\n(?=\[Event)/)
    .filter((game) => game.trim().length > 0);

  // Filter for rapid, blitz, and bullet games
  const filteredGames = games.filter((game) => {
    const timeControlMatch = game.match(/\[TimeControl "(.+?)"\]/);
    if (!timeControlMatch) return false;

    const timeControl = timeControlMatch[1];
    // Parse time control
    const baseTime = parseInt(timeControl.split("+")[0], 10);

    // Bullet: < 3 min, Blitz: 3-10 min, Rapid: > 10 min
    return baseTime < 180 || (baseTime >= 180 && baseTime <= 600) ||
      baseTime > 600;
  });

  return {
    total: games.length,
    filtered: filteredGames.length,
    games: filteredGames.join("\n\n"),
  };
}

Deno.serve(async (req) => {
  try {
    // Parse the request body if this is a POST request
    let username = "ekhar02"; // Default username

    if (req.method === "POST") {
      const requestText = await req.text();
      // If body is not empty, use it as username
      if (requestText.trim().length > 0) {
        username = requestText.trim();
      }
    } else if (req.method === "GET") {
      // Check for username in query params
      const url = new URL(req.url);
      const queryUsername = url.searchParams.get("username");
      if (queryUsername) {
        username = queryUsername;
      }
    }

    console.log(`Importing games for ${username}`);

    // Get current year and month
    const now = new Date();
    const currentYear = now.getFullYear();
    const currentMonth = now.getMonth() + 1; // JavaScript months are 0-indexed

    // Fetch the last 3 months of games, similar to the Rust code
    console.log(`Fetching games for ${username} for the last 3 months`);

    // Calculate the months to fetch
    const monthsToFetch = [];
    for (let i = 0; i < 3; i++) {
      let month = currentMonth - i;
      let year = currentYear;

      if (month <= 0) {
        month += 12;
        year -= 1;
      }

      monthsToFetch.push({ year, month });
    }

    // Fetch all the PGN data
    const pgnDataPromises = monthsToFetch.map(({ year, month }) =>
      getChessGames(username, year, month)
        .catch((error) => {
          console.error(
            `Error fetching games for ${year}-${month}: ${error.message}`,
          );
          return new Uint8Array();
        })
    );

    const pgnDataResults = await Promise.all(pgnDataPromises);

    // Combine all the PGN data
    let combinedPgn = new Uint8Array();
    for (const pgnData of pgnDataResults) {
      const newCombined = new Uint8Array(combinedPgn.length + pgnData.length);
      newCombined.set(combinedPgn);
      newCombined.set(pgnData, combinedPgn.length);
      combinedPgn = newCombined;
    }

    // Process the combined PGN data
    const processedData = processPgnData(combinedPgn);

    // For download request, return the PGN file
    const url = new URL(req.url);
    if (url.searchParams.get("download") === "true") {
      return new Response(processedData.games, {
        headers: {
          "Content-Type": "application/x-chess-pgn",
          "Content-Disposition":
            `attachment; filename="ChessCom_${username}_filtered.pgn"`,
        },
      });
    }

    // Otherwise return JSON response
    return new Response(
      JSON.stringify(
        {
          username,
          total_games: processedData.total,
          filtered_games: processedData.filtered,
          months_fetched: monthsToFetch,
        },
        null,
        2,
      ),
      { headers: { "Content-Type": "application/json" } },
    );
  } catch (error) {
    console.error("Error importing chess games:", error);

    return new Response(
      JSON.stringify({ error: error.message }),
      {
        status: 500,
        headers: { "Content-Type": "application/json" },
      },
    );
  }
});

/* To invoke locally:

  1. Run `supabase start` (see: https://supabase.com/docs/reference/cli/supabase-start)
  2. Make an HTTP request:

  // GET request with username as query parameter
  curl -i --location --request GET 'http://127.0.0.1:54321/functions/v1/import-user-games?username=ekhar02' \
    --header 'Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0'

  // POST request with username in the body
  curl -i --location --request POST 'http://127.0.0.1:54321/functions/v1/import-user-games' \
    --header 'Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0' \
    --header 'Content-Type: text/plain' \
    --data 'ekhar02'

  // Download the PGN file
  curl -i --location --request GET 'http://127.0.0.1:54321/functions/v1/import-user-games?username=ekhar02&download=true' \
    --header 'Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJpc3MiOiJzdXBhYmFzZS1kZW1vIiwicm9sZSI6ImFub24iLCJleHAiOjE5ODM4MTI5OTZ9.CRXP1A7WOeoJeXxjNni43kdQwgnWNReilDMblYTn_I0' \
    --output 'chess_games.pgn'

*/
