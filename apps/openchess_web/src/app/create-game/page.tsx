"use client";
import React, { useState, useEffect, useCallback } from "react";
import { useBoardContext } from "@/context/UsernameContext";
import { Chess } from "chess.js";
import { Button } from "@/components/ui/button";
import { ChevronLeft, ChevronRight } from "lucide-react";
import { supabase } from "@/utils/supabase/supabaseClient";
import { compressPosition, decompressPosition } from "@/lib/chess_compression";

interface BoardProps {
  gameId: string;
  playerId: string;
}

const Board: React.FC<BoardProps> = ({ gameId, playerId }) => {
  const { fen, setFen, color, setColor } = useBoardContext();
  const [chess] = useState(new Chess());
  const [history, setHistory] = useState<string[]>([]);
  const [currentMove, setCurrentMove] = useState(0);
  const [playerColor, setPlayerColor] = useState<"white" | "black" | null>(
    null,
  );

  useEffect(() => {
    fetchGameState();

    const subscription = supabase
      .channel(`game_${gameId}`)
      .on(
        "postgres_changes",
        {
          event: "UPDATE",
          schema: "public",
          table: "live_games",
          filter: `id=eq.${gameId}`,
        },
        handleGameUpdate,
      )
      .subscribe();

    return () => {
      subscription.unsubscribe();
    };
  }, [gameId]);

  const fetchGameState = async () => {
    try {
      const { data, error } = await supabase
        .from("live_games")
        .select("*")
        .eq("id", gameId)
        .single();

      if (error) throw error;

      if (data) {
        setPlayerColor(data.white_player === playerId ? "white" : "black");
        const newHistory = await Promise.all(
          data.moves.map(decompressPosition),
        );
        chess.load(newHistory[newHistory.length - 1] || chess.fen());
        setFen(chess.fen());
        setHistory(newHistory);
        setCurrentMove(newHistory.length - 1);
        setColor(chess.turn() === "w" ? "white" : "black");
      }
    } catch (error) {
      console.error("Error fetching game state:", error);
    }
  };

  const handleGameUpdate = async (payload: any) => {
    const { new: newGame } = payload;
    if (newGame.moves && newGame.moves.length > history.length) {
      try {
        const lastMove = newGame.moves[newGame.moves.length - 1];
        const decompressedFen = await decompressPosition(lastMove);
        if (decompressedFen) {
          chess.load(decompressedFen);
          setFen(decompressedFen);
          setHistory((prevHistory) => [...prevHistory, decompressedFen]);
          setCurrentMove(newGame.moves.length - 1);
          setColor(chess.turn() === "w" ? "white" : "black");
        }
      } catch (error) {
        console.error("Error handling game update:", error);
      }
    }
    if (newGame.status === "finished") {
      alert("Game over! An invalid move was played.");
    }
  };

  const handlePieceDrop = async (
    sourceSquare: string,
    targetSquare: string,
  ): Promise<boolean> => {
    if (chess.turn() !== playerColor?.[0]) {
      return false; // Not this player's turn
    }

    const move = {
      from: sourceSquare,
      to: targetSquare,
      promotion: "q", // Automatically promoting to a queen for simplicity
    };

    try {
      const result = chess.move(move);
      if (result) {
        const newFen = chess.fen();
        const compressedPosition = await compressPosition(newFen);
        if (compressedPosition) {
          const { error } = await supabase
            .from("live_games")
            .update({
              moves: supabase.sql`array_append(moves, ${compressedPosition})`,
              current_turn: chess.turn() === "w" ? "white" : "black",
            })
            .eq("id", gameId);

          if (error) throw error;

          setFen(newFen);
          setColor(color === "white" ? "black" : "white");
          setHistory((prevHistory) => [
            ...prevHistory.slice(0, currentMove + 1),
            newFen,
          ]);
          setCurrentMove((prevMove) => prevMove + 1);
          return true;
        }
      }
      return false;
    } catch (error) {
      console.error("Error making move:", error);
      return false;
    }
  };

  const goToPreviousMove = useCallback(() => {
    if (currentMove > 0) {
      const previousMove = currentMove - 1;
      const previousFen = history[previousMove];
      chess.load(previousFen!);
      setFen(previousFen!);
      setColor(previousMove % 2 === 0 ? "white" : "black");
      setCurrentMove(previousMove);
    }
  }, [currentMove, history, chess, setFen, setColor]);

  const goToNextMove = useCallback(() => {
    if (currentMove < history.length - 1) {
      const nextMove = currentMove + 1;
      const nextFen = history[nextMove];
      chess.load(nextFen!);
      setFen(nextFen!);
      setColor(nextMove % 2 === 0 ? "white" : "black");
      setCurrentMove(nextMove);
    }
  }, [currentMove, history, chess, setFen, setColor]);

  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "ArrowLeft") {
        goToPreviousMove();
      } else if (event.key === "ArrowRight") {
        goToNextMove();
      }
    };
    window.addEventListener("keydown", handleKeyDown);
    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [goToPreviousMove, goToNextMove]);

  const width = 400;
  const height = 400;

  return (
    <div>
      <div id="myBoard" style={{ width: width, height: height }}>
        <Chessboard position={fen} onPieceDrop={handlePieceDrop} />
      </div>
      <div className="mt-4 flex justify-center space-x-4">
        <Button onClick={goToPreviousMove} disabled={currentMove === 0}>
          <ChevronLeft className="mr-2 h-4 w-4" /> Previous
        </Button>
        <Button
          onClick={goToNextMove}
          disabled={currentMove === history.length - 1}
        >
          Next <ChevronRight className="ml-2 h-4 w-4" />
        </Button>
      </div>
    </div>
  );
};

export default Board;
