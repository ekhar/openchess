#!/bin/bash

input_file="./src/pgns/LUMBRAS_CLEAN_FILTERED.pgn"
output_file="first_6_million_events.pgn"
remaining_file="remaining_events.pgn"
target_count=6000000

# Count the total number of events
total_events=$(grep -F "[Event " -c "$input_file")
echo "Total events in the file: $total_events"

# Use awk to split the file
awk -v target=$target_count '
    /\[Event / {count++}
    {
        if (count <= target) {
            print > "'"$output_file"'"
        } else {
            print > "'"$remaining_file"'"
        }
    }
' "$input_file"

echo "Split complete. First $target_count events saved to $output_file"
echo "Remaining events saved to $remaining_file"
