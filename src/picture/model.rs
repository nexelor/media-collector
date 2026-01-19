use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use mongodb::bson;

/// Status of a picture download
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PictureStatus {
    Pending,
    Downloading,
    Completed,
    Failed { error: String },
}

/// Metadata for a downloaded picture
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PictureMetadata {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    
    /// Original URL of the picture
    pub url: String,
    
    /// Local file path where the picture is stored
    pub file_path: String,
    
    /// Original filename from URL or custom filename
    pub filename: String,
    
    /// File size in bytes
    pub file_size: Option<u64>,
    
    /// MIME type (e.g., "image/jpeg", "image/png")
    pub mime_type: Option<String>,
    
    /// Width of the image in pixels
    pub width: Option<u32>,
    
    /// Height of the image in pixels
    pub height: Option<u32>,
    
    /// Current status of the download
    pub status: PictureStatus,
    
    /// Optional tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    
    /// Associated entity type (e.g., "anime", "manga", "character")
    pub entity_type: Option<String>,
    
    /// Associated entity ID
    pub entity_id: Option<String>,
    
    /// Number of download attempts
    pub download_attempts: u32,
    
    /// SHA-256 hash of the file content for deduplication
    pub content_hash: Option<String>,
    
    /// When the picture was first requested
    pub created_at: DateTime<Utc>,
    
    /// When the picture was last updated
    pub updated_at: DateTime<Utc>,
    
    /// When the picture was successfully downloaded
    pub downloaded_at: Option<DateTime<Utc>>,
}

impl PictureMetadata {
    pub fn new(url: String, file_path: String, filename: String) -> Self {
        Self {
            id: None,
            url,
            file_path,
            filename,
            file_size: None,
            mime_type: None,
            width: None,
            height: None,
            status: PictureStatus::Pending,
            tags: Vec::new(),
            entity_type: None,
            entity_id: None,
            download_attempts: 0,
            content_hash: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            downloaded_at: None,
        }
    }
    
    /// Check if the download was successful
    pub fn is_completed(&self) -> bool {
        matches!(self.status, PictureStatus::Completed)
    }
    
    /// Check if the download failed
    pub fn is_failed(&self) -> bool {
        matches!(self.status, PictureStatus::Failed { .. })
    }
}

/// Statistics about downloaded pictures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PictureStats {
    pub total_pictures: u64,
    pub completed: u64,
    pub pending: u64,
    pub failed: u64,
    pub total_size_bytes: u64,
    pub by_entity_type: Vec<EntityTypeStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTypeStats {
    pub entity_type: String,
    pub count: u64,
}