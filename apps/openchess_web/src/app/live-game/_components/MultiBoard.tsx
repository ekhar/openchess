// ./_components/MultiBoard.tsx
"use client";

import React, { useState, useEffect, useCallback } from "react";
import { Chessboard } from "react-chessboard";
import { Chess, type Move } from "chess.js"; // Import Move type
import { supabase } from "@/utils/supabase/supabaseClient";
import {
  compressPosition,
  decompressPosition,
  compressPgn,
  decompressPgn,
} from "@/lib/chess_compression";

import { type Database } from "@/types/database.types";

type PlayerColor = "white" | "black";
type GameStatus = "waiting" | "ongoing" | "finished";
const encoder = new TextEncoder();
interface MultiBoardProps {
  gameId: string;
  playerColor: PlayerColor;
  playerId: string;
}
type LiveGame = Database["public"]["Tables"]["live_games"]["Row"];
const MultiBoard: React.FC<MultiBoardProps> = ({
  gameId,
  playerColor,
  playerId,
}) => {
  const [chess] = useState(new Chess());
  const [fen, setFen] = useState(chess.fen());
  const [history, setHistory] = useState<string[]>([]);
  const [currentMove, setCurrentMove] = useState(0);
  const [turn, setTurn] = useState<PlayerColor>("white");
  const [status, setStatus] = useState<GameStatus>("waiting");
  const [gameLoaded, setGameLoaded] = useState(false);
  const [isProcessing, setIsProcessing] = useState(false); // To prevent multiple simultaneous moves

  // Fetch initial game state
  const fetchGame = useCallback(async () => {
    const { data, error } = await supabase
      .from("live_games")
      .select()
      .eq("id", gameId)
      .single();

    if (error) {
      console.error("Error fetching game:", error);
      return;
    }

    if (data.status === "finished") {
      alert("Game is already finished.");
      return;
    }

    const decompressedPosition = await decompressPosition(
      encoder.encode(data.current_position),
    );
    if (!decompressedPosition) {
      console.error("Failed to decompress position");
      return;
    }

    chess.load(decompressedPosition);
    setFen(decompressedPosition);
    setTurn(data.turn);
    setHistory(data.moves ? await decompressMoves(data.moves) : []);
    setCurrentMove(data.moves ? (await decompressMoves(data.moves)).length : 0);
    setStatus(data.status);
    setGameLoaded(true);
  }, [gameId, chess]);

  useEffect(() => {
    fetchGame();
  }, [fetchGame]);

  // Real-time subscription
  useEffect(() => {
    const channel = supabase
      .channel(`live_games:id=eq.${gameId}`)
      .on(
        "postgres_changes",
        {
          event: "*",
          schema: "public",
          table: "live_games",
          filter: `id=eq.${gameId}`,
        },
        async (payload) => {
          const updatedGame = payload.new as LiveGame;
          if (updatedGame.status === "finished") {
            setStatus("finished");
            alert("Game over.");
            return;
          }

          const decompressedPosition = await decompressPosition(
            updatedGame.current_position,
          );
          if (!decompressedPosition) {
            console.error("Failed to decompress position");
            return;
          }
          chess.load(decompressedPosition);
          setFen(decompressedPosition);
          setTurn(updatedGame.turn);
          setHistory(updatedGame.moves || []);
          setCurrentMove(updatedGame.moves?.length || 0);
        },
      )
      .subscribe();

    return () => {
      supabase.removeChannel(channel);
    };
  }, [gameId, chess]);

  const handlePieceDrop = (
    sourceSquare: string,
    targetSquare: string,
  ): boolean => {
    if (status !== "ongoing") {
      alert("Game is not ongoing.");
      return false;
    }

    if (turn !== playerColor) {
      alert("It's not your turn.");
      return false;
    }

    const move = {
      from: sourceSquare,
      to: targetSquare,
      promotion: "q", // Automatically promoting to a queen for simplicity
    };

    const result: Move | null = chess.move(move);
    if (result === null) return false;

    const newFen = chess.fen();
    setFen(newFen);
    setTurn(turn === "white" ? "black" : "white");
    setHistory([...history, result.san]);
    setCurrentMove(currentMove + 1);

    // Handle asynchronous database update
    updateGameState(newFen, result.san);

    return true;
  };

  const updateGameState = async (newFen: string, sanMove: string) => {
    if (isProcessing) return; // Prevent multiple updates
    setIsProcessing(true);

    try {
      const compressedPosition = await compressPosition(newFen);
      if (!compressedPosition) {
        console.error("Failed to compress position");
        // Optionally revert the move here
        return;
      }

      // Check if position exists in the database
      const { data: existsData, error: existsError } = await supabase.rpc(
        "check_position_exists",
        { game_position: compressedPosition },
      );

      if (existsError) {
        console.error("Error checking position existence:", existsError);
        // Optionally revert the move here
        return;
      }

      if (!existsData) {
        await supabase
          .from("live_games")
          .update({ status: "finished" })
          .eq("id", gameId);
        alert("Position does not exist in the database. You lose.");
        // Optionally revert the move here
        return;
      }

      const updatedMoves = [...history, sanMove];

      const { error: updateError } = await supabase
        .from("live_games")
        .update({
          current_position: compressedPosition,
          moves: updatedMoves,
          turn: turn === "white" ? "black" : "white",
        })
        .eq("id", gameId);

      if (updateError) {
        console.error("Error updating game state:", updateError);
        // Optionally revert the move here
        return;
      }
    } catch (error) {
      console.error("Unexpected error updating game state:", error);
      // Optionally revert the move here
    } finally {
      setIsProcessing(false);
    }
  };

  const width = 400;

  if (!gameLoaded) {
    return <div>Loading game...</div>;
  }

  return (
    <div>
      <div id="MultiBoard" style={{ width: width }}>
        <Chessboard
          position={fen}
          onPieceDrop={handlePieceDrop}
          boardOrientation={playerColor}
        />
      </div>
    </div>
  );
};

export default MultiBoard;
