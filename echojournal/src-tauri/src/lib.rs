use dotenvy::dotenv;
use std::env;
use serde::{Deserialize, Serialize};
use tauri::command;
use tokio::time::{sleep, Duration};
use chrono::{Local, Datelike, NaiveDate};


#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<Content>,
}

#[derive(Serialize)]
struct Content {
    parts: Vec<Part>,
}

#[derive(Serialize)]
struct Part {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: String,
}

#[tauri::command]
fn is_date_locked(target_date_str: String) -> Result<bool, String> {
    let now = Local::now().date_naive();
    
    // Parse the date we are trying to write to (Format: YYYY-MM-DD)
    let target_date = NaiveDate::parse_from_str(&target_date_str, "%Y-%m-%d")
        .map_err(|_| "Invalid date format".to_string())?;

    // Calculate the difference
    let diff = now.signed_duration_since(target_date).num_days();

    // If the date is more than 7 days in the past, it's locked.
    // Also lock if the date is in the future (preventing time travel).
    Ok(diff > 7 || diff < 0)
}


#[command]
async fn refine_thought(input: String) -> Result<String, String> {
    dotenv().ok();
    let api_key = env::var("GEMINI_API_KEY").map_err(|_| "Key not found".to_string())?;

    // Switching to the Stable 2.5 Flash model for reliability
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let client = reqwest::Client::new();
    let prompt = format!(
        "Refine into ONE concise, professional bullet point starting with an active verb. No bolding. No intro. Input: {}",
        input
    );

    let body = GeminiRequest {
        contents: vec![Content {
            parts: vec![Part { text: prompt }],
        }],
    };

    // --- Simple Retry Logic ---
    let mut attempts = 0;
    let max_attempts = 2;

    while attempts < max_attempts {
        let res = client.post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let status = res.status();
        
        if status.is_success() {
            let json: GeminiResponse = res.json().await.map_err(|e| e.to_string())?;
            return Ok(json.candidates[0].content.parts[0].text.trim().to_string());
        } else if status.as_u16() == 503 && attempts < max_attempts - 1 {
            attempts += 1;
            sleep(Duration::from_secs(2)).await; // Wait 2s before retrying
            continue;
        } else {
            let err_text = res.text().await.unwrap_or_default();
            return Err(format!("API Error ({}): {}", status, err_text));
        }
    }

    Err("Failed after multiple retries due to high demand.".into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            refine_thought, 
            is_date_locked
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}