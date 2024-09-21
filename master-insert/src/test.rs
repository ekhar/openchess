use pgn_reader::{Nag, RawComment, RawHeader, SanPlus, Visitor};
use serde::{Deserialize, Serialize};
use shakmaty::{
    fen::Fen,
    zobrist::{Zobrist128, ZobristHash},
    Chess, Position,
};
use sqlx::FromRow;
use sqlx::Type;
use std::fmt;
use std::fmt::Write;
use std::{str, usize};

#[derive(Debug, Default)]
pub enum Table {
    Master,
    #[default]
    Player,
}

impl Table {
    pub fn as_str(&self) -> &str {
        match self {
            Table::Master => "master_games",
            Table::Player => "player_games",
        }
    }
}

#[derive(Default, Debug)]
pub struct Games {
    pub games: Vec<Pgn>,
    pub pos: Chess,
    pub table_set: Table,
}

#[derive(Default, Debug, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct Move {
    pub fen: String,
}

#[derive(Default, Debug, Serialize, Deserialize, FromRow, PartialEq, Eq)]
pub struct Pgn {
    pub id: i32,
    pub eco: String,
    pub white_player: String,
    pub black_player: String,
    pub date: Option<String>,
    pub result: WinnerType,
    pub compressed_pgn: Vec<u8>,
    pub position_sequence: Vec<i32>,
    pub white_elo: Option<i16>,
    pub black_elo: Option<i16>,
    pub time_control: Option<Speed>,
    // Fields specific to player_games
    #[sqlx(skip_if = "Table::Master")]
    pub site: Option<Site>,
    // Fields to maintain compatibility with existing code
    #[sqlx(skip)]
    pub moves: Vec<Move>,
    #[sqlx(skip)]
    pub moves_str: String,
    // Additional fields from the original Pgn struct that might be used in the code
    pub event: Option<String>,
    pub round: Option<String>,
    pub utc_date: Option<String>,
    pub termination: Option<String>,
    pub tournament: Option<String>,
}

#[derive(Debug, PartialEq, Eq, Type, Serialize, Deserialize, Clone, Copy)]
#[sqlx(type_name = "game_result", rename_all = "lowercase")]
pub enum WinnerType {
    White,
    Black,
    Draw,
}

#[derive(Debug, PartialEq, Eq, Type, Serialize, Deserialize, Clone, Copy)]
#[sqlx(type_name = "site", rename_all = "lowercase")]
pub enum Site {
    Chesscom,
    Lichess,
    Custom,
}

#[derive(Debug, PartialEq, Eq, Type, Serialize, Deserialize, Clone, Copy)]
#[sqlx(type_name = "speed", rename_all = "lowercase")]
pub enum Speed {
    UltraBullet,
    Bullet,
    Blitz,
    Rapid,
    Classical,
    Correspondence,
}

impl fmt::Display for WinnerType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WinnerType::White => write!(f, "white"),
            WinnerType::Black => write!(f, "black"),
            WinnerType::Draw => write!(f, "draw"),
        }
    }
}

// Implement FromRow for PgHasArrayType
use sqlx::postgres::PgHasArrayType;

impl PgHasArrayType for WinnerType {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_game_result")
    }
}

impl Games {
    pub fn mru_move_mut(&mut self) -> Option<&mut Move> {
        self.games.last_mut().unwrap().moves.last_mut()
    }
}
