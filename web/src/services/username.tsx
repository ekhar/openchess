"use server";
export async function upload_games(username: string) {
  const baseUrl = process.env.BACKEND_URL; // Access environment variable
  const response = await fetch(`${baseUrl}/games/import`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: username, // Ensure the body is a JSON string
  });

  if (response.ok) {
    const jsonResponse = await response.json(); // Use .json() for JSON
    return jsonResponse;
  } else {
    console.error("Request failed");
  }
}
