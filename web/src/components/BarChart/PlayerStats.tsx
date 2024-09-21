"use client";
import React, { useState, useEffect, useCallback, useMemo } from "react";
import { player_moves } from "@/services/PlayerMoves";
import {
  type ChartDataItem,
  type PlayerApiResponse,
} from "@/types/lichess-api";
import StatsChart from "./StatsChart";
import PlayerControls from "./PlayerControls";

interface PlayerStatsProps {
  fen: string;
  username: string | null;
}

export default function PlayerStats({ fen, username }: PlayerStatsProps) {
  const [playerStatsData, setPlayerStatsData] = useState<
    PlayerApiResponse | undefined
  >(undefined);
  const [playerColor, setPlayerColor] = useState<"white" | "black">("white");
  const [selectedSpeeds, setSelectedSpeeds] = useState<string[]>([
    "bullet",
    "blitz",
    "rapid",
  ]);
  const [selectedModes, setSelectedModes] = useState<string[]>(["rated"]);

  const fetchPlayerData = useCallback(async () => {
    if (fen && username) {
      try {
        const playerData = await player_moves(
          fen,
          username,
          playerColor,
          "chesscom",
          selectedSpeeds,
          selectedModes,
        );
        console.log("Player data:", playerData);
        setPlayerStatsData(playerData);
      } catch (error) {
        console.error("Error fetching player data:", error);
      }
    } else {
      setPlayerStatsData(undefined);
    }
  }, [fen, username, playerColor, selectedSpeeds, selectedModes]);

  useEffect(() => {
    fetchPlayerData().catch((error) =>
      console.error("Error fetching player data:", error),
    );
  }, [fetchPlayerData]);

  const processChartData = useCallback(
    (data: PlayerApiResponse | undefined): ChartDataItem[] => {
      if (!data) return [];

      const { white, draws, black, moves } = data;
      const total = white + draws + black;
      if (total === 0) return [];

      const currentPositionData: ChartDataItem = {
        name: "Current Position",
        White: (white / total) * 100,
        Draw: (draws / total) * 100,
        Black: (black / total) * 100,
        avgElo: "-",
      };

      const movesData = moves.map((move) => ({
        name: move.san,
        White: (move.white / (move.white + move.draws + move.black)) * 100,
        Draw: (move.draws / (move.white + move.draws + move.black)) * 100,
        Black: (move.black / (move.white + move.draws + move.black)) * 100,
        avgElo: move.averageOpponentRating,
        label: `${move.san} (avg elo ${move.averageOpponentRating})`,
      }));

      return [currentPositionData, ...movesData];
    },
    [],
  );

  const playerChartData = useMemo(
    () => processChartData(playerStatsData),
    [playerStatsData, processChartData],
  );

  return (
    <>
      {username ? (
        <>
          <PlayerControls
            username={username}
            playerColor={playerColor}
            setPlayerColor={setPlayerColor}
            selectedSpeeds={selectedSpeeds}
            setSelectedSpeeds={setSelectedSpeeds}
            selectedModes={selectedModes}
            setSelectedModes={setSelectedModes}
          />
          {playerStatsData ? (
            <StatsChart chartData={playerChartData} />
          ) : (
            <div className="flex h-[400px] w-full items-center justify-center">
              Loading player data...
            </div>
          )}
        </>
      ) : (
        <div className="flex h-[400px] w-full items-center justify-center">
          Please enter a username to view player statistics.
        </div>
      )}
    </>
  );
}
