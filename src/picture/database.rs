// src/picture/database.rs
use anyhow::Result;
use mongodb::{Database, IndexModel};
use mongodb::options::{FindOptions, IndexOptions, ReplaceOptions, UpdateOptions};
use mongodb::bson::{doc, Document};
use tracing::{info, debug, warn};
use futures::stream::StreamExt;

use super::model::{PictureMetadata, PictureStatus, PictureStats, EntityTypeStats};
use crate::global::error::DatabaseError;

const COLLECTION_NAME: &str = "pictures";

/// Initialize picture tracking collection and indexes
pub async fn initialize_collections(db: &Database) -> Result<(), DatabaseError> {
    info!("Initializing picture tracking collections");
    
    let collection = db.collection::<PictureMetadata>(COLLECTION_NAME);
    
    // Unique index on URL
    let url_index = IndexModel::builder()
        .keys(doc! { "url": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build();
    
    // Index on file_path
    let path_index = IndexModel::builder()
        .keys(doc! { "file_path": 1 })
        .build();
    
    // Index on status for querying by download status
    let status_index = IndexModel::builder()
        .keys(doc! { "status": 1 })
        .build();
    
    // Index on content_hash for deduplication
    let hash_index = IndexModel::builder()
        .keys(doc! { "content_hash": 1 })
        .build();
    
    // Compound index on entity_type and entity_id
    let entity_index = IndexModel::builder()
        .keys(doc! { "entity_type": 1, "entity_id": 1 })
        .build();
    
    // Index on tags for filtering
    let tags_index = IndexModel::builder()
        .keys(doc! { "tags": 1 })
        .build();
    
    // Index on created_at for time-based queries
    let created_index = IndexModel::builder()
        .keys(doc! { "created_at": -1 })
        .build();
    
    collection.create_indexes(vec![
        url_index,
        path_index,
        status_index,
        hash_index,
        entity_index,
        tags_index,
        created_index,
    ]).await
        .map_err(|e| DatabaseError::Query(format!("Failed to create picture indexes: {}", e)))?;
    
    debug!("Created indexes for pictures collection");
    Ok(())
}

/// Insert or update picture metadata
pub async fn upsert_picture(db: &Database, picture: &PictureMetadata) -> Result<(), DatabaseError> {
    let collection = db.collection::<PictureMetadata>(COLLECTION_NAME);
    let filter = doc! { "url": &picture.url };
    let options = ReplaceOptions::builder().upsert(true).build();
    
    collection.replace_one(filter, picture)
        .with_options(options)
        .await
        .map_err(|e| DatabaseError::Query(format!("Failed to upsert picture: {}", e)))?;
    
    debug!(
        url = %picture.url,
        filename = %picture.filename,
        status = ?picture.status,
        "Picture metadata upserted"
    );
    Ok(())
}

/// Get picture metadata by URL
pub async fn get_picture_by_url(db: &Database, url: &str) -> Result<Option<PictureMetadata>, DatabaseError> {
    let collection = db.collection::<PictureMetadata>(COLLECTION_NAME);
    let filter = doc! { "url": url };
    
    collection.find_one(filter).await
        .map_err(|e| DatabaseError::Query(format!("Failed to get picture: {}", e)))
}

/// Get picture metadata by file path
pub async fn get_picture_by_path(db: &Database, path: &str) -> Result<Option<PictureMetadata>, DatabaseError> {
    let collection = db.collection::<PictureMetadata>(COLLECTION_NAME);
    let filter = doc! { "file_path": path };
    
    collection.find_one(filter).await
        .map_err(|e| DatabaseError::Query(format!("Failed to get picture: {}", e)))
}

/// Get picture metadata by content hash
pub async fn get_picture_by_hash(db: &Database, hash: &str) -> Result<Option<PictureMetadata>, DatabaseError> {
    let collection = db.collection::<PictureMetadata>(COLLECTION_NAME);
    let filter = doc! { "content_hash": hash };
    
    collection.find_one(filter).await
        .map_err(|e| DatabaseError::Query(format!("Failed to get picture by hash: {}", e)))
}

/// Check if a picture URL already exists
pub async fn picture_exists(db: &Database, url: &str) -> Result<bool, DatabaseError> {
    let collection = db.collection::<PictureMetadata>(COLLECTION_NAME);
    let filter = doc! { "url": url };
    
    let count = collection.count_documents(filter).await
        .map_err(|e| DatabaseError::Query(format!("Failed to check existence: {}", e)))?;
    
    Ok(count > 0)
}

/// Update picture status
pub async fn update_picture_status(
    db: &Database,
    url: &str,
    status: PictureStatus
) -> Result<(), DatabaseError> {
    let collection = db.collection::<PictureMetadata>(COLLECTION_NAME);
    let filter = doc! { "url": url };
    
    let mut update_doc = doc! {
        "$set": {
            "status": mongodb::bson::to_bson(&status)
                .map_err(|e| DatabaseError::Query(format!("Failed to serialize status: {}", e)))?,
            "updated_at": mongodb::bson::DateTime::now(),
        }
    };
    
    // If completed, set downloaded_at
    if matches!(status, PictureStatus::Completed) {
        update_doc.get_document_mut("$set").unwrap().insert(
            "downloaded_at",
            mongodb::bson::DateTime::now()
        );
    }
    
    collection.update_one(filter, update_doc).await
        .map_err(|e| DatabaseError::Query(format!("Failed to update status: {}", e)))?;
    
    Ok(())
}

/// Get all pictures for an entity
pub async fn get_pictures_by_entity(
    db: &Database,
    entity_type: &str,
    entity_id: &str
) -> Result<Vec<PictureMetadata>, DatabaseError> {
    let collection = db.collection::<PictureMetadata>(COLLECTION_NAME);
    let filter = doc! {
        "entity_type": entity_type,
        "entity_id": entity_id
    };
    
    let mut cursor = collection.find(filter).await
        .map_err(|e| DatabaseError::Query(format!("Failed to get pictures: {}", e)))?;
    
    let mut results = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(picture) => results.push(picture),
            Err(e) => warn!(error = %e, "Failed to deserialize picture"),
        }
    }
    
    Ok(results)
}

