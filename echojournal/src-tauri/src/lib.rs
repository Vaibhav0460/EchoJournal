use dotenvy::dotenv;
use std::env;
use serde::{Deserialize, Serialize};
use chrono::{Local, NaiveDate};
mod tags;
pub mod export;
mod vector_store;
use tauri::Manager;
use notify::{Watcher, RecursiveMode, Config, EventKind};
mod oracle_engine; // Declare the new module
use crate::oracle_engine::OracleEngine;
use std::sync::{Arc, Mutex};
use tauri::State;

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

#[derive(serde::Deserialize, serde::Serialize)]
struct RefinedEntry {
    tag: String,
    content: String,
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
async fn ask_oracle(question: String, app_handle: tauri::AppHandle, use_local: bool) -> Result<String, String> {
    use tauri::Manager;
    dotenv().ok();
    let db_path = app_handle.path().app_data_dir().unwrap();
    let relevant_entries = crate::vector_store::search_similar_entries(&question, &db_path, 5)?;
    if relevant_entries.is_empty() {
        return Ok("Your memory is currently blank. Try syncing your journal before consulting the Oracle.".into());
    }
    let context_data = relevant_entries.join("\n---\n");
    let prompt = format!(
        "SYSTEM: You are the 'Echo Oracle'. Answer using ONLY the provided journal data. 
         CITATIONS: Always cite the entry using the format (YYYY-MM-DD, #Tag).
         DATA: {}
         USER QUESTION: {}",
        context_data, question
    );

    if use_local {
        // Fix for Error #4: Explicitly telling Rust what 'engine_state' is
        let engine_state: State<'_, Arc<Mutex<OracleEngine>>> = app_handle.state();
        
        let engine = engine_state.lock().map_err(|e| e.to_string())?;
        // We pass the prompt we just built
        return engine.generate_response(&prompt, &question);
    }

    // --- Gemini Path ---
    let api_key = env::var("GEMINI_API_KEY").map_err(|_| "Key not found".to_string())?;
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
        api_key
    );

    let body = GeminiRequest {
        contents: vec![Content { parts: vec![Part { text: prompt }] }], // Now 'prompt' exists!
    };

    let client = reqwest::Client::new();
    let res = client.post(&url).json(&body).send().await.map_err(|e| format!("Network Error: {}", e))?;

    if !res.status().is_success() {
        let status = res.status();
        let err_text = res.text().await.unwrap_or_default();
        return Err(format!("Gemini API Error ({}): {}", status, err_text));
    }

    let json: GeminiResponse = res.json().await.map_err(|e| format!("JSON Decode Error: {}", e))?;
    
    if json.candidates.is_empty() || json.candidates[0].content.parts.is_empty() {
        return Ok("The Oracle is clouded.".to_string());
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

#[tauri::command]
fn is_date_locked(target_date_str: Option<String>, date: Option<String>) -> Result<bool, String> {
    let now = Local::now().date_naive();
    let date_str = target_date_str
        .or(date)
        .ok_or_else(|| "Missing date".to_string())?;
    let target_date = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .map_err(|_| "Invalid date format".to_string())?;
    let diff = now.signed_duration_since(target_date).num_days();
    Ok(diff > 7 || diff < 0)
}

#[tauri::command]
async fn refine_thought(thought: String, app_handle: tauri::AppHandle, use_local: bool) -> Result<RefinedEntry, String> {
    let system_prompt = "Refine this thought into a journal entry.\nReturn ONLY valid JSON (no markdown, no extra text) in this format:\n{\"tag\":\"Category\",\"content\":\"Refinedtext\"}\nMake sure the response ends with a closing '}' character.";
    let raw_json = if use_local {
        // --- Local Path ---
        let engine_state: tauri::State<'_, Arc<Mutex<OracleEngine>>> = app_handle.state();
        let engine = engine_state.lock().map_err(|e| e.to_string())?;
        engine.generate_response(system_prompt, &thought)?
    } else {
        // --- Gemini Path ---
        dotenvy::dotenv().ok();
        let api_key = std::env::var("GEMINI_API_KEY").map_err(|_| "Key not found".to_string())?;
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
            api_key
        );

        let body = GeminiRequest {
            contents: vec![Content { parts: vec![Part { text: format!("{}\n\nThought: {}", system_prompt, thought) }] }],
        };

        let client = reqwest::Client::new();
        let res = client.post(&url).json(&body).send().await.map_err(|e| e.to_string())?;
        let json_res: GeminiResponse = res.json().await.map_err(|e| e.to_string())?;
        
        json_res.candidates.get(0)
            .and_then(|c| c.content.parts.get(0))
            .map(|p| p.text.clone())
            .ok_or("Gemini returned an empty response")?
    };

    // Clean and robustly parse (local LLMs sometimes truncate the final brace).
    let cleaned = raw_json
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let try_parse = |s: &str| serde_json::from_str::<RefinedEntry>(s);

    if let Ok(v) = try_parse(cleaned) {
        return Ok(v);
    }

    // Attempt 2: extract JSON object region.
    if let Some(start) = cleaned.find('{') {
        let slice = &cleaned[start..];
        if let Some(end) = slice.rfind('}') {
            let candidate = &slice[..=end];
            if let Ok(v) = try_parse(candidate) {
                return Ok(v);
            }
        } else {
            // Attempt 3: missing closing brace; append one.
            let mut candidate = slice.to_string();
            candidate.push('}');
            if let Ok(v) = try_parse(&candidate) {
                return Ok(v);
            }
        }
    }

    Err(format!(
        "JSON Parse Error: model did not return valid JSON. Raw output: {}",
        cleaned
    ))
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
    
            // We will place the model in the app_data_dir/models folder
            let app_data = app.path().app_data_dir().unwrap();
            let model_path = app_data.join("models").join("oracle-model.gguf");

            if model_path.exists() {
                if let Ok(engine) = OracleEngine::new(&model_path) {
                    app.manage(Arc::new(Mutex::new(engine)));
                }
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![refine_thought, is_date_locked, get_all_entries, ask_oracle, export_journal, sync_vectors])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}