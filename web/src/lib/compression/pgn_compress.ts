// src/compression/pgnCompress.ts

import { Chess, type Move, type Square, type Color } from "chess.js";

class EncoderError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "EncoderError";
  }
}

class ScoredMove {
  mv: Move;
  score: number;

  constructor(board: Chess, mv: Move) {
    this.mv = mv;
    this.score = this.computeScore(board, mv);
  }

  private computeScore(board: Chess, mv: Move): number {
    const us = board.turn();
    const them = us === "w" ? "b" : "w";
    const fromSquare = mv.from;
    const toSquare = mv.to;

    let score = 0;

    if (mv.promotion) {
      score += this.getPieceValue(mv.promotion) << 26;
    }

    if (mv.captured) {
      score += 1 << 25;
    }

    const defendingPawns = this.getPawnAttacks(them, toSquare, board);
    const defendingPawnsScore =
      defendingPawns.length === 0 ? 6 : 5 - this.getPieceValue(mv.piece);
    score += defendingPawnsScore << 22;

    const moveValue =
      this.getPieceSquareValue(mv.piece, toSquare) -
      this.getPieceSquareValue(mv.piece, fromSquare);
    score += (512 + moveValue) << 12;

    score += this.squareToIndex(toSquare) << 6;
    score += this.squareToIndex(fromSquare);

    return score;
  }

  private getPieceValue(piece: string): number {
    const values: Record<string, number> = {
      p: 0,
      n: 1,
      b: 2,
      r: 3,
      q: 4,
      k: 5,
    };
    return values[piece.toLowerCase()];
  }

  private getPieceSquareValue(piece: string, square: Square): number {
    // Simplified piece-square table, you may want to use more sophisticated values
    const index = this.squareToIndex(square);
    const centerBonus =
      Math.abs(3.5 - (index % 8)) + Math.abs(3.5 - Math.floor(index / 8));
    return 10 - centerBonus;
  }

  private getPawnAttacks(color: Color, square: Square, board: Chess): Square[] {
    const file = square.charAt(0);
    const rank = parseInt(square.charAt(1));
    const attackingSquares: Square[] = [];

    if (color === "w") {
      if (file !== "a")
        attackingSquares.push(
          `${String.fromCharCode(file.charCodeAt(0) - 1)}${rank - 1}` as Square,
        );
      if (file !== "h")
        attackingSquares.push(
          `${String.fromCharCode(file.charCodeAt(0) + 1)}${rank - 1}` as Square,
        );
    } else {
      if (file !== "a")
        attackingSquares.push(
          `${String.fromCharCode(file.charCodeAt(0) - 1)}${rank + 1}` as Square,
        );
      if (file !== "h")
        attackingSquares.push(
          `${String.fromCharCode(file.charCodeAt(0) + 1)}${rank + 1}` as Square,
        );
    }

    return attackingSquares.filter((sq) => {
      const piece = board.get(sq);
      return piece && piece.type === "p" && piece.color === color;
    });
  }

  private squareToIndex(square: Square): number {
    const file = square.charCodeAt(0) - "a".charCodeAt(0);
    const rank = 8 - parseInt(square.charAt(1));
    return rank * 8 + file;
  }
}

export class Encoder {
  private board: Chess;
  private buffer: number[];
  private bitPosition: number;

  constructor() {
    this.board = new Chess();
    this.buffer = [0];
    this.bitPosition = 0;
  }

  public encodeMove(pgnMove: string): void {
    const move = this.board.move(pgnMove);
    if (!move) {
      throw new EncoderError(`Invalid move: ${pgnMove}`);
    }

    const legalMoves = this.board.moves({ verbose: true });
    const scoredMoves = legalMoves.map((m) => new ScoredMove(this.board, m));
    scoredMoves.sort((a, b) => b.score - a.score);

    const index = scoredMoves.findIndex((sm) => sm.mv.san === move.san);
    if (index === -1) {
      throw new EncoderError("Move not found in scored moves");
    }

    this.encodeBits(index, this.calculateBitsNeeded(legalMoves.length));
  }

  private calculateBitsNeeded(moveCount: number): number {
    return Math.ceil(Math.log2(moveCount));
  }

  private encodeBits(value: number, bits: number): void {
    for (let i = bits - 1; i >= 0; i--) {
      const bit = (value >> i) & 1;
      if (bit) {
        this.buffer[this.buffer.length - 1] |= 1 << (7 - this.bitPosition);
      }
      this.bitPosition++;
      if (this.bitPosition === 8) {
        this.buffer.push(0);
        this.bitPosition = 0;
      }
    }
  }

  public finalize(): Uint8Array {
    return new Uint8Array(this.buffer);
  }

  public decode(data: Uint8Array): string[] {
    const output: string[] = [];
    const board = new Chess();
    let byteIndex = 0;
    let bitIndex = 0;

    while (byteIndex < data.length) {
      const legalMoves = board.moves({ verbose: true });
      const scoredMoves = legalMoves.map((m) => new ScoredMove(board, m));
      scoredMoves.sort((a, b) => b.score - a.score);

      const bitsNeeded = this.calculateBitsNeeded(legalMoves.length);
      let moveIndex = 0;

      for (let i = 0; i < bitsNeeded; i++) {
        if (byteIndex >= data.length) break;
        const bit = (data[byteIndex] >> (7 - bitIndex)) & 1;
        moveIndex = (moveIndex << 1) | bit;
        bitIndex++;
        if (bitIndex === 8) {
          byteIndex++;
          bitIndex = 0;
        }
      }

      if (moveIndex >= scoredMoves.length) {
        throw new EncoderError("Invalid move index during decoding");
      }

      const move = scoredMoves[moveIndex].mv;
      board.move(move);
      output.push(move.san);

      if (board.isGameOver()) break;
    }

    return output;
  }
}

// Test function
function runTests(): void {
  const pgnMoves = [
    "e4",
    "c5",
    "Nf3",
    "d6",
    "Bb5+",
    "Bd7",
    "Bxd7+",
    "Nxd7",
    "O-O",
    "Ngf6",
    "Re1",
    "e6",
  ];

  const encoder = new Encoder();
  for (const move of pgnMoves) {
    encoder.encodeMove(move);
  }
  const compressed = encoder.finalize();
  const decodedMoves = encoder.decode(compressed);

  console.assert(
    JSON.stringify(pgnMoves) === JSON.stringify(decodedMoves),
    "Test failed: Decoded moves do not match original moves",
  );
  console.log("All tests passed!");

  // Print compression statistics
  console.log("Original size:", pgnMoves.join(" ").length, "bytes");
  console.log("Compressed size:", compressed.length, "bytes");
  console.log(
    "Compression ratio:",
    pgnMoves.join(" ").length / compressed.length,
  );
}

// Run tests
runTests();
