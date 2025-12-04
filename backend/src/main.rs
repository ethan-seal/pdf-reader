mod api;
mod claude;
mod models;
mod storage;

use crate::api::{chat_handler, get_document_handler, upload_handler, AppState};
use crate::claude::ClaudeClient;
use crate::storage::{FileStorage, LocalStorage};
use axum::{routing::*, Router};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let api_key =
        std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY environment variable must be set");

    let storage: Arc<dyn FileStorage> =
        Arc::new(LocalStorage::new("./uploads").expect("Failed to create storage"));

    let state = Arc::new(AppState {
        claude: ClaudeClient::new(api_key),
        storage: storage.clone(),
    });

    let app = Router::new()
        .route("/api/upload", post(upload_handler))
        .route("/api/chat", post(chat_handler))
        .route("/api/documents/:id", get(get_document_handler))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .unwrap();

    println!("Server running on http://localhost:3001");

    axum::serve(listener, app).await.unwrap();
}
