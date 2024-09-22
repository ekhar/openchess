// main.rs
use dotenv::dotenv;
use pgn_reader::{BufferedReader, RawHeader, SanPlus, Skip, Visitor};
use shakmaty::{fen::Fen, CastlingMode, Chess, Position};
use sqlx::postgres::PgPoolOptions;
use sqlx::types::chrono::NaiveDate;
use sqlx::Row;
use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::{mpsc, Mutex};

mod compression; // Include your helper functions here

#[derive(Debug)]
struct Importer {
    tx: mpsc::Sender<Vec<Game>>,
    batch_size: usize,
    current_game: Game,
    skip: bool,
    batch_games: Vec<Game>,
}

#[derive(Debug, Copy, Clone, sqlx::Type)]
#[sqlx(type_name = "mood", rename_all = "lowercase")]
enum Speed {
    UltraBullet,
    Bullet,
    Blitz,
    Rapid,
    Classical,
    Correspondence,
}

impl Speed {
    fn from_seconds_and_increment(seconds: u64, increment: u64) -> Speed {
        let total = seconds + 40 * increment;

        if total < 30 {
            Speed::UltraBullet
        } else if total < 180 {
            Speed::Bullet
        } else if total < 480 {
            Speed::Blitz
        } else if total < 1500 {
            Speed::Rapid
        } else if total < 21_600 {
            Speed::Classical
        } else {
            Speed::Correspondence
        }
    }

    fn from_bytes(bytes: &[u8]) -> Result<Speed, ()> {
        if bytes == b"-" {
            return Ok(Speed::Correspondence);
        }
        if bytes.contains(&b'/') {
            return Ok(Speed::Correspondence);
        }

        let mut parts = bytes.splitn(2, |ch| *ch == b'+');

        let seconds = match parts.next() {
            Some(seconds_bytes) => btoi::btou(seconds_bytes).map_err(|_| ())?,
            None => return Err(()),
        };

        let increment = match parts.next() {
            Some(increment_bytes) => btoi::btou(increment_bytes).map_err(|_| ())?,
            None => 0,
        };

        Ok(Speed::from_seconds_and_increment(seconds, increment))
    }
}

#[derive(Default, Debug, Clone)]
struct Game {
    eco: String,
    white_player: String,
    black_player: String,
    date: Option<NaiveDate>,
    result: String, // 'white', 'black', 'draw'
    pgn_moves: Vec<String>,
    white_elo: i16,
    black_elo: i16,
    time_control: Option<Speed>, // 'ultraBullet', etc.
    fen: Option<String>,
}

impl Visitor for Importer {
    type Result = ();

    fn begin_game(&mut self) {
        self.skip = false;
        self.current_game = Game::default();
    }

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        match key {
            b"White" => {
                self.current_game.white_player = value.decode_utf8().unwrap().into_owned();
            }
            b"Black" => {
                self.current_game.black_player = value.decode_utf8().unwrap().into_owned();
            }
            b"WhiteElo" => {
                if value.as_bytes() != b"?" {
                    self.current_game.white_elo = btoi::btoi(value.as_bytes()).unwrap_or(0);
                } else {
                    self.current_game.white_elo = 0;
                }
            }
            b"BlackElo" => {
                if value.as_bytes() != b"?" {
                    self.current_game.black_elo = btoi::btoi(value.as_bytes()).unwrap_or(0);
                } else {
                    self.current_game.black_elo = 0;
                }
            }
            b"Date" => {
                let date_str = value.decode_utf8().unwrap().into_owned();
                if let Ok(date) = NaiveDate::parse_from_str(&date_str, "%Y.%m.%d") {
                    self.current_game.date = Some(date);
                }
            }
            b"Result" => {
                let result_str = value.decode_utf8().unwrap().into_owned();
                self.current_game.result = match result_str.as_str() {
                    "1-0" => "white".to_string(),
                    "0-1" => "black".to_string(),
                    "1/2-1/2" => "draw".to_string(),
                    _ => {
                        self.skip = true;
                        "draw".to_string()
                    }
                };
            }
            b"ECO" => {
                self.current_game.eco = value.decode_utf8().unwrap().into_owned();
            }
            b"TimeControl" => {
                Speed::from_bytes(value.as_bytes()).expect("TimeControl");
            }
            b"FEN" => {
                self.current_game.fen = Some(value.decode_utf8().unwrap().into_owned());
            }
            _ => {}
        }
    }

    fn end_headers(&mut self) -> Skip {
        self.skip |= self.current_game.white_player.is_empty()
            || self.current_game.black_player.is_empty()
            || self.current_game.eco.is_empty();
        Skip(self.skip)
    }

    fn san(&mut self, san_plus: SanPlus) {
        self.current_game.pgn_moves.push(format!("{}", san_plus));
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true) // Skip variations
    }

    fn end_game(&mut self) {
        if !self.skip {
            self.batch_games.push(self.current_game.clone());
        }

        if self.batch_games.len() >= self.batch_size {
            let batch = std::mem::take(&mut self.batch_games);
            let _ = self.tx.blocking_send(batch);
        }
    }
}

