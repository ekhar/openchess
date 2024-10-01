#!/bin/bash

# Create the output directory if it doesn't exist
mkdir -p ./twic_unzipped

# Unzip all files from twic_zip to twic_unzipped
for zip_file in ./twic_zip/*.zip; do
	unzip -o "$zip_file" -d ./twic_unzipped
done

# Create or clear the all_twic.pgn file
>./all_twic.pgn

# Append all PGN files from twic_unzipped to all_twic.pgn
for pgn_file in ./twic_unzipped/*.pgn; do
	cat "$pgn_file" >>./all_twic.pgn
	# Add a newline between files to ensure they don't run together
	echo "" >>./all_twic.pgn
done

echo "All operations completed successfully!"
