// app/api/test/route.ts
import { NextResponse } from "next/server";
import {
  compressPosition,
  decompressPosition,
} from "@/lib/server_chess_compression";

export async function GET() {
  try {
    const startingFen =
      "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

    const compressed = compressPosition(startingFen);
    if (!compressed) {
      return NextResponse.json(
        { error: "Compression failed" },
        { status: 500 },
      );
    }

    const decompressed = decompressPosition(compressed);
    if (!decompressed) {
      return NextResponse.json(
        { error: "Decompression failed" },
        { status: 500 },
      );
    }

    return NextResponse.json({
      original: startingFen,
      compressed,
      decompressed,
      success: startingFen === decompressed,
    });
  } catch (error) {
    console.error("Test failed:", error);
    return NextResponse.json({ error: "Test failed" }, { status: 500 });
  }
}