#[derive(Error, Debug)]
enum ImportError {
    #[error("IO error")]
    IoError(#[from] std::io::Error),
    #[error("SQLx error")]
    SqlxError(#[from] sqlx::Error),
}

#[tokio::main]
async fn main() -> Result<(), ImportError> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_path>", args[0]);
        std::process::exit(1);
    }

    let file_path = &args[1];
    // Set up the database connection
    dotenv().ok();
    let supa_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&supa_url)
        .await?;

    // Create the mpsc channel
    let (tx, mut rx) = mpsc::channel::<Vec<Game>>(10);

    // Positions cache shared between batches
    let positions_cache = Arc::new(Mutex::new(HashMap::new()));

    // Spawn the async task to process batches
    let pool_clone = pool.clone();
    let positions_cache_clone = positions_cache.clone();
    tokio::spawn(async move {
        while let Some(batch) = rx.recv().await {
            if let Err(e) = process_batch(&pool_clone, positions_cache_clone.clone(), batch).await {
                eprintln!("Error processing batch: {:?}", e);
            }
        }
    });

    // Create the importer
    let mut importer = Importer {
        tx,
        batch_size: 1000,
        current_game: Game::default(),
        skip: false,
        batch_games: Vec::new(),
    };

    // Read the PGN file
    let file = File::open(file_path)?;
    let mut reader = BufferedReader::new(BufReader::new(file));

    // Read all games
    reader.read_all(&mut importer)?;

    // After reading all games, send any remaining batch
    if !importer.batch_games.is_empty() {
        let batch = std::mem::take(&mut importer.batch_games);
        importer.tx.send(batch).await.unwrap();
    }

    Ok(())
}

