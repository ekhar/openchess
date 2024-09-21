"use client";
import { createContext, useContext, useState, useEffect } from "react";

interface BoardContextType {
  username: string;
  fen: string;
  color: "white" | "black";
  setUsername: (username: string) => void;
  setFen: (fen: string) => void;
  setColor: (color: "white" | "black") => void;
}

const default_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

const BoardContext = createContext<BoardContextType>({
  username: "",
  color: "white",
  fen: default_fen,
  setUsername: () => "",
  setFen: () => "",
  setColor: () => "",
});

export const useBoardContext = () => useContext(BoardContext);

export const BoardInfoProvider: React.FC<{ children: React.ReactNode }> = ({
  children,
}) => {
  const [username, setUsernameState] = useState(() => {
    if (typeof window !== "undefined") {
      return localStorage.getItem("username") ?? "";
    }
    return "";
  });
  const [fen, setFen] = useState(default_fen);
  const [color, setColor] = useState<"white" | "black">("white");

  const setUsername = (newUsername: string) => {
    setUsernameState(newUsername);
    if (typeof window !== "undefined") {
      localStorage.setItem("username", newUsername);
    }
  };

  useEffect(() => {
    if (typeof window !== "undefined") {
      const storedUsername = localStorage.getItem("username");
      if (storedUsername) {
        setUsernameState(storedUsername);
      }
    }
  }, []);

  return (
    <BoardContext.Provider
      value={{ username, fen, color, setColor, setUsername, setFen }}
    >
      {children}
    </BoardContext.Provider>
  );
};
