"use client";
import React, { useState, useEffect, useCallback, useMemo } from "react";
import { master_moves } from "@/services/MasterMoves";
import {
  type ChartDataItem,
  type MastersApiResponse,
} from "@/types/lichess-api";
import StatsChart from "@/components/BarChart/StatsChart";

interface MasterStatsProps {
  fen: string;
}

export default function MasterStats({ fen }: MasterStatsProps) {
  const [masterStatsData, setMasterStatsData] = useState<
    MastersApiResponse | undefined
  >(undefined);

  const fetchMasterData = useCallback(async () => {
    if (fen) {
      try {
        const masterData = await master_moves(fen);
        setMasterStatsData(masterData);
      } catch (error) {
        console.error("Error fetching master data:", error);
      }
    }
  }, [fen]);

  useEffect(() => {
    fetchMasterData().catch((error) =>
      console.error("Error fetching master data:", error),
    );
  }, [fetchMasterData]);

  const processChartData = useCallback(
    (data: MastersApiResponse | undefined): ChartDataItem[] => {
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
        avgElo: move.averageRating,
        label: `${move.san} (avg elo ${move.averageRating})`,
      }));

      return [currentPositionData, ...movesData];
    },
    [],
  );

  const masterChartData = useMemo(
    () => processChartData(masterStatsData),
    [masterStatsData, processChartData],
  );

  return (
    <>
      {masterStatsData ? (
        <StatsChart chartData={masterChartData} />
      ) : (
        <div className="flex h-[400px] w-full items-center justify-center">
          Loading master data...
        </div>
      )}
    </>
  );
}
