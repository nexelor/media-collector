use std::sync::Arc;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{info, debug, warn, error};

use crate::global::{
    database::DatabaseInstance,
    error::AppError,
    queue::{Task, TaskPriority, TaskData, TaskStatus},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchPicturePayload {
    pub url: String,
    pub storage_path: String,
    pub filename: Option<String>,
}

pub struct FetchPictureTask {
    id: String,
    url: String,
    storage_path: PathBuf,
    filename: Option<String>,
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
            created_at: chrono::Utc::now(),
        }
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
        TaskPriority::Low  // Pictures are lower priority
    }

    fn to_data(&self) -> TaskData {
        let payload = FetchPicturePayload {
            url: self.url.clone(),
            storage_path: self.storage_path.to_string_lossy().to_string(),
            filename: self.filename.clone(),
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
        _db: Arc<DatabaseInstance>,
        client: reqwest::Client,
    ) -> Result<(), AppError> {
        info!(
            task = %self.name(),
            url = %self.url,
            "Fetching picture"
        );

        // Fetch the image
        let response = client
            .get(&self.url)
            .send()
            .await
            .map_err(|e| AppError::Module(format!("Failed to fetch picture: {}", e)))?;

        if !response.status().is_success() {
            return Err(AppError::Module(format!(
                "Failed to fetch picture: HTTP {}",
                response.status()
            )));
        }

        // Get the image bytes
        let bytes = response
            .bytes()
            .await
            .map_err(|e| AppError::Module(format!("Failed to read picture bytes: {}", e)))?;

        debug!(
            task = %self.name(),
            size = bytes.len(),
            "Downloaded picture"
        );

        // Prepare file path
        let filename = Self::sanitize_filename(&self.get_filename());
        let file_path = self.storage_path.join(&filename);

        // Ensure parent directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)
                .await
                .map_err(|e| AppError::Module(format!("Failed to create directory: {}", e)))?;
        }

        // Write to file
        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|e| AppError::Module(format!("Failed to create file: {}", e)))?;

        file.write_all(&bytes)
            .await
            .map_err(|e| AppError::Module(format!("Failed to write file: {}", e)))?;

        file.flush()
            .await
            .map_err(|e| AppError::Module(format!("Failed to flush file: {}", e)))?;

        info!(
            task = %self.name(),
            url = %self.url,
            path = ?file_path,
            size = bytes.len(),
            "Picture saved successfully"
        );

        Ok(())
    }
}