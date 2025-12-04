use async_trait::async_trait;
use bytes::Bytes;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("File not found: {0}")]
    NotFound(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid file format")]
    InvalidFormat,

    #[error("Storage error: {0}")]
    Other(String),
}

pub type StorageResult<T> = Result<T, StorageError>;

#[async_trait]
pub trait FileStorage: Send + Sync {
    /// Store a PDF file and return its document ID
    async fn store_pdf(&self, filename: &str, data: Bytes) -> StorageResult<String>;

    /// Retrieve a PDF file by document ID
    async fn get_pdf(&self, document_id: &str) -> StorageResult<Bytes>;

    /// Check if a document exists
    async fn exists(&self, document_id: &str) -> StorageResult<bool>;

    /// Delete a document
    async fn delete(&self, document_id: &str) -> StorageResult<()>;

    /// Get the base64 encoded PDF (for Claude API)
    async fn get_pdf_base64(&self, document_id: &str) -> StorageResult<String>;

    /// Store metadata (for caching conversation state)
    async fn store_metadata(&self, document_id: &str, metadata: &[u8]) -> StorageResult<()>;

    /// Get metadata
    async fn get_metadata(&self, document_id: &str) -> StorageResult<Option<Vec<u8>>>;
}
