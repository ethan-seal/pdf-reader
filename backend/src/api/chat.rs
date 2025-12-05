use crate::claude::{ChatRequest, ClaudeClient, ResponseContent, SystemBlock};
use crate::db::{ChatDatabase, StoredMessage};
use crate::models::{ChatApiRequest, ChatApiResponse};
use crate::storage::FileStorage;
use axum::{extract::{Path, State}, http::StatusCode, Json};
use moka::future::Cache;
use std::sync::Arc;

pub struct AppState {
    pub claude: ClaudeClient,
    pub storage: Arc<dyn FileStorage>,
    pub pdf_cache: Cache<String, String>, // document_id -> base64
    pub chat_db: ChatDatabase,
}

const SYSTEM_PROMPT: &str = r#"You are an AI assistant helping users understand research papers.

Guidelines:
- CRITICAL: Always format page references using EXACTLY this format: (page X) for single pages or (page X, page Y) for multiple pages
- ONLY state information you can actually find in the PDF content
- NEVER make assumptions or educated guesses
- If you cannot find specific information, clearly state "I cannot find this information in the paper"
- Use markdown formatting for better readability
- Be concise and clear in your explanations"#;

pub async fn chat_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<ChatApiRequest>,
) -> Result<Json<ChatApiResponse>, (StatusCode, String)> {
    // Get or create conversation for this document
    let conversation_id = state
        .chat_db
        .get_or_create_conversation(&payload.document_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    // Get PDF from cache or storage
    let pdf_base64 = match state.pdf_cache.get(&payload.document_id).await {
        Some(cached) => cached,
        None => {
            // Not in cache, fetch from storage and encode
            let base64 = state
                .storage
                .get_pdf_base64(&payload.document_id)
                .await
                .map_err(|e| (StatusCode::NOT_FOUND, format!("Document not found: {}", e)))?;

            // Store in cache for future requests
            state.pdf_cache.insert(payload.document_id.clone(), base64.clone()).await;
            base64
        }
    };

    let mut messages = Vec::new();

    // Build conversation history
    for (idx, msg) in payload.messages.iter().enumerate() {
        if idx == 0 && msg.role == "user" {
            // First message: include PDF with cache control enabled
            messages.push(
                state
                    .claude
                    .create_pdf_message(pdf_base64.clone(), msg.content.clone(), true),
            );
        } else {
            // Subsequent messages: text only
            messages.push(state.claude.create_text_message(&msg.role, msg.content.clone()));
        }
    }

    // Create system prompt with cache control
    let system = Some(vec![SystemBlock {
        block_type: "text".to_string(),
        text: SYSTEM_PROMPT.to_string(),
        cache_control: Some(crate::claude::types::CacheControl {
            cache_type: "ephemeral".to_string(),
        }),
    }]);

    let request = ChatRequest {
        model: "claude-sonnet-4-5-20250929".to_string(),
        max_tokens: 4096,
        messages,
        system,
    };

    let response = state.claude.chat(request).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Claude API error: {}", e),
        )
    })?;

    // Extract text from response
    let text = response
        .content
        .iter()
        .filter_map(|c| match c {
            ResponseContent::Text { text } => Some(text.clone()),
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Save the user message and assistant response to database
    // Get the last user message from the payload
    if let Some(last_user_msg) = payload.messages.last() {
        if last_user_msg.role == "user" {
            state
                .chat_db
                .save_message(&conversation_id, "user", &last_user_msg.content)
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;
        }
    }

    // Save assistant response
    state
        .chat_db
        .save_message(&conversation_id, "assistant", &text)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    Ok(Json(ChatApiResponse {
        response: text,
        usage: Some(response.usage),
    }))
}

pub async fn get_chat_history_handler(
    State(state): State<Arc<AppState>>,
    Path(document_id): Path<String>,
) -> Result<Json<Vec<StoredMessage>>, (StatusCode, String)> {
    let messages = state
        .chat_db
        .get_conversation_messages(&document_id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    Ok(Json(messages))
}
