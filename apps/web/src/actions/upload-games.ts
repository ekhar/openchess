"use server";
import { upload_games } from "@/services/username";

export async function uploadGames(prevState: any, formData: FormData) {
  const username = formData.get("username") as string;
  try {
    const response = await upload_games(username);
    if (response) {
      return {
        success: true,
        numGames: response.len_games,
        username,
        error: null,
      };
    } else {
      return { success: false, error: "Request failed", username, numGames: 0 };
    }
  } catch (error) {
    console.error("Error:", error);
    return {
      success: false,
      error: "An error occurred",
      username,
      numGames: 0,
    };
  }
}
