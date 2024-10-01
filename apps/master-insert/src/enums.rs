// Define an enum to represent the game result in PostgreSQL

use std::fmt;
#[derive(Debug, Copy, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "result", rename_all = "lowercase")]
pub enum ResultType {
    White,
    Black,
    Draw,
}

//make a default for ResultType
impl Default for ResultType {
    fn default() -> Self {
        ResultType::Draw
    }
}

// Implement Display for the ResultType enum
impl fmt::Display for ResultType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ResultType::White => write!(f, "white"),
            ResultType::Black => write!(f, "black"),
            ResultType::Draw => write!(f, "draw"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, sqlx::Type)]
#[sqlx(type_name = "chess_speed", rename_all = "camelCase")] // Adjust based on PostgreSQL definition
pub enum ChessSpeed {
    UltraBullet,
    Bullet,
    Blitz,
    Rapid,
    Classical,
    Correspondence,
}

// Implement Display for the ChessSpeed enum
impl fmt::Display for ChessSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ChessSpeed::UltraBullet => write!(f, "UltraBullet"),
            ChessSpeed::Bullet => write!(f, "Bullet"),
            ChessSpeed::Blitz => write!(f, "Blitz"),
            ChessSpeed::Rapid => write!(f, "Rapid"),
            ChessSpeed::Classical => write!(f, "Classical"),
            ChessSpeed::Correspondence => write!(f, "Correspondence"),
        }
    }
}

impl ChessSpeed {
    pub fn from_seconds_and_increment(seconds: u64, increment: u64) -> ChessSpeed {
        let total = seconds + 40 * increment;

        if total < 30 {
            ChessSpeed::UltraBullet
        } else if total < 180 {
            ChessSpeed::Bullet
        } else if total < 480 {
            ChessSpeed::Blitz
        } else if total < 1500 {
            ChessSpeed::Rapid
        } else if total < 21_600 {
            ChessSpeed::Classical
        } else {
            ChessSpeed::Correspondence
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<ChessSpeed, ()> {
        if bytes == b"-" || bytes.contains(&b'/') {
            return Ok(ChessSpeed::Correspondence);
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

        Ok(ChessSpeed::from_seconds_and_increment(seconds, increment))
    }
}
