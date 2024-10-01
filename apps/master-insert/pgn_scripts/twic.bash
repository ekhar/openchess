#!/bin/bash

# Create a directory called twic_zip if it doesn't exist
mkdir -p twic_zip

# Define the start and end range for X
start=920
end=1559

# Loop through the range and download each file using wget into the twic_zip folder
for ((X = start; X <= end; X++)); do
	url="https://theweekinchess.com/zips/twic${X}g.zip"
	echo "Downloading $url into twic_zip"
	wget -P twic_zip $url
done
