import React from "react";
import { RadioGroup, RadioGroupItem } from "@/components/ui/radio-group";
import { Label } from "@/components/ui/label";
import { Checkbox } from "@/components/ui/checkbox";

interface PlayerControlsProps {
  username: string;
  playerColor: "white" | "black";
  setPlayerColor: (color: "white" | "black") => void;
  selectedSpeeds: string[];
  setSelectedSpeeds: (speeds: string[]) => void;
  selectedModes: string[];
  setSelectedModes: (modes: string[]) => void;
}

const speedOptions = [
  "ultraBullet",
  "bullet",
  "blitz",
  "rapid",
  "classical",
  "correspondence",
];
const modeOptions = ["casual", "rated"];

export default function PlayerControls({
  username,
  playerColor,
  setPlayerColor,
  selectedSpeeds,
  setSelectedSpeeds,
  selectedModes,
  setSelectedModes,
}: PlayerControlsProps) {
  return (
    <div className="space-y-4">
      <div>
        <h3 className="mb-2 text-lg font-semibold">
          {username} chess.com stats
        </h3>
        <h4 className="mb-2 font-medium">Player Color</h4>
        <RadioGroup
          value={playerColor}
          onValueChange={(value) => setPlayerColor(value as "white" | "black")}
          className="flex space-x-4"
        >
          <div className="flex items-center space-x-2">
            <RadioGroupItem value="white" id="white" />
            <Label htmlFor="white">White</Label>
          </div>
          <div className="flex items-center space-x-2">
            <RadioGroupItem value="black" id="black" />
            <Label htmlFor="black">Black</Label>
          </div>
        </RadioGroup>
      </div>
      <div>
        <h4 className="mb-2 font-medium">Speed Controls</h4>
        <div className="flex flex-wrap gap-2">
          {speedOptions.map((speed) => (
            <label key={speed} className="flex items-center">
              <Checkbox
                checked={selectedSpeeds.includes(speed)}
                onCheckedChange={(checked) => {
                  if (checked) {
                    setSelectedSpeeds([...selectedSpeeds, speed]);
                  } else {
                    setSelectedSpeeds(
                      selectedSpeeds.filter((s) => s !== speed),
                    );
                  }
                }}
              />
              <span className="ml-2">{speed}</span>
            </label>
          ))}
        </div>
      </div>
      <div>
        <h4 className="mb-2 font-medium">Game Types</h4>
        <div className="flex flex-wrap gap-2">
          {modeOptions.map((mode) => (
            <label key={mode} className="flex items-center">
              <Checkbox
                checked={selectedModes.includes(mode)}
                onCheckedChange={(checked) => {
                  if (checked) {
                    setSelectedModes([...selectedModes, mode]);
                  } else {
                    setSelectedModes(selectedModes.filter((m) => m !== mode));
                  }
                }}
              />
              <span className="ml-2">{mode}</span>
            </label>
          ))}
        </div>
      </div>
    </div>
  );
}
