import React from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import Link from "next/link";

export default function AboutPage() {
  return (
    <div className="container mx-auto py-8">
      <h1 className="mb-6 text-4xl font-bold">About OpenChess AI</h1>

      <Card className="mb-8">
        <CardHeader>
          <CardTitle>Our Mission</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="mb-4">
            OpenChess AI is a tool designed to help chess players of all levels
            study openings, identify weaknesses, and improve their game. By
            combining the open-source database from Lichess with games from
            Chess.com, we hope to offer a comprehensive platform for chess
            analysis and improvement.
          </p>
          <p className="mb-4">Our unique features allow you to:</p>
          <ul className="mb-4 list-inside list-disc">
            <li>Study chess openings in depth</li>
            <li>Analyze your own games to find areas for improvement</li>
            <li>Investigate potential opponents' strengths and weaknesses</li>
            <li>
              Visualize your progress and game statistics through intuitive
              graphs
            </li>
          </ul>
          <p>
            Whether you're a beginner looking to improve or an advanced player
            aiming for mastery, OpenChess AI provides the insights and tools you
            need to take your chess game to the next level.
          </p>
        </CardContent>
      </Card>

      <Card className="mb-8">
        <CardHeader>
          <CardTitle>About the Creator</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="mb-4">
            Hey there! I'm Eric, and I'm pretty passionate about chess and
            coding. I started playing when I was about 16 on chess.com and
            aspire one day to be fide master. Any tips or recommendations on the
            site are strongly appriciated :) My highest elo was 2081 on rapid.
            Hit me on chess.com ekhar02 and let's play a game! Share the site
            with friends or or your local club - I want this thing to be cool!
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Get Involved</CardTitle>
        </CardHeader>
        <CardContent>
          <p className="mb-4">
            We're always looking to improve OpenChess AI and add new features
            that benefit our users. If you have ideas, suggestions, or feedback,
            we'd love to hear from you!
          </p>
          <Button asChild>
            <Link href="mailto:ericsrealemail@gmail.com">Contact Us</Link>
          </Button>
        </CardContent>
      </Card>
    </div>
  );
}
