use std::sync::Arc;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{info, debug, warn, error};
use sha2::{Sha256, Digest};

use crate::global::{
    database::DatabaseInstance,
    error::AppError,
    queue::{Task, TaskPriority, TaskData, TaskStatus},
};
use super::model::{PictureMetadata, PictureStatus};
use super::database;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchPicturePayload {
    pub url: String,
    pub storage_path: String,
    pub filename: Option<String>,
    pub tags: Vec<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
}

pub struct FetchPictureTask {
    id: String,
    url: String,
    storage_path: PathBuf,
    filename: Option<String>,
    tags: Vec<String>,
    entity_type: Option<String>,
    entity_id: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl FetchPictureTask {
    pub fn new(
        url: String,
        storage_path: PathBuf,
        filename: Option<String>,
    ) -> Self {
        let id = format!("fetch_picture_{}", uuid::Uuid::new_v4());
        Self {
            id,
            url,
            storage_path,
            filename,
            tags: Vec::new(),
            entity_type: None,
            entity_id: None,
            created_at: chrono::Utc::now(),
        }
    }
    
    /// Add tags for categorization
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
    
    /// Associate with an entity
    pub fn with_entity(mut self, entity_type: String, entity_id: String) -> Self {
        self.entity_type = Some(entity_type);
        self.entity_id = Some(entity_id);
        self
    }

    /// Extract filename from URL or use provided filename
    fn get_filename(&self) -> String {
        if let Some(ref name) = self.filename {
            return name.clone();
        }

        // Try to extract from URL
        self.url
            .split('/')
            .last()
            .and_then(|s| s.split('?').next())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("image_{}.jpg", uuid::Uuid::new_v4()))
    }

    /// Sanitize filename to prevent path traversal
    fn sanitize_filename(filename: &str) -> String {
        filename
            .chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                _ => c,
            })
            .collect()
    }
    
    /// Calculate SHA-256 hash of file content
    fn calculate_hash(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        format!("{:x}", hasher.finalize())
    }
    
    /// Detect MIME type from file extension
    fn detect_mime_type(filename: &str) -> Option<String> {
        let extension = filename.split('.').last()?.to_lowercase();
        match extension.as_str() {
            "jpg" | "jpeg" => Some("image/jpeg".to_string()),
            "png" => Some("image/png".to_string()),
            "gif" => Some("image/gif".to_string()),
            "webp" => Some("image/webp".to_string()),
            "bmp" => Some("image/bmp".to_string()),
            "svg" => Some("image/svg+xml".to_string()),
            _ => None,
        }
    }
}

#[async_trait::async_trait]
impl Task for FetchPictureTask {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> &str {
        "fetch_picture"
    }

    fn priority(&self) -> TaskPriority {
        TaskPriority::Low
    }

    fn to_data(&self) -> TaskData {
        let payload = FetchPicturePayload {
            url: self.url.clone(),
            storage_path: self.storage_path.to_string_lossy().to_string(),
            filename: self.filename.clone(),
            tags: self.tags.clone(),
            entity_type: self.entity_type.clone(),
            entity_id: self.entity_id.clone(),
        };

        TaskData {
            id: self.id.clone(),
            name: self.name().to_string(),
            priority: self.priority(),
            status: TaskStatus::Pending,
            created_at: self.created_at,
            payload: serde_json::to_value(payload).unwrap(),
        }
    }

