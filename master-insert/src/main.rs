mod import;
use actix_web::middleware::Logger;
use actix_web::{get, post, web, App, HttpResponse, HttpServer};
use chrono::Datelike;
use reqwest::header::{HeaderMap, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
struct AppState {
    chess_api_lock: Arc<Mutex<()>>, // Mutex to ensure sequential API requests
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:9003".to_string());

    let state = web::Data::new(AppState {
        chess_api_lock: Arc::new(Mutex::new(())),
    });

    // Setup and run the HTTP server
    HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allowed_origin("http://localhost:3000") // Specify allowed origin
            .allowed_methods(vec!["GET", "POST"]) // Specify allowed methods
            .allowed_headers(vec![
                actix_web::http::header::AUTHORIZATION,
                actix_web::http::header::ACCEPT,
            ])
            .allowed_header(actix_web::http::header::CONTENT_TYPE)
            .max_age(3600); // Cache the CORS preflight requests for 1 hour
                            //
        App::new()
            .wrap(cors)
            .wrap(Logger::default())
            .app_data(state.clone()) // Share state across handlers
            .service(import_games)
            .service(health_check)
    })
    .bind(bind_address)?
    .run()
    .await
}

#[derive(Serialize, Deserialize)]
struct Response {
    len_games: usize,
}

#[post("/games/import")]
async fn import_games(
    state: actix_web::web::Data<AppState>,
    body: String,
) -> Result<HttpResponse, actix_web::Error> {
    let profile_name = body;
    let mut pgn = Vec::new();

    let current_date = chrono::Utc::now();
    for i in 0..3 {
        let year = current_date.year();
        let month = current_date.month() - i as u32; // Adjust month for each iteration if needed
        let games = get_chess_games(&state, &profile_name, year as u32, month)
            .await
            .unwrap();
        pgn.extend(games);
    }

    let num_games = import::import_pgn(&pgn).await.unwrap();
    Ok(HttpResponse::Ok().json(Response {
        len_games: num_games,
    }))
}

async fn get_chess_games(
    state: &actix_web::web::Data<AppState>,
    username: &str,
    year: u32,
    month: u32,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let _lock = state.chess_api_lock.lock().await; // Acquire the lock

    let url = format!(
        "https://api.chess.com/pub/player/{}/games/{}/{:02}/pgn",
        username, year, month
    );

    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, "My Chess App".parse().unwrap());

    let client = reqwest::Client::new();
    let response = client.get(&url).headers(headers).send().await?;

    if response.status().is_success() {
        let pgn_bytes = response.bytes().await?;
        Ok(pgn_bytes.to_vec())
    } else {
        Err(format!("Request failed with status: {}", response.status()).into())
    }
}

// Health check endpoint
#[get("/monitor")]
async fn health_check() -> HttpResponse {
    println!("Health check");
    HttpResponse::Ok().finish()
}
