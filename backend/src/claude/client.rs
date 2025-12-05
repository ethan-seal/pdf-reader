use super::types::*;
use anyhow::Result;
use reqwest::Client;

pub struct ClaudeClient {
    client: Client,
    api_key: String,
    model: String,
}

impl ClaudeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: "claude-sonnet-4-5-20250929".to_string(),
        }
    }

    pub async fn chat(&self, request: ChatRequest) -> Result<ChatResponse> {
        let response = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "prompt-caching-2024-07-31")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Claude API error: {}", error_text);
        }

        Ok(response.json().await?)
    }

    /// Create a message with PDF document (with cache control for first message)
    pub fn create_pdf_message(&self, pdf_base64: String, text: String, enable_cache: bool) -> Message {
        let cache_control = if enable_cache {
            Some(super::types::CacheControl {
                cache_type: "ephemeral".to_string(),
            })
        } else {
            None
        };

        Message {
            role: "user".to_string(),
            content: vec![
                ContentBlock::Document {
                    source: DocumentSource {
                        source_type: "base64".to_string(),
                        media_type: "application/pdf".to_string(),
                        data: pdf_base64,
                    },
                    cache_control: cache_control.clone(),
                },
                ContentBlock::Text {
                    text,
                    cache_control: None,
                },
            ],
        }
    }

    /// Create a text-only message
    pub fn create_text_message(&self, role: &str, text: String) -> Message {
        Message {
            role: role.to_string(),
            content: vec![ContentBlock::Text {
                text,
                cache_control: None,
            }],
        }
    }
}
