// filename: apps/openchess_web/src/components/StockfishAnalysisBar.tsx
'use client';
// components/StockfishAnalysisBar.jsx
import React, { useState, useEffect, useRef } from 'react';
import styles from './StockfishAnalysisBar.module.css';

const StockfishAnalysisBar = ({ fen, onMoveSelect }) => {
  const [engine, setEngine] = useState(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [evaluation, setEvaluation] = useState(0);
  const [depth, setDepth] = useState(15);
  const [topMoves, setTopMoves] = useState([]);
  const [isEngineReady, setIsEngineReady] = useState(false);
  const [error, setError] = useState(null);

  const engineRef = useRef(null);
  const analyzeTimeoutRef = useRef(null);

  // Helper function to convert evaluation to display format
  const formatEvaluation = (evalScore) => {
    if (typeof evalScore === 'string' && evalScore.startsWith('mate')) {
      return evalScore;
    }
    return (evalScore / 100).toFixed(2);
  };

  // Initialize Stockfish
  useEffect(() => {
    const initEngine = async () => {
      try {
        // Check if running in browser
        if (typeof window === 'undefined') return;

        // Check for WebAssembly threads support
        const wasmThreadsSupported = () => {
          // WebAssembly 1.0
          const source = Uint8Array.of(0x0, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00);
          if (
            typeof WebAssembly !== "object" ||
            typeof WebAssembly.validate !== "function"
          )
            return false;
          if (!WebAssembly.validate(source)) return false;

          // SharedArrayBuffer
          if (typeof SharedArrayBuffer !== "function") return false;

          // Atomics
          if (typeof Atomics !== "object") return false;

          // Shared memory
          try {
            const mem = new WebAssembly.Memory({ shared: true, initial: 8, maximum: 16 });
            if (!(mem.buffer instanceof SharedArrayBuffer)) return false;

            // Structured cloning
            try {
              // You have to make sure nobody cares about these messages!
              window.postMessage(mem, "*");
            } catch (e) {
              return false;
            }

            // Growable shared memory (optional)
            try {
              mem.grow(8);
            } catch (e) {
              return false;
            }

            return true;
          } catch (e) {
            return false;
          }
        };

        // Check for WASM threads support
        if (!wasmThreadsSupported()) {
          throw new Error("WebAssembly threads not supported in this browser. Try using Chrome or Firefox.");
        }

        // Dynamically import Stockfish
        const Stockfish = (await import('public/stockfish.wasm')).default;
        const sf = await Stockfish();

        engineRef.current = sf;
        setEngine(sf);

        // Configure message listener
        sf.addMessageListener((message) => {
          // Handle "readyok" response
          if (message === 'readyok') {
            setIsEngineReady(true);
            return;
          }

          // Handle bestmove response (analysis complete)
          if (message.startsWith('bestmove')) {
            setIsAnalyzing(false);
            return;
          }

          // Handle info string with evaluation
          if (message.startsWith('info') && message.includes('score')) {
            try {
              // Extract depth
              const depthMatch = message.match(/depth (\d+)/);
              const currentDepth = depthMatch ? parseInt(depthMatch[1]) : 0;

              // Only process if this is for our target depth or final result
              if (message.includes(`depth ${depth}`) || message.includes('bestmove')) {
                // Extract score
                let evalScore;
                if (message.includes('score cp')) {
                  const scoreMatch = message.match(/score cp ([-\d]+)/);
                  evalScore = scoreMatch ? parseInt(scoreMatch[1]) : 0;
                } else if (message.includes('score mate')) {
                  const mateMatch = message.match(/score mate ([-\d]+)/);
                  const mateIn = mateMatch ? parseInt(mateMatch[1]) : 0;
                  evalScore = `mate ${mateIn}`;
                }

                // Update evaluation if we got a valid score
                if (evalScore !== undefined) {
                  setEvaluation(evalScore);
                }

                // Extract PV (principal variation - the line of best moves)
                if (message.includes(' pv ')) {
                  const pvMatch = message.match(/ pv (.+?)( bmc| score| depth|$)/);
                  const movesString = pvMatch ? pvMatch[1] : '';
                  const moves = movesString.trim().split(' ');

                  // For multipv, get the multipv number
                  let multipvIndex = 0;
                  if (message.includes('multipv')) {
                    const multipvMatch = message.match(/multipv (\d+)/);
                    multipvIndex = multipvMatch ? parseInt(multipvMatch[1]) - 1 : 0;
                  }

                  // Update the appropriate move in the top moves array
                  setTopMoves(prevMoves => {
                    const newMoves = [...prevMoves];
                    newMoves[multipvIndex] = {
                      line: moves,
                      evaluation: evalScore,
                      depth: currentDepth
                    };
                    return newMoves;
                  });
                }
              }
            } catch (err) {
              console.error('Error parsing engine output:', err);
            }
          }
        });

        // Initialize engine with UCI commands
        sf.postMessage('uci');
        sf.postMessage('isready');

        // Set MultiPV to 3 (top 3 moves)
        sf.postMessage('setoption name MultiPV value 3');
      } catch (err) {
        console.error('Failed to initialize Stockfish:', err);
        setError(`Failed to initialize Stockfish: ${err.message}`);
      }
    };

    initEngine();

    // Cleanup function
    return () => {
      if (engineRef.current) {
        engineRef.current.postMessage('quit');
      }
      if (analyzeTimeoutRef.current) {
        clearTimeout(analyzeTimeoutRef.current);
      }
    };
  }, []);

  // Analyze position when FEN changes or depth changes
  useEffect(() => {
    const analyze = () => {
      if (!engineRef.current || !isEngineReady || !fen) return;

      // Clear any pending analysis
      if (analyzeTimeoutRef.current) {
        clearTimeout(analyzeTimeoutRef.current);
      }

      // Start new analysis after a small delay to prevent rapid successive calls
      analyzeTimeoutRef.current = setTimeout(() => {
        setIsAnalyzing(true);
        setTopMoves([]);

        engineRef.current.postMessage('stop'); // Stop any ongoing analysis
        engineRef.current.postMessage(`position fen ${fen}`);
        engineRef.current.postMessage(`go depth ${depth}`);
      }, 100);
    };

    analyze();
  }, [fen, depth, isEngineReady]);

  // Update depth when user changes it
  const handleDepthChange = (event) => {
    const newDepth = parseInt(event.target.value);
    setDepth(newDepth);
  };

  // Handle move selection
  const handleMoveSelect = (move) => {
    if (onMoveSelect && move.line && move.line.length > 0) {
      onMoveSelect(move.line[0]);
    }
  };

  // Calculate evaluation bar height percentage
  const calculateEvalBarHeight = () => {
    if (typeof evaluation === 'string' && evaluation.startsWith('mate')) {
      // If it's checkmate, set to max or min
      return evaluation.includes('-') ? 0 : 100;
    }

    // Convert centipawns to a percentage (sigmoid function to keep within bounds)
    const rawEval = typeof evaluation === 'number' ? evaluation : 0;
    const sigmoid = (x) => 100 / (1 + Math.exp(-0.0075 * x));
    return sigmoid(rawEval);
  };

  if (error) {
    return <div className={styles.error}>{error}</div>;
  }

  return (
    <div className={styles.analysisBar}>
      <div className={styles.evalBarContainer}>
        <div
          className={styles.evalBar}
          style={{
            height: `${calculateEvalBarHeight()}%`,
            backgroundColor: evaluation < 0 ? '#000' : '#fff',
          }}
        />
        <div className={styles.evalText}>
          {typeof evaluation === 'string'
            ? evaluation
            : (evaluation > 0 ? '+' : '') + formatEvaluation(evaluation)}
        </div>
      </div>

      <div className={styles.controls}>
        <div className={styles.depthControl}>
          <label htmlFor="depth-select">Depth:</label>
          <select
            id="depth-select"
            value={depth}
            onChange={handleDepthChange}
            disabled={isAnalyzing}
          >
            {[5, 10, 15, 20, 25, 30].map(d => (
              <option key={d} value={d}>{d}</option>
            ))}
          </select>
        </div>

        <div className={styles.status}>
          {isAnalyzing ? 'Analyzing...' : 'Ready'}
        </div>
      </div>

      <div className={styles.topMoves}>
        <h3>Top Moves</h3>
        {topMoves.length === 0 && isAnalyzing && (
          <div className={styles.loading}>Calculating best moves...</div>
        )}
        <ul>
          {topMoves.map((move, index) => (
            move ? (
              <li
                key={index}
                className={styles.moveOption}
                onClick={() => handleMoveSelect(move)}
              >
                <span className={styles.moveRank}>{index + 1}.</span>
                <span className={styles.moveNotation}>{move.line?.[0]}</span>
                <span className={styles.moveEval}>
                  {typeof move.evaluation === 'string'
                    ? move.evaluation
                    : (move.evaluation > 0 ? '+' : '') + formatEvaluation(move.evaluation)}
                </span>
                <span className={styles.moveLine}>
                  {move.line?.slice(0, 5).join(' ')}...
                </span>
              </li>
            ) : null
          ))}
        </ul>
      </div>
    </div>
  );
};

export default StockfishAnalysisBar;
