// app/game/[gameId]/page.tsx
"use client";

import { useState, useEffect } from "react";
import { useParams } from "next/navigation";
import { supabase } from "@/lib/supabaseClient";
import { Chessboard } from "react-chessboard";
import Chess from "chess.js";

export default function GamePage() {
  const { gameId } = useParams();
  const [game, setGame] = useState(new Chess());
  const [gameData, setGameData] = useState(null);
  const [isPlayerTurn, setIsPlayerTurn] = useState(true);

  useEffect(() => {
    // Fetch game data from Supabase
    const fetchGameData = async () => {
      const { data, error } = await supabase
        .from("live_games")
        .select("*")
        .eq("id", gameId)
        .single();

      if (error) {
        console.error(error);
      } else {
        setGameData(data);
        // Load moves into the game
        if (data.moves && data.moves.length > 0) {
          data.moves.forEach((move: string) => game.move(move));
        }
      }
    };

    fetchGameData();
  }, [gameId]);

  // Handle making a move
  const onDrop = async (sourceSquare: string, targetSquare: string) => {
    // Ignore if not player's turn
    if (!isPlayerTurn) return false;

    const move = game.move({
      from: sourceSquare,
      to: targetSquare,
      promotion: "q", // Always promote to queen for simplicity
    });

    if (move === null) return false;

    // Check if the new position exists in the database
    const positionExists = await checkPositionExists(game.fen());
    if (!positionExists) {
      alert("Position does not exist in the database. You lose!");
      // Update game status in the database
      await updateGameStatus("finished");
      return false;
    }

    // Update moves in the database
    await updateGameMoves(move.san);

    // Switch turn
    setIsPlayerTurn(false);

    return true;
  };

  // Function to check if position exists in the database
  const checkPositionExists = async (fen: string) => {
    const { data, error } = await supabase.rpc("check_position_exists", {
      fen,
    });
    if (error) {
      console.error(error);
      return false;
    }
    return data.exists;
  };

  // Function to update moves in the database
  const updateGameMoves = async (sanMove: string) => {
    const { data, error } = await supabase
      .from("live_games")
      .update({ moves: [...(gameData.moves || []), sanMove] })
      .eq("id", gameId);

    if (error) {
      console.error(error);
    } else {
      setGameData(data[0]);
    }
  };

  // Function to update game status
  const updateGameStatus = async (status: string) => {
    await supabase.from("live_games").update({ status }).eq("id", gameId);
  };

  return (
    <div>
      <h1>Chess Game ID: {gameId}</h1>
      <Chessboard position={game.fen()} onPieceDrop={onDrop} />
    </div>
  );
}