async fn process_batch(
    pool: &sqlx::Pool<sqlx::Postgres>,
    positions_cache: Arc<Mutex<HashMap<Vec<u8>, i32>>>,
    batch_games: Vec<Game>,
) -> Result<(), sqlx::Error> {
    // Collect all unique compressed_fens across all games
    let mut compressed_fens_set: HashSet<Vec<u8>> = HashSet::new();

    // For mapping compressed_fen to position_id
    let mut positions_map: HashMap<Vec<u8>, i32> = HashMap::new();

    // For storing game data and positions for master_game_positions
    let mut game_data_list = Vec::new(); // (Game, positions_in_game)
    let mut master_games_inserts = Vec::new();

    // First pass: collect compressed_fens and positions_in_game
    for game in batch_games {
        // Initialize the position
        let mut position = if let Some(fen) = &game.fen {
            match Fen::from_ascii(fen.as_bytes()) {
                Ok(fen) => match fen.into_position(CastlingMode::Standard) {
                    Ok(pos) => pos,
                    Err(_) => {
                        println!(
                            "Error creating position from FEN for game ID: {}. Skipping.",
                            game.fen.unwrap()
                        );
                        continue; // Skip to the next iteration
                    }
                },
                Err(_) => {
                    println!(
                        "Error parsing FEN for game ID: {}. Skipping.",
                        game.fen.unwrap()
                    );
                    continue; // Skip to the next iteration
                }
            }
        } else {
            Chess::default()
        };
        let mut positions_in_game = Vec::new();

        // Create an Encoder instance
        let mut encoder = compression::pgn_compress::Encoder::new();

        // Collect the positions and moves
        for (move_number, san_str) in game.pgn_moves.iter().enumerate() {
            let san_plus: SanPlus = san_str.parse().unwrap();
            let mv = san_plus.san.to_move(&position).unwrap();

            // Encode the move
            encoder.encode_move(san_str).unwrap();

            // Play the move
            position = position.play(&mv).unwrap();

            let compressed_fen = compression::fen_compress::CompressedPosition::compress(&position);

            let mut data = Vec::new();
            compressed_fen.write_to_big_endian(&mut data);
            compressed_fens_set.insert(data);
            positions_in_game.push((compressed_fen, move_number + 1)); // move_number starts from 1
        }

        // Finalize the compression
        let compressed_pgn = encoder.finalize();

        // You can now use compressed_pgn, which is a BitVec
        // If you need it as bytes, you can convert it:
        let compressed_pgn_bytes: Vec<u8> = compressed_pgn.to_bytes(); // Prepare data for master_games
        master_games_inserts.push((
            game.eco.clone(),
            game.white_player.clone(),
            game.black_player.clone(),
            game.date,
            game.result.clone(),
            compressed_pgn_bytes,
            game.white_elo,
            game.black_elo,
            game.time_control,
        ));

        game_data_list.push((game, positions_in_game));
    }

    // Now, check which compressed_fens are not in positions_cache
    let positions_cache_lock = positions_cache.lock().await;
    let cached_fens: HashSet<Vec<u8>> = positions_cache_lock.keys().cloned().collect();
    let compressed_fens_to_check_db: Vec<Vec<u8>> = compressed_fens_set
        .difference(&cached_fens)
        .cloned()
        .collect();

    drop(positions_cache_lock); // Release the lock

    // Now, check in database
    if !compressed_fens_to_check_db.is_empty() {
        // Build a query to select positions with compressed_fen in (...)
        let rows = sqlx::query!(
            "SELECT id, compressed_fen FROM positions WHERE compressed_fen = ANY($1)",
            &compressed_fens_to_check_db
        )
        .fetch_all(pool)
        .await?;

        let mut positions_cache_lock = positions_cache.lock().await;

        // Update positions_cache with the found positions
        for row in rows {
            let id: i32 = row.id;
            let compressed_fen: Vec<u8> = row.compressed_fen.unwrap_or_default();
            positions_cache_lock.insert(compressed_fen.clone(), id);
            positions_map.insert(compressed_fen, id);
        }

        // Now, find compressed_fens not found in database
        let compressed_fens_not_in_db: Vec<Vec<u8>> = compressed_fens_to_check_db
            .into_iter()
            .filter(|cf| !positions_map.contains_key(cf))
            .collect();

        // Insert new positions into positions table
        if !compressed_fens_not_in_db.is_empty() {
            let mut query_builder =
                sqlx::QueryBuilder::new("INSERT INTO positions (compressed_fen) VALUES ");
            query_builder.push_values(compressed_fens_not_in_db.iter(), |mut b, cf| {
                b.push_bind(cf);
            });
            query_builder.push(" RETURNING id, compressed_fen");

            // Execute the query and get the ids
            let rows = query_builder.build().fetch_all(pool).await?;

            // Update positions_cache and positions_map with the new ids
            for row in rows {
                let id: i32 = row.get("id");
                let compressed_fen: Vec<u8> = row.get("compressed_fen");
                positions_cache_lock.insert(compressed_fen.clone(), id);
                positions_map.insert(compressed_fen, id);
            }
        }

        drop(positions_cache_lock); // Release the lock
    }

    // Now, process the games and prepare data for master_game_positions
    let mut master_game_positions_inserts = Vec::new();
    let mut game_ids = Vec::new();

    for (game_index, (_game, positions_in_game)) in game_data_list.into_iter().enumerate() {
        // Insert into master_games
        let row: (i32,) = sqlx::query_as(
            "INSERT INTO master_games
            (eco, white_player, black_player, date, result, compressed_pgn, white_elo, black_elo, time_control)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id",
        )
        .bind(&master_games_inserts[game_index].0)
        .bind(&master_games_inserts[game_index].1)
        .bind(&master_games_inserts[game_index].2)
        .bind(master_games_inserts[game_index].3)
        .bind(&master_games_inserts[game_index].4)
        .bind(&master_games_inserts[game_index].5)
        .bind(master_games_inserts[game_index].6)
        .bind(master_games_inserts[game_index].7)
        .bind(master_games_inserts[game_index].8)
        .fetch_one(pool)
        .await?;

        let game_id = row.0;
        game_ids.push(game_id);

        // Prepare data for master_game_positions
        let mut positions_data = Vec::new();

        for (compressed_fen, move_number) in positions_in_game {
            let mut data = Vec::new();
            compressed_fen.write_to_big_endian(&mut data);
            let position_id = {
                let positions_cache_lock = positions_cache.lock().await;
                *positions_cache_lock.get(&data).unwrap()
            };
            positions_data.push((game_id, position_id, move_number as i32));
        }

        master_game_positions_inserts.push(positions_data);
    }

    // Now, insert into master_game_positions
    for positions_data in master_game_positions_inserts {
        if positions_data.is_empty() {
            continue;
        }

        let mut query_builder = sqlx::QueryBuilder::new(
            "INSERT INTO master_game_positions (game_id, position_id, move_number) VALUES ",
        );
        query_builder.push_values(
            positions_data.iter(),
            |mut b, (game_id, position_id, move_number)| {
                b.push_bind(*game_id)
                    .push_bind(*position_id)
                    .push_bind(*move_number);
            },
        );

        // Execute the query
        query_builder.build().execute(pool).await?;
    }

    Ok(())
}
