use dotenvy::dotenv;
use std::env;
use serde::{Deserialize, Serialize};
use tauri::command;
use tokio::time::{sleep, Duration};
use chrono::{Local, Datelike, NaiveDate};
mod tags;
pub mod export;
mod vector_store;
use tauri::Manager;
use notify::{Watcher, RecursiveMode, Config, EventKind};
use std::path::Path;

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

fn setup_file_watcher(app_handle: tauri::AppHandle) {
    let handle = app_handle.clone();
    
    // Run the watcher in a separate thread so it doesn't block the UI
    std::thread::spawn(move || {
        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = notify::RecommendedWatcher::new(tx, Config::default())
            .expect("Failed to create watcher");

        // Resolve the journal path (same logic as your frontend)
        let docs_path = handle.path().document_dir().unwrap();
        let journal_path = docs_path.join("EchoJournal");

        // Start watching the folder
        watcher.watch(&journal_path, RecursiveMode::Recursive).ok();

        println!("Watcher: Monitoring {} for changes...", journal_path.display());

        for res in rx {
            match res {
                Ok(event) => {
                    if let EventKind::Modify(_) = event.kind {
                        println!("Watcher: Change detected, re-syncing...");
                        
                        let h = handle.clone();
                        // CLONE the path here so the closure has its own instance
                        let path_for_task = journal_path.clone(); 
                        
                        tauri::async_runtime::spawn(async move {
                            let path_str = path_for_task.to_string_lossy().to_string();
                            let entries = get_all_entries(path_str).await.unwrap_or_default();
                            
                            let db_path = h.path().app_data_dir().unwrap();
                            let _ = crate::vector_store::sync_journal_to_db(entries, &db_path);
                        });
                    }
                }
                Err(e) => println!("Watcher Error: {:?}", e),
            }
        }
    });
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
async fn ask_oracle(question: String, app_handle: tauri::AppHandle) -> Result<String, String> {
    use tauri::Manager;
    dotenv().ok();
    
    // 1. Auth check
    let api_key = env::var("GEMINI_API_KEY").map_err(|_| "GEMINI_API_KEY not found in .env".to_string())?;

    // 2. Local Semantic Search (The "Retrieval" part of RAG)
    let db_path = app_handle.path().app_data_dir().unwrap();
    let relevant_entries = crate::vector_store::search_similar_entries(&question, &db_path, 5)?;
    
    if relevant_entries.is_empty() {
        return Ok("Your memory is currently blank. Try syncing your journal before consulting the Oracle.".into());
    }

    // 3. Construct the Augmented Prompt
    let context_data = relevant_entries.join("\n---\n");
    let prompt = format!(
        "SYSTEM: You are the 'Echo Oracle'. Answer using ONLY the provided journal data. 
         CITATIONS: Always cite the entry using the format (YYYY-MM-DD, #Tag).
         DATA: {}
         USER QUESTION: {}",
        context_data, question
    );

    // 4. API Configuration (Using 1.5-flash for stability)
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash:generateContent?key={}",
        api_key
    );

    let body = GeminiRequest {
        contents: vec![Content { parts: vec![Part { text: prompt }] }],
    };

    // 5. Single Network Call
    let client = reqwest::Client::new();
    let res = client.post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network Error: {}", e))?;

    // 6. Robust Error Handling
    if !res.status().is_success() {
        let status = res.status();
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Gemini API Error ({}): {}", status, err_text));
    }

    let json: GeminiResponse = res.json().await.map_err(|e| format!("JSON Decode Error: {}", e))?;
    
    if json.candidates.is_empty() || json.candidates[0].content.parts.is_empty() {
        return Ok("The Oracle is clouded. No candidates were returned for this inquiry.".to_string());
    }

    Ok(json.candidates[0].content.parts[0].text.trim().to_string())
}

#[tauri::command]
async fn sync_vectors(journal_path: String, app_handle: tauri::AppHandle) -> Result<String, String> {
    let entries = get_all_entries(journal_path.clone()).await?;
    let db_path = app_handle.path().app_data_dir().unwrap();
    
    // Ensure the DB exists first
    crate::vector_store::init_vector_db(&db_path)?;
    
    let count = crate::vector_store::sync_journal_to_db(entries, &db_path)?;
    
    if count > 0 {
        Ok(format!("Successfully indexed {} new memories.", count))
    } else {
        Ok("Journal is already up to date.".into())
    }
}

// Simple math for vector comparison
fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    let dot_product: f32 = v1.iter().zip(v2).map(|(a, b)| a * b).sum();
    let mag1: f32 = v1.iter().map(|a| a * a).sum::<f32>().sqrt();
    let mag2: f32 = v2.iter().map(|a| a * a).sum::<f32>().sqrt();
    dot_product / (mag1 * mag2)
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
        .setup(|app| {
            setup_file_watcher(app.handle().clone());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![refine_thought, is_date_locked, get_all_entries, ask_oracle, export_journal, sync_vectors])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}