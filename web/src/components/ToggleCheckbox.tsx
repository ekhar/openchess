"use client";

import { useState } from "react";
import { Checkbox } from "@/components/ui/checkbox";
import { Label } from "@/components/ui/label";

export default function ToggleCheckbox() {
  const [selectedOption, setSelectedOption] = useState<string>("all");

  const handleOptionChange = (option: string) => {
    setSelectedOption(option);
  };

  const options = [
    { id: "all", label: "All" },
    { id: "tournament", label: "Tournament Play" },
    { id: "blitz", label: "Blitz" },
  ];

  return (
    <div className="space-y-4">
      <h2 className="text-lg font-semibold">Filter by Game Type:</h2>
      <div className="space-y-2">
        {options.map((option) => (
          <div key={option.id} className="flex items-center space-x-2">
            <Checkbox
              id={option.id}
              checked={selectedOption === option.id}
              onCheckedChange={() => handleOptionChange(option.id)}
            />
            <Label
              htmlFor={option.id}
              className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
            >
              {option.label}
            </Label>
          </div>
        ))}
      </div>
    </div>
  );
}
