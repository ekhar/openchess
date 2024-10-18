// app/api/create-game/route.ts
import { type NextRequest, NextResponse } from "next/server";
import { createClient } from "@/utils/supabase/supabaseServer";

export async function POST(request: NextRequest) {
  const { gameId } = await request.json();

  // Insert a new game into the 'games' table
  const { data, error } = await createClient()
    .from("live_games")
    .insert({ id: gameId });

  if (error) {
    return NextResponse.json({ error: error.message }, { status: 500 });
  }

  return NextResponse.json({ data });
}
