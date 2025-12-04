use crate::claude::{ChatRequest, ClaudeClient, ResponseContent};
use crate::models::{ChatApiRequest, ChatApiResponse};
use crate::storage::FileStorage;
use axum::{extract::State, http::StatusCode, Json};
use std::sync::Arc;

pub struct AppState {
    pub claude: ClaudeClient,
    pub storage: Arc<dyn FileStorage>,
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
    // Get PDF from storage
    let pdf_base64 = state
        .storage
        .get_pdf_base64(&payload.document_id)
        .await
        .map_err(|e| (StatusCode::NOT_FOUND, format!("Document not found: {}", e)))?;

    let mut messages = Vec::new();

    // Build conversation history
    for (idx, msg) in payload.messages.iter().enumerate() {
        if idx == 0 && msg.role == "user" {
            // First message: include PDF
            messages.push(
                state
                    .claude
                    .create_pdf_message(pdf_base64.clone(), msg.content.clone()),
            );
        } else {
            // Subsequent messages: text only
            messages.push(state.claude.create_text_message(&msg.role, msg.content.clone()));
        }
    }

    let request = ChatRequest {
        model: "claude-sonnet-4-5-20250929".to_string(),
        max_tokens: 4096,
        messages,
        system: Some(SYSTEM_PROMPT.to_string()),
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

    Ok(Json(ChatApiResponse {
        response: text,
        usage: Some(response.usage),
    }))
}
