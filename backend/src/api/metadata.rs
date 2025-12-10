use crate::api::AppState;
use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use std::time::Duration;

#[derive(Serialize)]
pub struct BackfillResponse {
    pub processed: usize,
    pub succeeded: usize,
    pub failed: usize,
}

/// Backfill metadata for existing PDFs without keywords/topics
pub async fn backfill_metadata_handler(
    State(state): State<Arc<AppState>>,
) -> Result<Json<BackfillResponse>, (StatusCode, String)> {
    let result = backfill_metadata(&state).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}

/// Extract keywords and topics from a PDF and save to database
pub async fn extract_and_save_metadata(state: &Arc<AppState>, document_id: &str) -> anyhow::Result<()> {
    // Get PDF as base64
    let pdf_base64 = state.storage.get_pdf_base64(document_id).await?;

    // Extract metadata using Claude
    let metadata = state.claude.extract_metadata(pdf_base64).await?;

    // Save to database as JSON arrays
    let keywords_json = serde_json::to_string(&metadata.keywords)?;
    let topics_json = serde_json::to_string(&metadata.topics)?;

    state.chat_db.update_document_metadata(
        document_id,
        Some(&keywords_json),
        Some(&topics_json),
    ).await?;

    println!("Extracted metadata for {}: {} keywords, {} topics",
        document_id, metadata.keywords.len(), metadata.topics.len());

    Ok(())
}

/// Backfill metadata for all documents that don't have it yet
pub async fn backfill_metadata(state: &Arc<AppState>) -> anyhow::Result<BackfillResponse> {
    // Get all documents
    let documents = state.chat_db.list_recent_documents(1000).await?;

    let mut processed = 0;
    let mut succeeded = 0;
    let mut failed = 0;

    for doc in documents {
        // Skip documents that already have metadata
        if doc.keywords.is_some() && doc.topics.is_some() {
            continue;
        }

        processed += 1;
        println!("Processing document {} ({})...", doc.id, doc.filename);

        // Retry with exponential backoff
        match retry_with_backoff(|| extract_and_save_metadata(state, &doc.id), 3).await {
            Ok(_) => {
                succeeded += 1;
                println!("Successfully processed {}", doc.id);
            }
            Err(e) => {
                failed += 1;
                eprintln!("Failed to process {} after retries: {}", doc.id, e);
            }
        }
    }

    Ok(BackfillResponse {
        processed,
        succeeded,
        failed,
    })
}

/// Retry a function with exponential backoff
async fn retry_with_backoff<F, Fut, T>(
    mut f: F,
    max_retries: u32,
) -> anyhow::Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = anyhow::Result<T>>,
{
    let mut retry_count = 0;
    let mut delay = Duration::from_secs(1);

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                retry_count += 1;
                if retry_count >= max_retries {
                    return Err(e);
                }

                eprintln!("Attempt {} failed: {}. Retrying in {:?}...", retry_count, e, delay);
                tokio::time::sleep(delay).await;

                // Exponential backoff: double the delay each time
                delay *= 2;
            }
        }
    }
}
