import chess.pgn
import argparse


def extract_first_n_pgns(input_file, output_file, n):
    """
    Extract the first n PGN games from the input file and write them to the output file.

    Args:
    input_file (str): The path to the input PGN file.
    output_file (str): The path to the output PGN file.
    n (int): The number of games to extract.

    Returns:
    int: The number of games actually extracted.
    """
    games_extracted = 0

    with open(input_file, "r") as pgn_input, open(output_file, "w") as pgn_output:
        for _ in range(n):
            game = chess.pgn.read_game(pgn_input)
            if game is None:
                break

            exporter = chess.pgn.FileExporter(pgn_output)
            game.accept(exporter)

            # Add a blank line between games as per PGN standard
            pgn_output.write("\n")

            games_extracted += 1

    return games_extracted


def main():
    parser = argparse.ArgumentParser(
        description="Extract the first N PGN games from a file."
    )
    parser.add_argument("input_file", help="The path to the input PGN file.")
    parser.add_argument(
        "--output",
        default="testing.pgn",
        help="The path to the output PGN file. Default is 'testing.pgn'.",
    )
    parser.add_argument(
        "--count",
        type=int,
        default=200,
        help="The number of games to extract. Default is 200.",
    )
    args = parser.parse_args()

    try:
        games_extracted = extract_first_n_pgns(args.input_file, args.output, args.count)
        print(f"Successfully extracted {games_extracted} games to {args.output}")
    except FileNotFoundError:
        print(f"Error: The file {args.input_file} was not found.")
    except Exception as e:
        print(f"An error occurred: {str(e)}")


if __name__ == "__main__":
    main()
