use fastembed::{TextEmbedding, InitOptions, EmbeddingModel};
use rusqlite::{params, Connection};
use std::path::Path;
use crate::RawEntry;

pub fn index_journal_entries(entries: Vec<RawEntry>, db_path: &Path) -> Result<usize, String> {
    // 1. Initialize the local model (100% offline)
    let model = TextEmbedding::try_new(
        InitOptions::new(EmbeddingModel::AllMiniLML6V2)
            .with_show_download_progress(true),
    ).map_err(|e| e.to_string())?;

    // 2. Open (or create) the local vector store
    let conn = Connection::open(db_path.join("echo_vectors.db"))
        .map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS journal_vectors (
            id TEXT PRIMARY KEY,
            date TEXT,
            content TEXT,
            embedding BLOB
        )",
        [],
    ).map_err(|e| e.to_string())?;

    let mut count = 0;
    for entry in entries {
        // Generate embedding locally
        let embeddings = model.embed(vec![&entry.text], None)
            .map_err(|e| e.to_string())?;
        
        let vector_blob: Vec<u8> = embeddings[0].iter()
            .flat_map(|f| f.to_le_bytes()).collect();

        conn.execute(
            "INSERT OR REPLACE INTO journal_vectors (id, date, content, embedding) VALUES (?1, ?2, ?3, ?4)",
            params![format!("{}-{}", entry.date, entry.time), entry.date, entry.text, vector_blob],
        ).map_err(|e| e.to_string())?;
        
        count += 1;
    }

    Ok(count)
}