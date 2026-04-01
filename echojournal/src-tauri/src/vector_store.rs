use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use rusqlite::{params, Connection};
use std::path::Path;
use crate::RawEntry;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::fs;

pub fn search_similar_entries(query: &str, db_path: &Path, top_k: usize) -> Result<Vec<String>, String> {
    // 1. Turn the query into a vector
    let query_vec = generate_embedding(query)?;

    // 2. Connect to the DB
    let conn = Connection::open(db_path.join("echo_vectors.db"))
        .map_err(|e| e.to_string())?;

    let mut stmt = conn.prepare("SELECT content, embedding FROM journal_vectors")
        .map_err(|e| e.to_string())?;

    let mut matches: Vec<(f32, String)> = stmt.query_map([], |row| {
        let content: String = row.get(0)?;
        let bytes: Vec<u8> = row.get(1)?;
        
        // Convert bytes back to Vec<f32>
        let vector: Vec<f32> = bytes.chunks_exact(4)
            .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        
        Ok((content, vector))
    }).map_err(|e| e.to_string())?
    .filter_map(|res| res.ok())
    .map(|(content, vector)| {
        let score = cosine_similarity(&query_vec, &vector);
        (score, content)
    })
    .collect();

    // 3. Sort by highest similarity and take top_k
    matches.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    
    Ok(matches.into_iter().take(top_k).map(|(_, content)| content).collect())
}

// Math helper for vector comparison
fn cosine_similarity(v1: &[f32], v2: &[f32]) -> f32 {
    let dot_product: f32 = v1.iter().zip(v2).map(|(a, b)| a * b).sum();
    let mag1: f32 = v1.iter().map(|a| a * a).sum::<f32>().sqrt();
    let mag2: f32 = v2.iter().map(|a| a * a).sum::<f32>().sqrt();
    if mag1 == 0.0 || mag2 == 0.0 { return 0.0; }
    dot_product / (mag1 * mag2)
}

pub fn sync_journal_to_db(entries: Vec<RawEntry>, db_path: &Path) -> Result<usize, String> {
    let model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::AllMiniLML6V2)
    ).map_err(|e| e.to_string())?;

    let conn = Connection::open(db_path.join("echo_vectors.db"))
        .map_err(|e| e.to_string())?;

    let mut new_indexed_count = 0;

    for entry in entries {
        let mut hasher = DefaultHasher::new();
        entry.text.hash(&mut hasher);
        let content_hash = hasher.finish();
        let entry_id = format!("{}-{}", entry.date, content_hash);

        let exists: bool = conn.query_row(
            "SELECT EXISTS(SELECT 1 FROM journal_vectors WHERE id = ?1)",
            params![entry_id],
            |row| row.get(0),
        ).unwrap_or(false);

        if !exists {
            // NEW: Enriched content string for better citations and search accuracy
            let full_content = format!("Date: {} | Tag: #{} | Entry: {}", entry.date, entry.tag, entry.text);

            // We embed the enriched string so the vector search accounts for the metadata
            let embeddings = model.embed(vec![&full_content], None)
                .map_err(|e| e.to_string())?;
            
            let vector_blob: Vec<u8> = embeddings[0].iter()
                .flat_map(|f| f.to_le_bytes()).collect();

            conn.execute(
                "INSERT OR REPLACE INTO journal_vectors (id, content, embedding) VALUES (?1, ?2, ?3)",
                params![entry_id, full_content, vector_blob],
            ).map_err(|e| e.to_string())?;
            
            new_indexed_count += 1;
        }
    }

    Ok(new_indexed_count)
}

pub fn generate_embedding(text: &str) -> Result<Vec<f32>, String> {
    // Initialize the model (downloads ~20MB on first run, then 100% offline)
    let model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::AllMiniLML6V2)
            .with_show_download_progress(true),
    ).map_err(|e| e.to_string())?;

    let embeddings = model.embed(vec![text], None)
        .map_err(|e| e.to_string())?;
    
    Ok(embeddings[0].clone())
}

pub fn init_vector_db(db_path: &Path) -> Result<(), String> {
    if !db_path.exists() {
        fs::create_dir_all(db_path).map_err(|e| format!("Failed to create AppData directory: {}", e))?;
    }

    let full_path = db_path.join("echo_vectors.db");
    
    // 2. Open the connection
    let conn = Connection::open(full_path)
        .map_err(|e| format!("SQLite Open Error: {}", e))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS journal_vectors (
            id TEXT PRIMARY KEY,
            content TEXT,
            embedding BLOB
        )",
        [],
    ).map_err(|e| e.to_string())?;

    Ok(())
}