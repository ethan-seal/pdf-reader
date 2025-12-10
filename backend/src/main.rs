mod api;
mod claude;
mod db;
mod models;
mod storage;

use crate::api::{chat_handler, get_chat_history_handler, get_document_handler, upload_handler, AppState};
use crate::claude::ClaudeClient;
use crate::db::{initialize_database, ChatDatabase};
use crate::storage::{FileStorage, LocalStorage};
use axum::{routing::*, Router};
use moka::future::Cache;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    let api_key =
        std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY environment variable must be set");

    let storage: Arc<dyn FileStorage> =
        Arc::new(LocalStorage::new("./uploads").expect("Failed to create storage"));

    // Initialize SQLite database
    let db_pool = initialize_database("sqlite:chat_history.db?mode=rwc")
        .await
        .expect("Failed to initialize database");
    let chat_db = ChatDatabase::new(db_pool);

    // Create cache for base64-encoded PDFs
    // Cache up to 100 PDFs, TTL of 1 hour
    let pdf_cache = Cache::builder()
        .max_capacity(100)
        .time_to_live(Duration::from_secs(3600))
        .build();

    let state = Arc::new(AppState {
        claude: ClaudeClient::new(api_key),
        storage: storage.clone(),
        pdf_cache,
        chat_db,
    });

    let app = Router::new()
        .route("/api/upload", post(upload_handler))
        .route("/api/chat", post(chat_handler))
        .route("/api/chat/history/:document_id", get(get_chat_history_handler))
        .route("/api/documents/:id", get(get_document_handler))
        .with_state(state)
        .layer(CorsLayer::permissive());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001")
        .await
        .unwrap();

    println!("Server running on http://localhost:3001");

    axum::serve(listener, app).await.unwrap();
}
