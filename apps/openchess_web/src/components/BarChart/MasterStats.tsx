// filename: apps/openchess_web/src/components/BarChart/MasterStats.tsx
"use client";
import React, { useState, useEffect, useCallback, useMemo } from "react";
import { master_moves, type MastersApiResponse } from "@/services/MasterMoves";
import StatsChart from "@/components/BarChart/StatsChart";

export interface ChartDataItem {
  name: string;
  White: number;
  Draw: number;
  Black: number;
  avgElo: string | number;
  label?: string;
}

interface MasterStatsProps {
  fen: string;
}

export default function MasterStats({ fen }: MasterStatsProps) {
  const [masterStatsData, setMasterStatsData] = useState<MastersApiResponse | undefined>(undefined);
  const [isLoading, setIsLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  const fetchMasterData = useCallback(async () => {
    if (fen) {
      setIsLoading(true);
      setError(null);
      try {
        const masterData = await master_moves(fen);
        setMasterStatsData(masterData);
      } catch (error) {
        console.error("Error fetching master data:", error);
        setError("Failed to load master game data");
      } finally {
        setIsLoading(false);
      }
    }
  }, [fen]);

  useEffect(() => {
    fetchMasterData();
  }, [fetchMasterData]);

  const processChartData = useCallback(
    (data: MastersApiResponse | undefined): ChartDataItem[] => {
      if (!data) return [];

      const { white, draws, black, moves, total = white + draws + black } = data;

      if (total === 0) return [];

      const currentPositionData: ChartDataItem = {
        name: "Current Position",
        White: (white / total) * 100,
        Draw: (draws / total) * 100,
        Black: (black / total) * 100,
        avgElo: "-",
      };

      // Process move data
      const movesData = moves.map((move) => {
        const moveTotal = move.white + move.draws + move.black;
        return {
          name: move.san,
          White: moveTotal > 0 ? (move.white / moveTotal) * 100 : 0,
          Draw: moveTotal > 0 ? (move.draws / moveTotal) * 100 : 0,
          Black: moveTotal > 0 ? (move.black / moveTotal) * 100 : 0,
          avgElo: move.averageRating || "-",
          label: `${move.san} (${moveTotal} games, avg elo ${move.averageRating || "?"})`,
        };
      });

      return [currentPositionData, ...movesData];
    },
    []
  );

  const masterChartData = useMemo(
    () => processChartData(masterStatsData),
    [masterStatsData, processChartData]
  );

  return (
    <>
      {isLoading ? (
        <div className="flex h-[400px] w-full items-center justify-center">
          <span className="text-lg">Loading master data...</span>
        </div>
      ) : error ? (
        <div className="flex h-[400px] w-full items-center justify-center">
          <span className="text-red-500">{error}</span>
        </div>
      ) : masterStatsData && masterChartData.length > 0 ? (
        <div>
          <div className="mb-4 text-sm text-gray-600">
            Based on {masterStatsData.total} master games
          </div>
          <StatsChart chartData={masterChartData} />
        </div>
      ) : (
        <div className="flex h-[400px] w-full items-center justify-center">
          <span className="text-lg">No master games found for this position</span>
        </div>
      )}
    </>
  );
}
