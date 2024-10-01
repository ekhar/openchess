// src/lib/positions.ts

import { supabase } from "@/utils/supabaseClient";
import { type Position } from "@/types/interfaces";

export const insertPosition = async (
  compressed_fen: Uint8Array,
): Promise<Position | null> => {
  const { data, error } = await supabase
    .from("positions")
    .insert([{ compressed_fen }])
    .single();

  if (error) {
    console.error("Error inserting position:", error.message);
    return null;
  }

  return data;
};
