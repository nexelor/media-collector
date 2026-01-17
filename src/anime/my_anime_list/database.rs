use anyhow::Result;
use mongodb::{Database, IndexModel};
use mongodb::options::{FindOptions, IndexOptions, ReplaceOptions, UpdateOptions};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn};
use futures::stream::StreamExt;

use super::model::AnimeData;
use crate::global::error::DatabaseError;

/// Initialize MyAnimeList-specific collections and indexes
pub async fn initialize_collections(db: &Database) -> Result<(), DatabaseError> {
    info!("Initializing MyAnimeList database collections");

    // Main anime collection
    create_anime_indexes(db).await?;
    
    // Anime metadata/cache collection
    create_anime_cache_indexes(db).await?;
    
    // Search history collection
    create_search_history_indexes(db).await?;

    info!("MyAnimeList collections initialized");
    Ok(())
}

async fn create_anime_indexes(db: &Database) -> Result<(), DatabaseError> {
    let collection = db.collection::<AnimeData>("anime");
    
    // Unique index on anime ID
    let id_index = IndexModel::builder()
        .keys(doc! { "id": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build();
    
    // Text index on title for search
    let title_index = IndexModel::builder()
        .keys(doc! { "title": "text" })
        .build();
    
    // Index on score for sorting
    let score_index = IndexModel::builder()
        .keys(doc! { "score": -1 })
        .build();

    collection.create_indexes(vec![id_index, title_index, score_index]).await
        .map_err(|e| DatabaseError::Query(format!("Failed to create anime indexes: {}", e)))?;

    debug!("Created indexes for anime collection");
    Ok(())
}

async fn create_anime_cache_indexes(db: &Database) -> Result<(), DatabaseError> {
    let collection = db.collection::<mongodb::bson::Document>("anime_cache");
    
    // Compound index on anime_id and cache type
    let cache_index = IndexModel::builder()
        .keys(doc! { "anime_id": 1, "cache_type": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build();
    
    // TTL index to auto-expire cache after 24 hours
    let ttl_index = IndexModel::builder()
        .keys(doc! { "cached_at": 1 })
        .options(IndexOptions::builder()
            .expire_after(std::time::Duration::from_secs(86400))
            .build())
        .build();

    collection.create_indexes(vec![cache_index, ttl_index]).await
        .map_err(|e| DatabaseError::Query(format!("Failed to create cache indexes: {}", e)))?;

    debug!("Created indexes for anime_cache collection");
    Ok(())
}

async fn create_search_history_indexes(db: &Database) -> Result<(), DatabaseError> {
    let collection = db.collection::<mongodb::bson::Document>("search_history");
    
    // Index on search query
    let query_index = IndexModel::builder()
        .keys(doc! { "query": 1 })
        .build();
    
    // Index on timestamp for recent searches
    let timestamp_index = IndexModel::builder()
        .keys(doc! { "searched_at": -1 })
        .build();
    
    // TTL index to delete old searches after 30 days
    let ttl_index = IndexModel::builder()
        .keys(doc! { "searched_at": 1 })
        .options(IndexOptions::builder()
            .expire_after(std::time::Duration::from_secs(2592000)) // 30 days
            .build())
        .build();

    collection.create_indexes(vec![query_index, timestamp_index, ttl_index]).await
        .map_err(|e| DatabaseError::Query(format!("Failed to create search_history indexes: {}", e)))?;

    debug!("Created indexes for search_history collection");
    Ok(())
}

// ========================================================================
// Database Operations for AnimeData
// ========================================================================

/// Insert or update anime in database
pub async fn upsert_anime(db: &Database, data: &AnimeData) -> Result<(), DatabaseError> {
    let collection = db.collection::<AnimeData>("anime");
    let filter = doc! { "id": data.id };
    let options = ReplaceOptions::builder().upsert(true).build();

    collection.replace_one(filter, data)
        .with_options(options)
        .await
        .map_err(|e| DatabaseError::Query(format!("Failed to upsert anime: {}", e)))?;

    debug!(anime_id = data.id, title = %data.title, "Anime upserted");
    Ok(())
}

/// Insert anime (kept for compatibility)
pub async fn insert_anime(db: &Database, data: &AnimeData) -> Result<(), DatabaseError> {
    upsert_anime(db, data).await
}

/// Get anime by ID
pub async fn get_anime_by_id(db: &Database, anime_id: u32) -> Result<Option<AnimeData>, DatabaseError> {
    let collection = db.collection::<AnimeData>("anime");
    let filter = doc! { "id": anime_id };

    collection.find_one(filter).await
        .map_err(|e| DatabaseError::Query(format!("Failed to get anime: {}", e)))
}

/// Check if anime exists in database
pub async fn anime_exists(db: &Database, anime_id: u32) -> Result<bool, DatabaseError> {
    let collection = db.collection::<AnimeData>("anime");
    let filter = doc! { "id": anime_id };

    let count = collection.count_documents(filter).await
        .map_err(|e| DatabaseError::Query(format!("Failed to check existence: {}", e)))?;

    Ok(count > 0)
}

/// Search anime by title (text search)
pub async fn search_anime_by_title(
    db: &Database,
    query: &str,
    limit: i64,
) -> Result<Vec<AnimeData>, DatabaseError> {
    let collection = db.collection::<AnimeData>("anime");
    
    let filter = doc! {
        "$text": { "$search": query }
    };
    
    let options = FindOptions::builder()
        .limit(limit)
        .sort(doc! { "score": { "$meta": "textScore" } })
        .build();

    let mut cursor = collection.find(filter)
        .with_options(options)
        .await
        .map_err(|e| DatabaseError::Query(format!("Failed to search: {}", e)))?;

    let mut results = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(anime) => results.push(anime),
            Err(e) => warn!(error = %e, "Failed to deserialize anime"),
        }
    }

    // Record search history
    record_search_history(db, query).await?;

    Ok(results)
}

/// Get top rated anime
pub async fn get_top_rated_anime(db: &Database, limit: i64) -> Result<Vec<AnimeData>, DatabaseError> {
    let collection = db.collection::<AnimeData>("anime");
    
    let filter = doc! {
        "score": { "$ne": null }
    };
    
    let options = FindOptions::builder()
        .limit(limit)
        .sort(doc! { "score": -1 })
        .build();

    let mut cursor = collection.find(filter)
        .with_options(options)
        .await
        .map_err(|e| DatabaseError::Query(format!("Failed to get top rated: {}", e)))?;

    let mut results = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(anime) => results.push(anime),
            Err(e) => warn!(error = %e, "Failed to deserialize anime"),
        }
    }

    Ok(results)
}

