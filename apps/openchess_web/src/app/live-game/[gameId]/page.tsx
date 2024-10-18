// pages/game/[gameId].tsx
"use client";
import React, { useEffect, useState } from "react";
import { useRouter, useParams } from "next/navigation";
import MultiBoard from "../_components/MultiBoard";
import { supabase } from "@/utils/supabase/supabaseClient";
import { v4 as uuidv4 } from "uuid";

const GamePage = () => {
  const router = useRouter();
  const params = useParams();
  const gameId = params.gameId as string;

  const [playerId, setPlayerId] = useState<string>("");
  const [playerColor, setPlayerColor] = useState<"white" | "black">("white");
  const [gameExists, setGameExists] = useState<boolean>(false);
  const [status, setStatus] = useState<"waiting" | "ongoing" | "finished">(
    "waiting",
  );
  const [loading, setLoading] = useState<boolean>(true);

  useEffect(() => {
    // Generate a unique player ID
    let storedPlayerId = localStorage.getItem("playerId");
    if (!storedPlayerId) {
      storedPlayerId = uuidv4();
      localStorage.setItem("playerId", storedPlayerId);
    }
    setPlayerId(storedPlayerId);
  }, []);

  useEffect(() => {
    const fetchGame = async () => {
      const { data, error } = await supabase
        .from("live_games")
        .select("*")
        .eq("id", gameId)
        .single();

      if (error) {
        console.error("Error fetching game:", error);
        router.push("/");
        return;
      }

      if (data.status === "finished") {
        alert("Game is already finished.");
        router.push("/");
        return;
      }

      setGameExists(true);
      setStatus(data.status);

      // Check if player is already in the game
      const isWhite = data.players.white === playerId;
      const isBlack = data.players.black === playerId;

      if (!isWhite && !isBlack) {
        // If game is waiting and black player slot is empty, join as black
        if (data.status === "waiting" && data.players.black === null) {
          const { error: updateError } = await supabase
            .from("live_games")
            .update({
              players: { ...data.players, black: playerId },
              status: "ongoing",
            })
            .eq("id", gameId);

          if (updateError) {
            console.error("Error joining game as black:", updateError);
            router.push("/");
            return;
          }

          setPlayerColor("black");
          setStatus("ongoing");
        } else {
          alert("Game is full.");
          router.push("/");
          return;
        }
      } else {
        // Player is already in the game
        setPlayerColor(isWhite ? "white" : "black");
        setStatus(data.status);
      }

      setLoading(false);
    };

    if (playerId) {
      fetchGame();
    }
  }, [gameId, playerId, router]);

  if (loading) {
    return <div>Loading...</div>;
  }

  if (!gameExists) {
    return <div>Game not found.</div>;
  }

  return (
    <div>
      <h1>Chess Game</h1>
      <p>You are playing as {playerColor}</p>
      <MultiBoard
        gameId={gameId}
        playerColor={playerColor}
        playerId={playerId}
      />
    </div>
  );
};

export default GamePage;
