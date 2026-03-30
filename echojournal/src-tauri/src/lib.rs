use dotenvy::dotenv;
use std::env;
use serde::{Deserialize, Serialize};
use tauri::command;
use tokio::time::{sleep, Duration};
use chrono::{Local, Datelike, NaiveDate};
mod tags;
pub mod export;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RawEntry {
    pub date: String,
    pub time: String,
    pub text: String,
    pub tag: String,
}

#[tauri::command]
async fn get_all_entries(journal_path: String) -> Result<Vec<RawEntry>, String> {
    use std::fs;
    use std::path::Path;

    let path = Path::new(&journal_path);
    if !path.exists() {
        return Ok(vec![]);
    }

    let mut all_entries = Vec::new();
    let entries = fs::read_dir(path).map_err(|e| e.to_string())?;

    for entry in entries {
        let entry = entry.map_err(|e| e.to_string())?;
        let file_path = entry.path();
        if file_path.extension().and_then(|s| s.to_str()) == Some("md") {
            let content = fs::read_to_string(&file_path).map_err(|e| e.to_string())?;
            
            // --- FIX 1: Use proper string splitting ---
            // We split by "### " to find the day blocks
            let days = content.split("### ").filter(|s| !s.trim().is_empty());

            for day_block in days {
                let lines: Vec<&str> = day_block.lines().collect();
                if lines.is_empty() { continue; }

                // The first line is now just the date (since "### " was removed by split)
                let date = lines[0].trim().to_string();

                for line in lines.iter().skip(1) {
                    if line.starts_with("- **") {
                        let parts: Vec<&str> = line.split("**: ").collect();
                        if parts.len() < 2 { continue; }

                        let time = parts[0].replace("- **", "").trim().to_string();
                        let rest = parts[1];

                        // Separate text and tag
                        let tag_split: Vec<&str> = rest.rsplitn(2, " #").collect();
                        let (text, tag) = if tag_split.len() == 2 {
                            (tag_split[1].trim().to_string(), tag_split[0].trim().to_string())
                        } else {
                            (rest.trim().to_string(), "Uncategorized".to_string())
                        };
                        all_entries.push(RawEntry { date: date.clone(), time, text, tag });
                    }
                }
            }
        }
    }

    all_entries.sort_by(|a, b| a.date.cmp(&b.date).then(a.time.cmp(&b.time)));
    Ok(all_entries)
}

#[tauri::command]
async fn ask_oracle(question: String, journal_data: String) -> Result<String, String> {
    dotenv().ok();
    let api_key = env::var("GEMINI_API_KEY").map_err(|_| "Key not found".to_string())?;

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let prompt = format!(
        "SYSTEM: You are the 'Echo Oracle'. Answer using ONLY the provided JSON data. 
         CITATIONS: cite (YYYY-MM-DD, #Tag).
         DATA: {}
         USER QUESTION: {}",
        journal_data, question
    );

    let body = GeminiRequest {
        contents: vec![Content { parts: vec![Part { text: prompt }] }],
    };

    let client = reqwest::Client::new();
    let res = client.post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network Error: {}", e))?;

    // --- NEW: Check status before decoding ---
    let res = client.post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network Error: {}", e))?;

    // --- FIX: Capture status code before consuming the response ---
    if !res.status().is_success() {
        let status = res.status(); // Capture status first
        let err_text = res.text().await.unwrap_or_default(); // Now consume the body
        return Err(format!("Gemini API Error ({}): {}", status, err_text));
    }

    // Since we didn't return an error, we now consume the body as JSON
    let json: GeminiResponse = res.json().await.map_err(|e| format!("JSON Decode Error: {}", e))?;
    
    // Check if candidates exist (Gemini sometimes returns empty if it blocks the prompt)
    if json.candidates.is_empty() || json.candidates[0].content.parts.is_empty() {
        return Ok("The Oracle is clouded. The AI refused to generate a response for this query.".to_string());
    }

    Ok(json.candidates[0].content.parts[0].text.trim().to_string())
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
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

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

#[tauri::command]
async fn export_journal(
    format: String, 
    entries: Vec<RawEntry>, 
    app: tauri::AppHandle
    ) -> Result<String, String> {   
    use tauri_plugin_dialog::DialogExt;

    let file_path = app.dialog()
        .file()
        .set_title("Select Export Location")
        .set_file_name(&format!("My_EchoJournal.{}", format))
        .blocking_save_file();

    let path = match file_path {
        Some(p) => p.as_path().unwrap().to_string_lossy().to_string(),
        None => return Ok("Export cancelled".into()),
    };

    if format == "pdf" {
        export::generate_pdf(entries, path)?;
    } else {
        export::generate_docx(entries, path)?;
    }

    Ok("Export Complete".into())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![refine_thought, is_date_locked, get_all_entries, ask_oracle, export_journal])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}