"use client";
import React, { useState, useEffect, useCallback } from "react";
import { Chessboard } from "react-chessboard";
import { useBoardContext } from "@/context/UsernameContext"; // Adjust this path as needed
import { Chess } from "chess.js";
import { Button } from "@/components/ui/button";
import { ChevronLeft, ChevronRight } from "lucide-react";

const Board = () => {
  const { fen, setFen, color, setColor } = useBoardContext();
  const [chess] = useState(new Chess());
  const [history, setHistory] = useState<string[]>([]);
  const [currentMove, setCurrentMove] = useState(0);

  useEffect(() => {
    chess.load(fen);
    setHistory([fen]);
  }, []);

  const handlePieceDrop = (
    sourceSquare: string,
    targetSquare: string,
  ): boolean => {
    const move = {
      from: sourceSquare,
      to: targetSquare,
      promotion: "q", // Automatically promoting to a queen for simplicity
    };
    try {
      chess.move(move);
      const newFen = chess.fen();
      setFen(newFen);
      setColor(color === "white" ? "black" : "white");
      setHistory((prevHistory) => [
        ...prevHistory.slice(0, currentMove + 1),
        newFen,
      ]);
      setCurrentMove((prevMove) => prevMove + 1);
      return true;
    } catch {
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
