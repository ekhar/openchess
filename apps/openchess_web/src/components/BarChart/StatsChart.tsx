import React from "react";
import {
  BarChart,
  Bar,
  XAxis,
  YAxis,
  Tooltip as RechartsTooltip,
  Legend,
  ResponsiveContainer,
} from "recharts";
import { type ChartDataItem } from "@/types/lichess-api";

interface StatsChartProps {
  chartData: ChartDataItem[];
}

export default function StatsChart({ chartData }: StatsChartProps) {
  console.log("StatsChart rendering with data:", chartData);
  if (!chartData || chartData.length === 0) {
    return <div>No data available</div>;
  }
  return (
    <div className="min-h-[500px] w-full">
      <ResponsiveContainer width="100%" height={500}>
        <BarChart
          id="stats-chart"
          data={chartData}
          layout="vertical"
          margin={{ top: 20, right: 30, left: 20, bottom: 60 }}
        >
          <XAxis
            type="number"
            domain={[0, 100]}
            tickFormatter={(value) => `${value}%`}
            height={50}
            label={{ value: "Percentage", position: "bottom", offset: 0 }}
          />
          <YAxis
            dataKey="label"
            type="category"
            width={150}
            tick={{ fontSize: 12 }}
          />
          <RechartsTooltip />
          <Legend verticalAlign="top" height={36} />
          <Bar dataKey="White" stackId="a" fill="#8884d8" />
          <Bar dataKey="Draw" stackId="a" fill="#82ca9d" />
          <Bar dataKey="Black" stackId="a" fill="#ffc658" />
        </BarChart>
      </ResponsiveContainer>
    </div>
  );
}
