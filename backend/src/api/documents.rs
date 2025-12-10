use crate::api::AppState;
use crate::error::ApiError;
use axum::{
    extract::{Path, State},
    http::header,
    response::IntoResponse,
};
use std::sync::Arc;

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
