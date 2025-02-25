// filename: apps/openchess_web/src/app/page.tsx
import Board from "@/components/Board";
import ChessUsername from "@/components/ChessUsername";
import ChessStatsTable from "@/components/BarChart/ChessStatsTable";

export default function HomePage() {
  return (
    <main className="flex min-h-screen flex-col items-center justify-center bg-gradient-to-b">
      <h1 className="mb-8 text-center text-4xl font-bold">Database Explorer</h1>
      <div className="flex flex-row items-center justify-center space-y-4 md:flex-row md:space-x-8 md:space-y-0">
        <Board />
        <ChessUsername />
      </div>

      <div className="mx-auto flex w-full max-w-7xl flex-col items-center justify-center px-4 sm:px-6 lg:px-8">
        <ChessStatsTable />
      </div>
    </main>
  );
}
