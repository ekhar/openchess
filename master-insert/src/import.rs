use pgn_reader::{BufferedReader, Color, Outcome, RawHeader, SanPlus, Skip, Visitor};
use serde::Serialize;
use serde_with::{formats::SpaceSeparator, serde_as, DisplayFromStr, StringWithSeparator};
use shakmaty::Chess;
use std::{io, mem, thread};

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

struct Batch {
    games: Vec<Game>,
}

impl Batch {
    fn last_month(&self) -> &str {
        self.games
            .last()
            .and_then(|g| g.date.as_deref())
            .unwrap_or("")
    }
}

struct Importer {
    tx: crossbeam::channel::Sender<Batch>,
    batch_size: usize,

    current: Game,
    skip: bool,
    batch: Vec<Game>,
    total_games: usize,
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
    white: Player,
    black: Player,
    #[serde_as(as = "Option<DisplayFromStr>")]
    winner: Option<Color>,
    #[serde_as(as = "StringWithSeparator<SpaceSeparator, SanPlus>")]
    moves: Vec<SanPlus>,
}

#[derive(Default, Serialize, Debug)]
struct Player {
    name: Option<String>,
    rating: Option<u16>,
}

impl Importer {
    fn new(tx: crossbeam::channel::Sender<Batch>, batch_size: usize) -> Importer {
        Importer {
            tx,
            batch_size,
            current: Game::default(),
            skip: false,
            batch: Vec::with_capacity(batch_size),

            total_games: 0,
        }
    }

    pub fn send(&mut self) -> usize {
        let batch = Batch {
            games: mem::replace(&mut self.batch, Vec::with_capacity(self.batch_size)),
        };
        self.total_games += batch.games.len();
        self.tx.send(batch).expect("send");
        self.total_games
    }
}

impl Visitor for Importer {
    type Result = ();

    fn begin_game(&mut self) {
        self.skip = false;
        self.current = Game::default();
    }

    fn header(&mut self, key: &[u8], value: RawHeader<'_>) {
        if key == b"White" {
            self.current.white.name = Some(value.decode_utf8().expect("White").into_owned());
        } else if key == b"Black" {
            self.current.black.name = Some(value.decode_utf8().expect("Black").into_owned());
        } else if key == b"WhiteElo" {
            if value.as_bytes() != b"?" {
                self.current.white.rating = Some(btoi::btoi(value.as_bytes()).expect("WhiteElo"));
            }
        } else if key == b"BlackElo" {
            if value.as_bytes() != b"?" {
                self.current.black.rating = Some(btoi::btoi(value.as_bytes()).expect("BlackElo"));
            }
        } else if key == b"TimeControl" {
            self.current.speed = Some(Speed::from_bytes(value.as_bytes()).expect("TimeControl"));
        } else if key == b"Variant" {
            self.current.variant = Some(value.decode_utf8().expect("Variant").into_owned());
        } else if key == b"Date" || key == b"UTCDate" {
            self.current.date = Some(value.decode_utf8().expect("Date").into_owned());
        } else if key == b"WhiteTitle" || key == b"BlackTitle" {
            if value.as_bytes() == b"BOT" {
                self.skip = true;
            }
        } else if key == b"Link" {
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
        } else if key == b"Site" {
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
        } else if key == b"Result" {
            match Outcome::from_ascii(value.as_bytes()) {
                Ok(outcome) => self.current.winner = outcome.winner(),
                Err(_) => self.skip = true,
            }
        } else if key == b"FEN" {
            if value.as_bytes() == b"rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1" {
                // https://github.com/ornicar/lichess-db/issues/40
                self.current.fen = None;
            } else {
                self.current.fen = Some(value.decode_utf8().expect("FEN").into_owned());
            }
        }
    }

    fn end_headers(&mut self) -> Skip {
        self.skip |= self.current.white.rating.is_none()
            || self.current.black.rating.is_none()
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
            self.batch.push(mem::take(&mut self.current));
        }

        if self.batch.len() >= self.batch_size {
            self.send();
        }
    }
}

struct Args {
    endpoint: String,
    batch_size: usize,
}

impl Default for Args {
    fn default() -> Self {
        Args {
            endpoint: "http://openchess-db:9002".to_owned(),
            batch_size: 200,
        }
    }
}

pub async fn import_pgn(pgn_str: &[u8]) -> Result<usize, io::Error> {
    let args = Args::default();

    let mut total = 0;

    let (tx, rx) = crossbeam::channel::bounded::<Batch>(50);

    let bg = thread::spawn(move || {
        let client = reqwest::blocking::Client::builder()
            .timeout(None)
            .build()
            .expect("client");

        //Todo: very janky fix this
        while let Ok(batch) = rx.recv() {
            let _res = client
                .put(format!("{}/import/lichess", args.endpoint))
                .json(&batch.games)
                .send()
                .expect("send batch");
        }
    });

    //drop the extra threads by doing this in its own lifetime
    {
        let mut reader = BufferedReader::new(pgn_str);
        let mut importer = Importer::new(tx.clone(), args.batch_size);
        reader.read_all(&mut importer)?;
        total += importer.send();
    }

    drop(tx);
    bg.join().unwrap();
    Ok(total)
}