/// Get pictures by status
pub async fn get_pictures_by_status(
    db: &Database,
    status: &str,
    limit: i64
) -> Result<Vec<PictureMetadata>, DatabaseError> {
    let collection = db.collection::<PictureMetadata>(COLLECTION_NAME);
    let filter = doc! { "status": status };
    
    let options = FindOptions::builder()
        .limit(limit)
        .sort(doc! { "created_at": 1 })
        .build();
    
    let mut cursor = collection.find(filter)
        .with_options(options)
        .await
        .map_err(|e| DatabaseError::Query(format!("Failed to get pictures: {}", e)))?;
    
    let mut results = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(picture) => results.push(picture),
            Err(e) => warn!(error = %e, "Failed to deserialize picture"),
        }
    }
    
    Ok(results)
}

/// Get pictures by tags
pub async fn get_pictures_by_tag(
    db: &Database,
    tag: &str,
    limit: i64
) -> Result<Vec<PictureMetadata>, DatabaseError> {
    let collection = db.collection::<PictureMetadata>(COLLECTION_NAME);
    let filter = doc! { "tags": tag };
    
    let options = FindOptions::builder()
        .limit(limit)
        .sort(doc! { "downloaded_at": -1 })
        .build();
    
    let mut cursor = collection.find(filter)
        .with_options(options)
        .await
        .map_err(|e| DatabaseError::Query(format!("Failed to get pictures: {}", e)))?;
    
    let mut results = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(picture) => results.push(picture),
            Err(e) => warn!(error = %e, "Failed to deserialize picture"),
        }
    }
    
    Ok(results)
}

