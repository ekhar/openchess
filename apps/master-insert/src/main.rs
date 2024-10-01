// main.rs
use sqlx::PgPool;

use pgn_reader::{BufferedReader, RawHeader, SanPlus, Skip, Visitor};
use shakmaty::{Chess, Position};
use sqlx::types::chrono::NaiveDate;
use std::env;
use std::error::Error;
use std::fs::File;
mod enums;
use chess_compression::{CompressedPosition, Encoder, EncoderError};
use dotenv::dotenv;
use enums::*;
// Define a struct to represent a row in the games table
#[derive(Debug, Clone, Default, sqlx::FromRow)]
struct Game {
    eco: String,
    white_player: String,
    black_player: String,
    date: Option<NaiveDate>,
    result: ResultType,
    pgn_moves: Vec<String>,
    white_elo: i32,
    black_elo: i32,
    time_control: Option<ChessSpeed>,
}
impl Game {
    pub fn compress_pgn(&self) -> Result<Vec<u8>, EncoderError> {
        let mut encoder = Encoder::new();

        for move_str in &self.pgn_moves {
            encoder.encode_move(move_str)?;
        }

        let compressed = encoder.finalize();
        Ok(compressed.to_bytes())
    }
}

struct Importer {
    current_game: Game,
    skip: bool,
}
impl Importer {
    fn new() -> Self {
        Self {
            current_game: Game {
                eco: String::new(),
                white_player: String::new(),
                black_player: String::new(),
                date: None,
                result: ResultType::Draw, // Default to Draw for safety
                pgn_moves: Vec::new(),
                white_elo: 0,
                black_elo: 0,
                time_control: None,
            },
            skip: false,
        }
    }
}

impl Visitor for Importer {
    type Result = Game;

