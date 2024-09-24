"use client";

import { useState } from "react";
import { Input } from "@/components/ui/input";
import { Button } from "@/components/ui/button";
import { useBoardContext } from "@/context/UsernameContext";
import { fetchAndStoreChessGames } from "@/lib/wasm/controller";

const initialState = {
  success: false,
  numGames: 0,
  username: "",
  error: null,
};

function SubmitButton() {
  const [pending, setPending] = useState(false);
  return (
    <Button type="submit" disabled={pending}>
      {pending ? "Submitting..." : "Submit"}
    </Button>
  );
}

export default function ChessUsername() {
  const [state, setState] = useState(initialState);
  const { setUsername } = useBoardContext();

  async function handleFormSubmit(event: React.FormEvent<HTMLFormElement>) {
    event.preventDefault();

    const formData = new FormData(event.target as HTMLFormElement);
    const username = formData.get("username") as string;

    setState({ ...state, success: false, error: null });

    try {
      let totalGames = 0;
      // Run the download action 3 times (fetch chess games)
      for (let i = 0; i < 3; i++) {
        const result = await fetchAndStoreChessGames(username, 2024, 9); // Example year/month
        // Assuming result contains the number of games, adjust accordingly
        totalGames += 1;
      }

      setState({ success: true, numGames: totalGames, username, error: null });
      setUsername(username);
    } catch (error) {
      setState({ ...state, error: (error as Error).message });
    }
  }

  return (
    <form onSubmit={handleFormSubmit} className="space-y-4">
      <div>
        <Input
          type="text"
          name="username"
          placeholder="Enter chess.com username"
          required
        />
      </div>
      <SubmitButton />
      {state.success && (
        <div>
          <p>Success Downloading {state.numGames} Games</p>
          <p>(Around {state.numGames * 49} Moves)</p>
          <p>Username: {state.username}</p>
        </div>
      )}
      {state.error && <p className="text-red-500">{state.error}</p>}
    </form>
  );
}
