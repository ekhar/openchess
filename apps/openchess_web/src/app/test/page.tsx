// filename: apps/openchess_web/src/app/test/page.tsx
"use client";
import { useState, useEffect } from "react";
import Board from "@/components/Board";
import ChessUsername from "@/components/ChessUsername";
import {
  compressPosition,
  decompressPosition,
} from "@/lib/client_chess_compression";

export default function HomePage() {
  const [testResult, setTestResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    async function runTest() {
      try {
        const startingFen =
          "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
        console.log("Starting FEN:", startingFen);

        const compressed = await compressPosition(startingFen);
        console.log("Compressed:", compressed);

        if (compressed) {
          const decompressed = await decompressPosition(compressed);
          console.log("Decompressed:", decompressed);
          setTestResult(decompressed || "Decompression failed");
        } else {
          setError("Compression failed");
        }
      } catch (err) {
        console.error("Test failed:", err);
        setError(err instanceof Error ? err.message : String(err));
      } finally {
        setIsLoading(false);
      }
    }

    runTest();
  }, []);

  return (
    <main className="flex min-h-screen flex-col items-center justify-center bg-linear-to-b">
      <h1 className="mb-8 text-center text-4xl font-bold">OpenChess AI</h1>
      {/* WASM Test Result */}
      <div className="mb-8 text-center">
        <h2 className="text-2xl font-bold">WASM Test Result:</h2>
        {error ? (
          <p className="text-red-500">{error}</p>
        ) : isLoading ? (
          <p>Running test...</p>
        ) : testResult ? (
          <p className="text-green-500">{testResult}</p>
        ) : null}
      </div>
      <div className="flex flex-row items-center justify-center space-y-4 md:flex-row md:space-x-8 md:space-y-0">
        <Board />
        <ChessUsername />
      </div>
      <div className="mx-auto flex w-full max-w-7xl flex-col items-center justify-center px-4 sm:px-6 lg:px-8"></div>
    </main>
  );
}
