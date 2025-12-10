use crate::api::AppState;
use axum::{
    extract::{Multipart, State},
    http::StatusCode,
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
) -> Result<Json<UploadResponse>, (StatusCode, String)> {
    let storage = &state.storage;
    let chat_db = &state.chat_db;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?
    {
        let name = field.name().unwrap_or("").to_string();

        if name == "pdf" {
            let filename = field.file_name().unwrap_or("document.pdf").to_string();

            let data = field
                .bytes()
                .await
                .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

            let document_id = storage
                .store_pdf(&filename, data)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            // Create document record in database
            chat_db
                .create_document(&document_id, &filename)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            return Ok(Json(UploadResponse { document_id }));
        }
    }

    Err((
        StatusCode::BAD_REQUEST,
        "No PDF file found".to_string(),
    ))
}