/// Get anime count in database
pub async fn get_anime_count(db: &Database) -> Result<u64, DatabaseError> {
    let collection = db.collection::<AnimeData>("anime");
    
    collection.count_documents(doc! {}).await
        .map_err(|e| DatabaseError::Query(format!("Failed to count anime: {}", e)))
}

/// Delete anime by ID
pub async fn delete_anime(db: &Database, anime_id: u32) -> Result<bool, DatabaseError> {
    let collection = db.collection::<AnimeData>("anime");
    let filter = doc! { "id": anime_id };

    let result = collection.delete_one(filter).await
        .map_err(|e| DatabaseError::Query(format!("Failed to delete anime: {}", e)))?;

    Ok(result.deleted_count > 0)
}

/// Bulk insert anime
pub async fn bulk_insert_anime(db: &Database, anime_list: Vec<AnimeData>) -> Result<u64, DatabaseError> {
    if anime_list.is_empty() {
        return Ok(0);
    }

    let collection = db.collection::<AnimeData>("anime");
    
    let result = collection.insert_many(anime_list).await
        .map_err(|e| DatabaseError::Query(format!("Failed to bulk insert: {}", e)))?;

    Ok(result.inserted_ids.len() as u64)
}

// ========================================================================
// Cache Operations
// ========================================================================

#[derive(Debug, Serialize, Deserialize)]
struct AnimeCache {
    anime_id: u32,
    cache_type: String,
    data: mongodb::bson::Document,
    cached_at: chrono::DateTime<chrono::Utc>,
}

/// Cache additional anime data (e.g., full details, recommendations)
pub async fn cache_anime_data(
    db: &Database,
    anime_id: u32,
    cache_type: &str,
    data: mongodb::bson::Document,
) -> Result<(), DatabaseError> {
    let collection = db.collection::<AnimeCache>("anime_cache");
    
    let cache = AnimeCache {
        anime_id,
        cache_type: cache_type.to_string(),
        data,
        cached_at: chrono::Utc::now(),
    };

    let filter = doc! { "anime_id": anime_id, "cache_type": cache_type };
    let options = ReplaceOptions::builder().upsert(true).build();

    collection.replace_one(filter, cache)
        .with_options(options)
        .await
        .map_err(|e| DatabaseError::Query(format!("Failed to cache data: {}", e)))?;

    debug!(anime_id = anime_id, cache_type = cache_type, "Data cached");
    Ok(())
}

/// Get cached anime data
pub async fn get_cached_data(
    db: &Database,
    anime_id: u32,
    cache_type: &str,
) -> Result<Option<mongodb::bson::Document>, DatabaseError> {
    let collection = db.collection::<AnimeCache>("anime_cache");
    let filter = doc! { "anime_id": anime_id, "cache_type": cache_type };

    let result = collection.find_one(filter).await
        .map_err(|e| DatabaseError::Query(format!("Failed to get cache: {}", e)))?;

    Ok(result.map(|cache| cache.data))
}

// ========================================================================
// Search History Operations
// ========================================================================

#[derive(Debug, Serialize, Deserialize)]
struct SearchHistory {
    query: String,
    searched_at: chrono::DateTime<chrono::Utc>,
}

/// Record a search query
async fn record_search_history(db: &Database, query: &str) -> Result<(), DatabaseError> {
    let collection = db.collection::<SearchHistory>("search_history");
    
    let history = SearchHistory {
        query: query.to_string(),
        searched_at: chrono::Utc::now(),
    };

    collection.insert_one(history).await
        .map_err(|e| DatabaseError::Query(format!("Failed to record search: {}", e)))?;

    Ok(())
}

/// Get recent search queries
pub async fn get_recent_searches(db: &Database, limit: i64) -> Result<Vec<String>, DatabaseError> {
    let collection = db.collection::<SearchHistory>("search_history");
    
    let options = FindOptions::builder()
        .limit(limit)
        .sort(doc! { "searched_at": -1 })
        .build();

    let mut cursor = collection.find(doc! {})
        .with_options(options)
        .await
        .map_err(|e| DatabaseError::Query(format!("Failed to get searches: {}", e)))?;

    let mut results = Vec::new();
    while let Some(result) = cursor.next().await {
        match result {
            Ok(history) => results.push(history.query),
            Err(e) => warn!(error = %e, "Failed to deserialize search history"),
        }
    }

    Ok(results)
}