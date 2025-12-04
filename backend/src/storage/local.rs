use super::r#trait::*;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use bytes::Bytes;
use std::path::{Path, PathBuf};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub struct LocalStorage {
    base_path: PathBuf,
}

impl LocalStorage {
    pub fn new<P: AsRef<Path>>(base_path: P) -> StorageResult<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        std::fs::create_dir_all(&base_path)?;
        std::fs::create_dir_all(base_path.join("pdfs"))?;
        std::fs::create_dir_all(base_path.join("metadata"))?;

        Ok(Self { base_path })
    }

    fn pdf_path(&self, document_id: &str) -> PathBuf {
        self.base_path.join("pdfs").join(format!("{}.pdf", document_id))
    }

    fn metadata_path(&self, document_id: &str) -> PathBuf {
        self.base_path.join("metadata").join(format!("{}.json", document_id))
    }
}

#[async_trait]
impl FileStorage for LocalStorage {
    async fn store_pdf(&self, _filename: &str, data: Bytes) -> StorageResult<String> {
        // Validate PDF header
        if data.len() < 4 || &data[..4] != b"%PDF" {
            return Err(StorageError::InvalidFormat);
        }

        // Generate document ID
        let document_id = uuid::Uuid::new_v4().to_string();
        let path = self.pdf_path(&document_id);

        let mut file = fs::File::create(&path).await?;
        file.write_all(&data).await?;

        Ok(document_id)
    }

    async fn get_pdf(&self, document_id: &str) -> StorageResult<Bytes> {
        let path = self.pdf_path(document_id);

        if !path.exists() {
            return Err(StorageError::NotFound(document_id.to_string()));
        }

        let mut file = fs::File::open(&path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        Ok(Bytes::from(buffer))
    }

    async fn exists(&self, document_id: &str) -> StorageResult<bool> {
        Ok(self.pdf_path(document_id).exists())
    }

    async fn delete(&self, document_id: &str) -> StorageResult<()> {
        let pdf_path = self.pdf_path(document_id);
        let metadata_path = self.metadata_path(document_id);

        if pdf_path.exists() {
            fs::remove_file(pdf_path).await?;
        }

        if metadata_path.exists() {
            fs::remove_file(metadata_path).await?;
        }

        Ok(())
    }

    async fn get_pdf_base64(&self, document_id: &str) -> StorageResult<String> {
        let data = self.get_pdf(document_id).await?;
        Ok(BASE64.encode(&data))
    }

    async fn store_metadata(&self, document_id: &str, metadata: &[u8]) -> StorageResult<()> {
        let path = self.metadata_path(document_id);
        let mut file = fs::File::create(&path).await?;
        file.write_all(metadata).await?;
        Ok(())
    }

    async fn get_metadata(&self, document_id: &str) -> StorageResult<Option<Vec<u8>>> {
        let path = self.metadata_path(document_id);

        if !path.exists() {
            return Ok(None);
        }

        let mut file = fs::File::open(&path).await?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).await?;

        Ok(Some(buffer))
    }
}
