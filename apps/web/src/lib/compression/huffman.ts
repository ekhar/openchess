// src/huffmanCode.ts

interface FrequencyPair {
  symbol: number;
  frequency: number;
}

class HuffmanNode {
  symbol: number | null;
  frequency: number;
  left: HuffmanNode | null;
  right: HuffmanNode | null;

  constructor(symbol: number | null, frequency: number) {
    this.symbol = symbol;
    this.frequency = frequency;
    this.left = null;
    this.right = null;
  }
}

class HuffmanCode {
  private tree: HuffmanNode | null = null;
  private codeBook = new Map<number, string>();

  constructor(private frequencies: FrequencyPair[]) {
    this.buildTree();
    this.generateCodeBook();
  }

  private buildTree(): void {
    const queue = this.frequencies.map(
      (pair) => new HuffmanNode(pair.symbol, pair.frequency),
    );

    while (queue.length > 1) {
      queue.sort((a, b) => a.frequency - b.frequency);
      const left = queue.shift();
      const right = queue.shift();

      if (left && right) {
        const parent = new HuffmanNode(null, left.frequency + right.frequency);
        parent.left = left;
        parent.right = right;
        queue.push(parent);
      }
    }

    this.tree = queue.length > 0 ? queue[0] : null;
  }

  private generateCodeBook(): void {
    const traverse = (node: HuffmanNode | null, code = ""): void => {
      if (!node) return;

      if (node.symbol !== null) {
        this.codeBook.set(node.symbol, code);
      }

      traverse(node.left, code + "0");
      traverse(node.right, code + "1");
    };

    traverse(this.tree);
  }

  encode(symbol: number): string | undefined {
    return this.codeBook.get(symbol);
  }

  decode(encodedString: string): number | null {
    let node = this.tree;
    for (const bit of encodedString) {
      if (!node) return null;
      node = bit === "0" ? node.left : node.right;
    }
    return node?.symbol ?? null;
  }
}

// Define frequencies as a static array
const FREQUENCIES: FrequencyPair[] = [
  { symbol: 0, frequency: 225883932 },
  { symbol: 1, frequency: 134956126 },
  // ... (include all frequency pairs here)
  { symbol: 255, frequency: 1 },
];

// Singleton pattern for HuffmanCode instance
let huffmanCodeInstance: HuffmanCode | null = null;

function getHuffmanCode(): HuffmanCode {
  if (!huffmanCodeInstance) {
    huffmanCodeInstance = new HuffmanCode(FREQUENCIES);
  }
  return huffmanCodeInstance;
}

export { getHuffmanCode, HuffmanCode, type FrequencyPair };
