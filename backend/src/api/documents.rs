use crate::api::AppState;
use crate::error::ApiError;
use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct ListDocumentsQuery {
    #[serde(default = "default_limit")]
    limit: i32,
}

fn default_limit() -> i32 {
    20
}

#[derive(Debug, Serialize)]
pub struct DocumentWithMetadata {
    pub id: String,
    pub filename: String,
    pub keywords: Vec<String>,
    pub topics: Vec<String>,
    pub uploaded_at: String,
}

pub async fn get_document_handler(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    let pdf_data = state
        .storage
        .get_pdf(&document_id)
        .await
        .map_err(|e| ApiError::NotFound(format!("Document not found: {}", e)))?;

    Ok((
        [(header::CONTENT_TYPE, "application/pdf")],
        pdf_data.to_vec(),
    ))
}

pub async fn list_documents_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListDocumentsQuery>,
) -> Result<Json<Vec<DocumentWithMetadata>>, (StatusCode, String)> {
    let documents = state
        .chat_db
        .list_recent_documents(params.limit)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    let documents_with_metadata = documents
        .into_iter()
        .map(|doc| {
            let keywords = doc
                .keywords
                .as_ref()
                .and_then(|k| serde_json::from_str(k).ok())
                .unwrap_or_else(Vec::new);

            let topics = doc
                .topics
                .as_ref()
                .and_then(|t| serde_json::from_str(t).ok())
                .unwrap_or_else(Vec::new);

            DocumentWithMetadata {
                id: doc.id,
                filename: doc.filename,
                keywords,
                topics,
                uploaded_at: doc.uploaded_at,
            }
        })
        .collect();

    Ok(Json(documents_with_metadata))
}
