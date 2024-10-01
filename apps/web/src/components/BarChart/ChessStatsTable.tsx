"use client";
import React from "react";
import { useBoardContext } from "@/context/UsernameContext";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import MasterStats from "@/components/BarChart/MasterStats";
import PlayerStats from "@/components/BarChart/PlayerStats";

export default function ChessStatsTable() {
  const { fen, username } = useBoardContext();

  return (
    <Card className="w-full">
      <CardHeader>
        <CardTitle>Chess Statistics</CardTitle>
      </CardHeader>
      <CardContent>
        <Tabs defaultValue="master">
          <TabsList>
            <TabsTrigger value="master">Master Moves</TabsTrigger>
            <TabsTrigger value="player">Player Moves</TabsTrigger>
          </TabsList>
          <div className="mt-4 grid grid-cols-1 gap-4 md:grid-cols-3">
            <div className="md:col-span-2">
              <TabsContent value="master">
                <MasterStats fen={fen} />
              </TabsContent>
              <TabsContent value="player">
                <PlayerStats fen={fen} username={username} />
              </TabsContent>
            </div>
          </div>
        </Tabs>
      </CardContent>
    </Card>
  );
}
