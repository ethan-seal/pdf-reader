use crate::api::AppState;
use crate::api::metadata::extract_and_save_metadata;
use crate::error::ApiError;
use axum::{
    extract::{Multipart, State},
    Json,
};
use serde::Serialize;
use std::sync::Arc;

#[derive(Serialize)]
pub struct UploadResponse {
    pub document_id: String,
}

pub async fn upload_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<UploadResponse>, ApiError> {
    let storage = &state.storage;
    let chat_db = &state.chat_db;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError::BadRequest(format!("Invalid multipart data: {}", e)))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "pdf" {
            let filename = field.file_name().unwrap_or("document.pdf").to_string();

            let data = field
                .bytes()
                .await
                .map_err(|e| ApiError::BadRequest(format!("Failed to read file data: {}", e)))?;

            let document_id = storage
                .store_pdf(&filename, data)
                .await
                .map_err(|e| ApiError::StorageError(e.to_string()))?;

            // Create document record in database
            chat_db
                .create_document(&document_id, &filename)
                .await
                .map_err(|e| ApiError::DatabaseError(e.to_string()))?;

            // Extract metadata in background (don't block upload response)
            let state_clone = state.clone();
            let doc_id = document_id.clone();
            tokio::spawn(async move {
                if let Err(e) = extract_and_save_metadata(&state_clone, &doc_id).await {
                    eprintln!("Failed to extract metadata for {}: {}", doc_id, e);
                }
            });

            return Ok(Json(UploadResponse { document_id }));
        }
    }

    Err(ApiError::BadRequest(
        "No PDF file found in request".to_string(),
    ))
}