/// Get picture statistics
pub async fn get_picture_stats(db: &Database) -> Result<PictureStats, DatabaseError> {
    let collection = db.collection::<Document>(COLLECTION_NAME);
    
    let total = collection.count_documents(doc! {}).await
        .map_err(|e| DatabaseError::Query(format!("Failed to count total: {}", e)))?;
    
    let completed = collection.count_documents(doc! { "status": "Completed" }).await
        .map_err(|e| DatabaseError::Query(format!("Failed to count completed: {}", e)))?;
    
    let pending = collection.count_documents(doc! { "status": "Pending" }).await
        .map_err(|e| DatabaseError::Query(format!("Failed to count pending: {}", e)))?;
    
    let failed = collection.count_documents(doc! { 
        "status": { "$regex": "^Failed" } 
    }).await
        .map_err(|e| DatabaseError::Query(format!("Failed to count failed: {}", e)))?;
    
    // Calculate total size
    let pipeline = vec![
        doc! {
            "$match": {
                "status": "Completed",
                "file_size": { "$exists": true }
            }
        },
        doc! {
            "$group": {
                "_id": null,
                "total_size": { "$sum": "$file_size" }
            }
        }
    ];
    
    let mut cursor = collection.aggregate(pipeline).await
        .map_err(|e| DatabaseError::Query(format!("Failed to aggregate size: {}", e)))?;
    
    let total_size = if let Some(result) = cursor.next().await {
        let doc = result.map_err(|e| DatabaseError::Query(format!("Failed to read aggregate: {}", e)))?;
        doc.get_i64("total_size").unwrap_or(0) as u64
    } else {
        0
    };
    
    // Get entity type stats
    let entity_pipeline = vec![
        doc! {
            "$match": {
                "entity_type": { "$exists": true, "$ne": null }
            }
        },
        doc! {
            "$group": {
                "_id": "$entity_type",
                "count": { "$sum": 1 }
            }
        },
        doc! {
            "$sort": { "count": -1 }
        }
    ];
    
    let mut entity_cursor = collection.aggregate(entity_pipeline).await
        .map_err(|e| DatabaseError::Query(format!("Failed to aggregate entities: {}", e)))?;
    
    let mut by_entity_type = Vec::new();
    while let Some(result) = entity_cursor.next().await {
        match result {
            Ok(doc) => {
                if let (Some(entity_type), Some(count)) = (
                    doc.get_str("_id").ok(),
                    doc.get_i64("count").ok()
                ) {
                    by_entity_type.push(EntityTypeStats {
                        entity_type: entity_type.to_string(),
                        count: count as u64,
                    });
                }
            }
            Err(e) => warn!(error = %e, "Failed to read entity stats"),
        }
    }
    
    Ok(PictureStats {
        total_pictures: total,
        completed,
        pending,
        failed,
        total_size_bytes: total_size,
        by_entity_type,
    })
}

/// Delete picture metadata
pub async fn delete_picture(db: &Database, url: &str) -> Result<bool, DatabaseError> {
    let collection = db.collection::<PictureMetadata>(COLLECTION_NAME);
    let filter = doc! { "url": url };
    
    let result = collection.delete_one(filter).await
        .map_err(|e| DatabaseError::Query(format!("Failed to delete picture: {}", e)))?;
    
    Ok(result.deleted_count > 0)
}

/// Clean up failed downloads older than specified days
pub async fn cleanup_failed_pictures(db: &Database, days: i64) -> Result<u64, DatabaseError> {
    let collection = db.collection::<Document>(COLLECTION_NAME);
    let threshold = mongodb::bson::DateTime::now().timestamp_millis() - (days * 24 * 60 * 60 * 1000);
    
    let filter = doc! {
        "status": { "$regex": "^Failed" },
        "updated_at": { "$lt": threshold }
    };
    
    let result = collection.delete_many(filter).await
        .map_err(|e| DatabaseError::Query(format!("Failed to cleanup: {}", e)))?;
    
    info!(deleted = result.deleted_count, "Cleaned up failed picture records");
    Ok(result.deleted_count)
}