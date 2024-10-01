// src/lib.rs
use std::fmt;
use wasm_bindgen::prelude::*;
mod fen_compress;
mod huffman_code;
mod pgn_compress;
mod psqt;

use fen_compress::CompressedPosition;
use pgn_compress::Encoder as PgnEncoder;
use pgn_reader::{BufferedReader, RawHeader, SanPlus, Skip, Visitor};
use serde::Serialize;
use serde_with::{formats::SpaceSeparator, serde_as, DisplayFromStr, StringWithSeparator};
use std::mem;

#[wasm_bindgen]
pub struct FenCompressor;

#[wasm_bindgen]
impl FenCompressor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        FenCompressor
    }

    pub fn compress(&self, fen: &str) -> Result<Vec<u8>, JsValue> {
        let compressed =
            CompressedPosition::compress(fen).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let mut data = Vec::new();
        compressed.write_to_big_endian(&mut data);

        Ok(data)
    }
}

impl Default for FenCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
pub struct PgnCompressor;

#[wasm_bindgen]
impl PgnCompressor {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        PgnCompressor
    }

    pub fn compress(&self, pgn: &str) -> Result<Vec<u8>, JsValue> {
        let mut encoder = PgnEncoder::new();
        for mv in pgn.split_whitespace() {
            encoder
                .encode_move(mv)
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
        }
        let compressed = encoder.finalize();
        Ok(compressed.to_bytes())
    }

    pub fn decompress(&self, data: &[u8], num_moves: usize) -> Result<String, JsValue> {
        let encoder = PgnEncoder::new();
        let bit_vec = bit_vec::BitVec::from_bytes(data);
        let moves = encoder
            .decode(&bit_vec, num_moves)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(moves.join(" "))
    }
}

impl Default for PgnCompressor {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Serialize, Copy, Clone)]
#[serde(rename_all = "camelCase")]
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
        if bytes == b"-" || bytes.contains(&b'/') {
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
impl Default for GameResult {
    fn default() -> Self {
        Self::Draw
    }
}

#[serde_as]
#[derive(Default, Serialize, Debug)]
struct Game {
    chess_site: Option<String>,
    variant: Option<String>,
    speed: Option<Speed>,
    fen: Option<String>,
    id: Option<String>,
    date: Option<String>,
    white_player: Option<String>,
    black_player: Option<String>,
    white_elo: Option<u16>,
    black_elo: Option<u16>,
    #[serde_as(as = "DisplayFromStr")]
    result: GameResult,
    eco: Option<String>, // Added eco field
    #[serde_as(as = "StringWithSeparator<SpaceSeparator, SanPlus>")]
    moves: Vec<SanPlus>,
}

#[derive(Default, Serialize, Debug)]
struct Player {
    name: Option<String>,
    rating: Option<u16>,
}

#[derive(Debug, Copy, Clone)]
pub enum GameResult {
    White,
    Black,
    Draw,
}
impl GameResult {
    fn from_str(s: &str) -> Self {
        match s {
            "white" => GameResult::White,
            "black" => GameResult::Black,
            "draw" => GameResult::Draw,
            _ => panic!("Invalid game result: {}", s), // Or handle this error as appropriate for your application
        }
    }
    fn to_str(&self) -> &str {
        match self {
            GameResult::White => "white",
            GameResult::Black => "black",
            GameResult::Draw => "draw",
        }
    }
}
//impl format display for GameResult
impl fmt::Display for GameResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_str())
    }
}

struct Importer {
    games: Vec<Game>,
    current: Game,
    skip: bool,
}

impl Default for Importer {
    fn default() -> Self {
        Self::new()
    }
}

impl Importer {
    pub fn new() -> Importer {
        Importer {
            games: Vec::new(),
            current: Game::default(),
            skip: false,
        }
    }
}

impl Visitor for Importer {
    type Result = ();

