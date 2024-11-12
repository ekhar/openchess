"use client";
import { compressPosition } from "@/lib/chess_compression";
import { useEffect, useState } from "react";

export default function TestPage() {
  const [fen, setFen] = useState(
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
  );
  const [compressed, setCompressed] = useState<Uint8Array | null>(null);

  useEffect(() => {
    async function runTest() {
      try {
        const compressed = await compressPosition(fen);
        console.log("Compressed:", compressed);
        setCompressed(compressed);
      } catch (err) {
        console.error("Test failed:", err);
      }
    }

    runTest().catch((err) => {
      console.error("Error running test:", err);
    });
  }, [fen]);

  return (
    <div>
      <h1>Test Page</h1>
      <p>FEN: {fen}</p>
      <p>Compressed: {compressed ? compressed.length : 0} bytes</p>
    </div>
  );
}
