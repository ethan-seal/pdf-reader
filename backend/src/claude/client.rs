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

    /// Extract keywords and topics from a PDF document
    pub async fn extract_metadata(&self, pdf_base64: String) -> Result<super::types::MetadataExtractionResponse> {
        let message = self.create_pdf_message(
            pdf_base64,
            "Extract keywords and topics from this PDF document. Analyze the content and return ONLY a valid JSON object with this exact format: {\"keywords\": [\"keyword1\", \"keyword2\", ...], \"topics\": [\"topic1\", \"topic2\", ...]}. Provide 5-10 relevant keywords and 3-5 main topics. No additional text, just the JSON.".to_string(),
            false,
        );

        let request = super::types::ChatRequest {
            model: self.model.clone(),
            max_tokens: 1024,
            messages: vec![message],
            system: None,
        };

        let response = self.chat(request).await?;

        // Extract text from response
        if let Some(super::types::ResponseContent::Text { text }) = response.content.first() {
            // Try to parse JSON, handling possible markdown code blocks
            let json_text = text.trim()
                .trim_start_matches("```json")
                .trim_start_matches("```")
                .trim_end_matches("```")
                .trim();

            let metadata: super::types::MetadataExtractionResponse = serde_json::from_str(json_text)
                .map_err(|e| anyhow::anyhow!("Failed to parse metadata JSON: {}. Response was: {}", e, text))?;

            Ok(metadata)
        } else {
            anyhow::bail!("No text content in response")
        }
    }
}