    async fn execute(
        &self,
        db: Arc<DatabaseInstance>,
        client: reqwest::Client,
    ) -> Result<(), AppError> {
        info!(
            task = %self.name(),
            url = %self.url,
            "Fetching picture"
        );
        
        // Prepare filename and path
        let filename = Self::sanitize_filename(&self.get_filename());
        let file_path = self.storage_path.join(&filename);
        let file_path_str = file_path.to_string_lossy().to_string();
        
        // Create initial metadata
        let mut metadata = PictureMetadata::new(
            self.url.clone(),
            file_path_str.clone(),
            filename.clone(),
        );
        metadata.tags = self.tags.clone();
        metadata.entity_type = self.entity_type.clone();
        metadata.entity_id = self.entity_id.clone();
        metadata.status = PictureStatus::Downloading;
        
        // Check if picture already exists in database
        if let Some(existing) = database::get_picture_by_url(db.db(), &self.url).await? {
            if existing.is_completed() {
                info!(
                    task = %self.name(),
                    url = %self.url,
                    "Picture already downloaded, skipping"
                );
                return Ok(());
            }
            // Update download attempts
            metadata.download_attempts = existing.download_attempts + 1;
        }
        
        // Save initial metadata
        database::upsert_picture(db.db(), &metadata).await?;

        // Fetch the image
        let response = client
            .get(&self.url)
            .send()
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to fetch picture: {}", e);
                error!(task = %self.name(), error = %error_msg);
                AppError::Module(error_msg)
            })?;

        if !response.status().is_success() {
            let error_msg = format!("Failed to fetch picture: HTTP {}", response.status());
            metadata.status = PictureStatus::Failed { error: error_msg.clone() };
            database::upsert_picture(db.db(), &metadata).await?;
            return Err(AppError::Module(error_msg));
        }
        
        // Extract MIME type from response headers
        let mime_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string())
            .or_else(|| Self::detect_mime_type(&filename));

        // Get the image bytes
        let bytes = response
            .bytes()
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to read picture bytes: {}", e);
                error!(task = %self.name(), error = %error_msg);
                AppError::Module(error_msg)
            })?;

        debug!(
            task = %self.name(),
            size = bytes.len(),
            "Downloaded picture"
        );
        
        // Calculate content hash for deduplication
        let content_hash = Self::calculate_hash(&bytes);
        
        // Check if we already have this exact file
        if let Some(existing) = database::get_picture_by_hash(db.db(), &content_hash).await? {
            if existing.is_completed() && existing.url != self.url {
                info!(
                    task = %self.name(),
                    url = %self.url,
                    duplicate_of = %existing.url,
                    "Picture is a duplicate, updating metadata only"
                );
                metadata.file_path = existing.file_path;
                metadata.file_size = existing.file_size;
                metadata.width = existing.width;
                metadata.height = existing.height;
                metadata.content_hash = Some(content_hash);
                metadata.mime_type = mime_type;
                metadata.status = PictureStatus::Completed;
                database::upsert_picture(db.db(), &metadata).await?;
                return Ok(());
            }
        }

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| {
                    let error_msg = format!("Failed to create directory: {}", e);
                    error!(task = %self.name(), error = %error_msg);
                    AppError::Module(error_msg)
                })?;
        }

        // Write to file
        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to create file: {}", e);
                error!(task = %self.name(), error = %error_msg);
                AppError::Module(error_msg)
            })?;

        file.write_all(&bytes)
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to write file: {}", e);
                error!(task = %self.name(), error = %error_msg);
                AppError::Module(error_msg)
            })?;

        file.flush()
            .await
            .map_err(|e| {
                let error_msg = format!("Failed to flush file: {}", e);
                error!(task = %self.name(), error = %error_msg);
                AppError::Module(error_msg)
            })?;
        
        // Update metadata with file information
        metadata.file_size = Some(bytes.len() as u64);
        metadata.mime_type = mime_type;
        metadata.content_hash = Some(content_hash);
        metadata.status = PictureStatus::Completed;
        metadata.downloaded_at = Some(chrono::Utc::now());
        
        // Try to get image dimensions (optional, basic implementation)
        // For production, you might want to use an image processing library
        
        // Save final metadata
        database::upsert_picture(db.db(), &metadata).await?;

        info!(
            task = %self.name(),
            url = %self.url,
            path = ?file_path,
            size = bytes.len(),
            hash = %metadata.content_hash.as_ref().unwrap(),
            "Picture saved and tracked successfully"
        );

        Ok(())
    }
}