use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct StoredMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub created_at: String,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Document {
    pub id: String,
    pub filename: String,
    pub keywords: Option<String>,  // JSON array stored as TEXT
    pub topics: Option<String>,    // JSON array stored as TEXT
    pub uploaded_at: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct ChatDatabase {
    pool: SqlitePool,
}

impl ChatDatabase {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn get_or_create_conversation(&self, document_id: &str) -> Result<String, sqlx::Error> {
        // Check if conversation exists
        let existing: Option<(String,)> = sqlx::query_as(
            "SELECT id FROM conversations WHERE document_id = ? ORDER BY created_at DESC LIMIT 1",
        )
        .bind(document_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some((conversation_id,)) = existing {
            // Update the updated_at timestamp
            let now = Utc::now().to_rfc3339();
            sqlx::query("UPDATE conversations SET updated_at = ? WHERE id = ?")
                .bind(&now)
                .bind(&conversation_id)
                .execute(&self.pool)
                .await?;
            Ok(conversation_id)
        } else {
            // Create new conversation
            let conversation_id = Uuid::new_v4().to_string();
            let now = Utc::now().to_rfc3339();

            sqlx::query(
                "INSERT INTO conversations (id, document_id, created_at, updated_at) VALUES (?, ?, ?, ?)",
            )
            .bind(&conversation_id)
            .bind(document_id)
            .bind(&now)
            .bind(&now)
            .execute(&self.pool)
            .await?;

            Ok(conversation_id)
        }
    }

    pub async fn save_message(
        &self,
        conversation_id: &str,
        role: &str,
        content: &str,
    ) -> Result<String, sqlx::Error> {
        let message_id = Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO chat_messages (id, conversation_id, role, content, created_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&message_id)
        .bind(conversation_id)
        .bind(role)
        .bind(content)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        Ok(message_id)
    }

    pub async fn get_conversation_messages(
        &self,
        document_id: &str,
    ) -> Result<Vec<StoredMessage>, sqlx::Error> {
        let messages: Vec<StoredMessage> = sqlx::query_as(
            r#"
            SELECT m.id, m.role, m.content, m.created_at
            FROM chat_messages m
            JOIN conversations c ON m.conversation_id = c.id
            WHERE c.document_id = ?
            ORDER BY m.created_at ASC
            "#,
        )
        .bind(document_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(messages)
    }

    pub async fn create_document(
        &self,
        document_id: &str,
        filename: &str,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            INSERT INTO documents (id, filename, keywords, topics, uploaded_at, created_at, updated_at)
            VALUES (?, ?, NULL, NULL, ?, ?, ?)
            "#,
        )
        .bind(document_id)
        .bind(filename)
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_document(&self, document_id: &str) -> Result<Option<Document>, sqlx::Error> {
        let document: Option<Document> = sqlx::query_as(
            "SELECT id, filename, keywords, topics, uploaded_at, created_at, updated_at FROM documents WHERE id = ?",
        )
        .bind(document_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(document)
    }

    pub async fn update_document_metadata(
        &self,
        document_id: &str,
        keywords: Option<&str>,
        topics: Option<&str>,
    ) -> Result<(), sqlx::Error> {
        let now = Utc::now().to_rfc3339();

        sqlx::query(
            r#"
            UPDATE documents
            SET keywords = ?, topics = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(keywords)
        .bind(topics)
        .bind(&now)
        .bind(document_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn list_recent_documents(&self, limit: i32) -> Result<Vec<Document>, sqlx::Error> {
        let documents: Vec<Document> = sqlx::query_as(
            r#"
            SELECT id, filename, keywords, topics, uploaded_at, created_at, updated_at
            FROM documents
            ORDER BY uploaded_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        Ok(documents)
    }
}
