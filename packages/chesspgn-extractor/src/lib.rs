// filename: packages/chesspgn-extractor/src/lib.rs
use wasm_bindgen::prelude::*;
use js_sys::Uint8Array;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Request, RequestInit, Response};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

/// Fetches chess games in PGN format for a specific user, year, and month
#[wasm_bindgen]
pub async fn get_chess_games(username: &str, year: u32, month: u32) -> Result<Uint8Array, JsValue> {
    // Format the URL with padding for month
    let url = format!(
        "https://api.chess.com/pub/player/{}/games/{}/{:02}/pgn",
        username, year, month
    );

    // Create request with headers
    let opts = RequestInit::new();
    opts.set_method("GET");
    
    let request = Request::new_with_str_and_init(&url, &opts)?;
    request.headers().set("User-Agent", "Chess PGN Retriever")?;

    // Fetch from browser/Deno environment
    let window = web_sys::window().unwrap();
    let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
    let resp: Response = resp_value.dyn_into()?;

    // Check if the request was successful
    if !resp.ok() {
        return Err(JsValue::from_str(&format!(
            "Request failed with status: {}",
            resp.status()
        )));
    }

    // Get the array buffer from the response
    let array_buffer = JsFuture::from(resp.array_buffer()?).await?;
    let uint8_array = Uint8Array::new(&array_buffer);

    Ok(uint8_array)
}

/// Fetches chess games for multiple months and combines them
#[wasm_bindgen]
pub async fn get_combined_chess_games(username: &str, current_year: u32, current_month: u32, num_months: u32) -> Result<Uint8Array, JsValue> {
    let mut all_pgn_data = Vec::new();
    
    for i in 0..num_months {
        // Calculate year and month
        let (year, month) = if current_month > i {
            (current_year, current_month - i)
        } else {
            (current_year - 1, 12 - (i - current_month))
        };
        
        match get_chess_games(username, year, month).await {
            Ok(pgn_data) => {
                // Convert Uint8Array to Vec<u8>
                let len = pgn_data.length() as usize;
                let mut buffer = vec![0u8; len];
                pgn_data.copy_to(&mut buffer[..]);
                
                // Append to our combined data
                all_pgn_data.extend(buffer);
                
                console_log!("Successfully retrieved games for {}/{}", year, month);
            },
            Err(e) => {
                console_log!("Error fetching games for {}/{}: {:?}", year, month, e);
                // Continue with next month even if this one fails
            }
        }
    }
    
    // Create a new Uint8Array with the combined data
    let result = Uint8Array::new_with_length(all_pgn_data.len() as u32);
    // Use the set method to copy data from our Vec<u8> to the Uint8Array
    result.set(&Uint8Array::from(&all_pgn_data[..]), 0);
    
    Ok(result)
}