    fn begin_game(&mut self) {
        self.current_game = Game::default();
        self.skip = false;
    }

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        match key {
            b"White" => {
                self.current_game.white_player = value
                    .decode_utf8()
                    .map(|s| {
                        if s == "?" {
                            "Unknown".to_string()
                        } else {
                            s.to_string()
                        }
                    })
                    .unwrap_or_default();
            }
            b"Black" => {
                self.current_game.black_player = value
                    .decode_utf8()
                    .map(|s| {
                        if s == "?" {
                            "Unknown".to_string()
                        } else {
                            s.to_string()
                        }
                    })
                    .unwrap_or_default();
            }
            b"WhiteElo" => {
                self.current_game.white_elo = if value.as_bytes() != b"?" {
                    btoi::btoi(value.as_bytes()).unwrap_or(0)
                } else {
                    0
                };
            }
            b"BlackElo" => {
                self.current_game.black_elo = if value.as_bytes() != b"?" {
                    btoi::btoi(value.as_bytes()).unwrap_or(0)
                } else {
                    0
                };
            }
            b"Date" => {
                if let Ok(date_str) = value.decode_utf8() {
                    let processed_date_str = date_str
                        .replace(".??.", ".01.") // Replace month
                        .replace(".??", ".01"); // Replace day
                    if let Ok(date) = NaiveDate::parse_from_str(&processed_date_str, "%Y.%m.%d") {
                        self.current_game.date = Some(date);
                    } else {
                        // If parsing fails, you might want to handle partial dates
                        // For example, try parsing just the year
                        if let Ok(year) = processed_date_str
                            .split('.')
                            .next()
                            .unwrap_or("")
                            .parse::<i32>()
                        {
                            if let Some(date) = NaiveDate::from_ymd_opt(year, 1, 1) {
                                self.current_game.date = Some(date);
                            }
                        }
                    }
                }
            }
            b"Result" => {
                let result_str = value.decode_utf8().unwrap_or_default().into_owned();
                self.current_game.result = match result_str.as_str() {
                    "1-0" => ResultType::White,
                    "0-1" => ResultType::Black,
                    "1/2-1/2" => ResultType::Draw,
                    _ => {
                        self.skip = true;
                        ResultType::Draw
                    }
                };
            }
            b"ECO" => {
                self.current_game.eco = value.decode_utf8().unwrap_or_default().into_owned();
            }
            b"TimeControl" => {
                self.current_game.time_control = ChessSpeed::from_bytes(value.as_bytes()).ok();
            }
            _ => {}
        }
    }

    fn end_headers(&mut self) -> Skip {
        let d = NaiveDate::from_ymd_opt(2000, 1, 1).unwrap();
        if self.current_game.time_control.is_none()
            && self.current_game.date.unwrap_or_default() < d
        {
            self.current_game.time_control = Some(ChessSpeed::Classical);
        }
        self.skip |=
            self.current_game.white_player.is_empty() || self.current_game.black_player.is_empty();
        Skip(self.skip)
    }

    fn san(&mut self, san_plus: SanPlus) {
        self.current_game.pgn_moves.push(san_plus.to_string());
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true) // Skip variations
    }

    fn end_game(&mut self) -> Self::Result {
        if !self.skip {
            self.current_game.clone()
        } else {
            Game {
                eco: String::new(),
                white_player: String::new(),
                black_player: String::new(),
                date: None,
                result: ResultType::Draw,
                pgn_moves: Vec::new(),
                white_elo: 0,
                black_elo: 0,
                time_control: None,
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenv().ok();
    // Get the PGN file path from command-line arguments
    println!("Starting the importer...");
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <pgn_file>", args[0]);
        std::process::exit(1);
    }
    let file_path = &args[1];

    // Open the PGN file and create a BufferedReader
    let file = File::open(file_path)?;
    let mut reader = BufferedReader::new(file);

    // Connect to PostgreSQL
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    // Initialize variables for batching
    let batch_size = 5_000;
    let mut games_batch = Vec::with_capacity(batch_size);
    let mut games_processed = 0;
    let mut read_games: usize = 0;
    // Read and process games one at a time
    loop {
        let mut importer = Importer::new();

        read_games += 1;
        if read_games % 20_000 == 0 {
            println!("Read {} games", read_games);
        }
        match reader.read_game(&mut importer)? {
            Some(game) => {
                if !importer.skip {
                    games_batch.push(game);
                }

                if games_batch.len() >= batch_size {
                    // Process and insert the batch
                    process_and_insert_batch(&mut games_batch, &pool).await?;

                    games_processed += batch_size;
                    println!("Processed and inserted {} games", games_processed);
                }
            }
            None => {
                // No more games to read
                break;
            }
        }
    }

    // Process any remaining games in the batch
    if !games_batch.is_empty() {
        process_and_insert_batch(&mut games_batch, &pool).await?;

        games_processed += games_batch.len();
        println!("Processed and inserted {} games", games_processed);
    }

    println!("Import completed successfully.");
    Ok(())
}

async fn process_and_insert_batch(
    games_batch: &mut Vec<Game>,
    pool: &PgPool,
) -> Result<(), Box<dyn Error>> {
    // Start a new transaction
    let mut tx = pool.begin().await?;

    // Prepare vectors for bulk inserting into the 'games' table
    let mut eco_vec: Vec<String> = Vec::with_capacity(games_batch.len());
    let mut white_player_vec: Vec<String> = Vec::with_capacity(games_batch.len());
    let mut black_player_vec: Vec<String> = Vec::with_capacity(games_batch.len());
    let mut date_vec: Vec<Option<NaiveDate>> = Vec::with_capacity(games_batch.len());
    let mut result_vec: Vec<ResultType> = Vec::with_capacity(games_batch.len());
    let mut white_elo_vec: Vec<i32> = Vec::with_capacity(games_batch.len());
    let mut black_elo_vec: Vec<i32> = Vec::with_capacity(games_batch.len());
    let mut time_control_vec: Vec<ChessSpeed> = Vec::with_capacity(games_batch.len());

    let mut compressed_pgn_vec: Vec<Vec<u8>> = Vec::with_capacity(games_batch.len());

    // Populate the vectors with data from the games_batch
    for game in games_batch.iter() {
        match game.compress_pgn() {
            Ok(compressed) => {
                compressed_pgn_vec.push(compressed);
            }
            Err(e) => {
                eprintln!("Error compressing game: {:?}", e);
                // Decide how to handle errors (skip, use uncompressed, etc.)
                continue; // Here, we choose to skip the game on compression error
            }
        }

        eco_vec.push(game.eco.clone());
        white_player_vec.push(game.white_player.clone());
        black_player_vec.push(game.black_player.clone());
        date_vec.push(game.date);
        result_vec.push(game.result);
        white_elo_vec.push(game.white_elo);
        black_elo_vec.push(game.black_elo);
        time_control_vec.push(game.time_control.unwrap_or(ChessSpeed::Classical));
    }

    // Bulk insert into the 'games' table and retrieve the generated ids
    let inserted_game_ids: Vec<i32> = sqlx::query!(
        r#"
        INSERT INTO games (
            eco, white_player, black_player, date, result, white_elo, black_elo, time_control, pgn_moves
        )
        SELECT 
            t.eco, 
            t.white_player, 
            t.black_player, 
            t.date, 
            t.result::result, 
            t.white_elo, 
            t.black_elo, 
            t.time_control::chess_speed, 
            t.pgn_moves
        FROM UNNEST(
            $1::VARCHAR[],
            $2::VARCHAR[],
            $3::VARCHAR[],
            $4::DATE[],
            $5::VARCHAR[],
            $6::INTEGER[],
            $7::INTEGER[],
            $8::VARCHAR[],
            $9::BYTEA[]
        ) AS t(eco, white_player, black_player, date, result, white_elo, black_elo, time_control, pgn_moves)
        RETURNING id
        "#,
        &eco_vec,
        &white_player_vec,
        &black_player_vec,
        &date_vec as &[Option<NaiveDate>],
        &result_vec.iter().map(|r| r.to_string()).collect::<Vec<_>>(),
        &white_elo_vec,
        &black_elo_vec,
        &time_control_vec.iter().map(|tc| tc.to_string()).collect::<Vec<_>>(),
        &compressed_pgn_vec,
    )
    .fetch_all(&mut *tx)
    .await?
    .iter()
    .map(|row| row.id)
    .collect();

    // Ensure that the number of returned ids matches the number of inserted games
    if inserted_game_ids.len() != games_batch.len() {
        return Err("Mismatch between inserted games and returned game IDs".into());
    }

    // Prepare vectors for bulk inserting into the 'positions' table
    let mut position_game_ids: Vec<i32> = Vec::new();
    let mut move_numbers: Vec<i16> = Vec::new();
    let mut positions_vec: Vec<Vec<u8>> = Vec::new();

    // Process each game to extract and compress positions
    for (i, game) in games_batch.iter().enumerate() {
        let game_id = inserted_game_ids[i];

        // Initialize the position
        let mut position = Chess::default();

        let mut compressed_positions = Vec::new();
        let mut valid_game = true;

        for san_str in &game.pgn_moves {
            // Parse the move
            let san_plus: SanPlus = match san_str.parse() {
                Ok(san_plus) => san_plus,
                Err(e) => {
                    println!(
                        "Error parsing SAN: {} for game ID {}. Skipping game.",
                        e, game_id
                    );
                    valid_game = false;
                    break;
                }
            };

            // Convert to a Move
            let mv = match san_plus.san.to_move(&position) {
                Ok(mv) => mv,
                Err(e) => {
                    println!(
                        "Error converting SAN to move: {} for game ID {}. Skipping game.",
                        e, game_id
                    );
                    valid_game = false;
                    break;
                }
            };

            // Play the move
            position = match position.play(&mv) {
                Ok(pos) => pos,
                Err(e) => {
                    println!(
                        "Error playing move: {} for game ID {}. Skipping game.",
                        e, game_id
                    );
                    valid_game = false;
                    break;
                }
            };

            // Compress the position
            let compressed = CompressedPosition::compress(&position);
            compressed_positions.push(compressed);
        }

        if !valid_game {
            continue; // Skip to the next game
        }

        // Limit to 50 positions per game
        let limited_compressed_positions = compressed_positions
            .into_iter()
            .take(50)
            .collect::<Vec<_>>();

        for (move_number, compressed) in limited_compressed_positions.iter().enumerate() {
            // Ensure compressed position is 32 bytes as per table constraint
            let compressed_bytes = compressed.to_vec();
            if compressed_bytes.len() != 32 {
                println!(
                    "Unexpected compressed FEN length: {} for game ID {}. Skipping this position.",
                    compressed_bytes.len(),
                    game_id
                );
                continue;
            }

            position_game_ids.push(game_id);
            move_numbers.push((move_number + 1) as i16);
            positions_vec.push(compressed_bytes);
        }
    }

    // Bulk insert into the 'positions' table
    if !position_game_ids.is_empty() {
        sqlx::query!(
            r#"
            INSERT INTO positions (game_id, move_number, position)
            SELECT * FROM UNNEST(
                $1::INTEGER[],
                $2::SMALLINT[],
                $3::BYTEA[]
            ) AS t(game_id, move_number, position)
            "#,
            &position_game_ids,
            &move_numbers,
            &positions_vec,
        )
        .execute(&mut *tx)
        .await?;
    }

    // Commit the transaction
    tx.commit().await?;

    // Update the count of processed games
    let games_inserted = inserted_game_ids.len();
    println!("Processed and inserted {} games", games_inserted);

    // To get the number of positions inserted, you can query the positions_vec length
    let positions_inserted = positions_vec.len();
    println!("Processed and inserted {} positions", positions_inserted);

    // Clear the games_batch for the next batch
    games_batch.clear();

    Ok(())
}
