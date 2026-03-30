use dotenvy::dotenv;
use std::env;
use serde::{Deserialize, Serialize};
use tauri::command;
use tokio::time::{sleep, Duration};
use chrono::{Local, Datelike, NaiveDate};
mod tags;

#[derive(Serialize)]
struct GeminiRequest { contents: Vec<Content>, }

#[derive(Serialize)]
struct Content { parts: Vec<Part>, }

#[derive(Serialize)]
struct Part { text: String, }

#[derive(Deserialize)]
struct GeminiResponse { candidates: Vec<Candidate>, }

#[derive(Deserialize)]
struct Candidate { content: ResponseContent, }

#[derive(Deserialize)]
struct ResponseContent { parts: Vec<ResponsePart>, }

#[derive(Deserialize)]
struct ResponsePart { text: String, }

#[derive(Serialize, Deserialize)]
pub struct RefinedOutput {
    pub text: String,
    pub tag: String,
}

#[tauri::command]
fn is_date_locked(target_date_str: String) -> Result<bool, String> {
    let now = Local::now().date_naive();
    let target_date = NaiveDate::parse_from_str(&target_date_str, "%Y-%m-%d")
        .map_err(|_| "Invalid date format".to_string())?;
    let diff = now.signed_duration_since(target_date).num_days();
    Ok(diff > 7 || diff < 0)
}


#[tauri::command]
async fn refine_thought(input: String, settings_json: String) -> Result<RefinedOutput, String> {
    dotenv().ok();
    let api_key = env::var("GEMINI_API_KEY").map_err(|_| "GEMINI_API_KEY not found".to_string())?;
    
    let tag_list: Vec<String> = tags::get_tag_registry().iter().map(|t| t.id.clone()).collect();
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}", api_key);

    let prompt = format!(
        "USER_SETTINGS: {settings}
         VALID_TAGS: {tags:?}

         GOAL: Process the journal entry.
         1. LITERAL MODE: If input is in quotes, return text exactly as is (no quotes).
         2. REFINED MODE: If no quotes, rewrite as a 1-sentence bullet point using the tone specified in USER_SETTINGS for the detected tag.
         3. CATEGORIZATION: Select the most appropriate tag from VALID_TAGS.

         RETURN JSON ONLY: {{\"text\": \"...\", \"tag\": \"...\"}}
         INPUT: {input}",
        settings = settings_json,
        tags = tag_list,
        input = input
    );

    let client = reqwest::Client::new();
    let res = client.post(&url).json(&GeminiRequest {
        contents: vec![Content { parts: vec![Part { text: prompt }] }]
    }).send().await.map_err(|e| e.to_string())?;

    let json: GeminiResponse = res.json().await.map_err(|e| e.to_string())?;
    let raw_output = json.candidates[0].content.parts[0].text.trim();
    
    
    let clean_json = raw_output.trim_start_matches("```json").trim_end_matches("```").trim();

    let output: RefinedOutput = serde_json::from_str(clean_json)
        .map_err(|_| format!("AI Parse Error: {}", clean_json))?;

    Ok(output)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![refine_thought, is_date_locked])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}