    fn begin_game(&mut self) {
        self.skip = false;
        self.current = Game::default();
    }

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        match key {
            b"White" => {
                self.current.white_player = Some(value.decode_utf8().expect("White").into_owned());
            }
            b"Black" => {
                self.current.black_player = Some(value.decode_utf8().expect("Black").into_owned());
            }
            b"WhiteElo" => {
                if value.as_bytes() != b"?" {
                    self.current.white_elo = Some(btoi::btoi(value.as_bytes()).expect("WhiteElo"));
                }
            }
            b"BlackElo" => {
                if value.as_bytes() != b"?" {
                    self.current.black_elo = Some(btoi::btoi(value.as_bytes()).expect("BlackElo"));
                }
            }
            b"TimeControl" => {
                self.current.speed =
                    Some(Speed::from_bytes(value.as_bytes()).expect("TimeControl"));
            }
            b"Variant" => {
                self.current.variant = Some(value.decode_utf8().expect("Variant").into_owned());
            }

            b"ECO" => {
                self.current.eco = Some(value.decode_utf8().expect("ECO").into_owned());
            }
            b"Date" | b"UTCDate" => {
                self.current.date = Some(value.decode_utf8().expect("Date").into_owned());
            }
            b"WhiteTitle" | b"BlackTitle" => {
                if value.as_bytes() == b"BOT" {
                    self.skip = true;
                }
            }
            b"Link" => {
                self.current.id = Some(
                    String::from_utf8(
                        value
                            .as_bytes()
                            .rsplitn(2, |ch| *ch == b'/')
                            .next()
                            .expect("Site")
                            .to_owned(),
                    )
                    .expect("Site"),
                );
            }
            b"Site" => {
                if value.as_bytes().starts_with(b"https://lichess.org/") {
                    self.current.chess_site = Some("lichess".to_owned());
                    self.current.id = Some(
                        String::from_utf8(
                            value
                                .as_bytes()
                                .rsplitn(2, |ch| *ch == b'/')
                                .next()
                                .expect("Site")
                                .to_owned(),
                        )
                        .expect("Site"),
                    );
                } else if value.as_bytes().starts_with(b"Chess.com") {
                    self.current.chess_site = Some("chesscom".to_owned());
                }
            }
            b"Result" => {
                let result_str = value.decode_utf8().unwrap_or_default().into_owned();
                self.current.result = match result_str.as_str() {
                    "1-0" => GameResult::White,
                    "0-1" => GameResult::Black,
                    "1/2-1/2" => GameResult::Draw,
                    _ => {
                        self.skip = true;
                        GameResult::Draw
                    }
                };
            }
            b"FEN" => {
                if value.as_bytes() == b"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" {
                    // https://github.com/ornicar/lichess-db/issues/40
                    self.current.fen = None;
                } else {
                    self.current.fen = Some(value.decode_utf8().expect("FEN").into_owned());
                }
            }
            _ => {}
        }
    }

    fn end_headers(&mut self) -> Skip {
        self.skip |= self.current.white_elo.is_none()
            || self.current.black_elo.is_none()
            || self.current.chess_site.is_none();
        Skip(self.skip)
    }

    fn san(&mut self, san: SanPlus) {
        self.current.moves.push(san);
    }

    fn begin_variation(&mut self) -> Skip {
        Skip(true) // stay in the mainline
    }

    fn end_game(&mut self) {
        if !self.skip {
            self.games.push(mem::take(&mut self.current));
        }
    }
}

#[wasm_bindgen]
pub struct PgnParser;

#[wasm_bindgen]
impl PgnParser {
    #[wasm_bindgen(constructor)]
    pub fn new() -> PgnParser {
        PgnParser {}
    }

    /// Parses PGN data and returns JSON string of games
    pub fn parse_pgn(&self, pgn_data: &str) -> Result<String, JsValue> {
        let mut reader = BufferedReader::new(pgn_data.as_bytes());
        let mut importer = Importer::new();

        let _read_all = reader.read_all(&mut importer);

        let games = &importer.games;

        serde_json::to_string(games).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

impl Default for PgnParser {
    fn default() -> Self {
        Self::new()
    }
}
