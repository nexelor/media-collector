use anyhow::Result;
use mongodb::{Database, IndexModel};
use mongodb::options::{FindOptions, IndexOptions, ReplaceOptions};
use mongodb::bson::doc;
use tracing::{info, debug, warn};
use futures::stream::StreamExt;

use crate::anime::anilist::model::AniListAnimeData;
use crate::global::error::DatabaseError;

// Collection name for AniList anime
const COLLECTION_NAME: &str = "anime_anilist";

/// Initialize AniList-specific collections and indexes
pub async fn initialize_collections(db: &Database) -> Result<(), DatabaseError> {
    info!("Initializing AniList database collections");

    // Main anime collection
    create_anime_indexes(db).await?;

    info!("AniList collections initialized");
    Ok(())
}

async fn create_anime_indexes(db: &Database) -> Result<(), DatabaseError> {
    let collection = db.collection::<AniListAnimeData>(COLLECTION_NAME);
    
    // Unique index on AniList ID
    let anilist_id_index = IndexModel::builder()
        .keys(doc! { "anilist_id": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build();
    
    // Index on MAL ID for cross-referencing
    let mal_id_index = IndexModel::builder()
        .keys(doc! { "mal_id": 1 })
        .build();
    
    // Text index on titles for search
    let title_index = IndexModel::builder()
        .keys(doc! { 
            "titles.title": "text",
            "synopsis": "text" 
        })
        .build();
    
    // Index on score for sorting
    let score_index = IndexModel::builder()
        .keys(doc! { "score": -1 })
        .build();
    
    // Index on popularity for sorting
    let popularity_index = IndexModel::builder()
        .keys(doc! { "popularity": -1 })
        .build();
    
    // Index on media type for filtering
    let media_type_index = IndexModel::builder()
        .keys(doc! { "media_type": 1 })
        .build();
    
    // Index on status for filtering
    let status_index = IndexModel::builder()
        .keys(doc! { "status": 1 })
        .build();
    
    // Compound index for season queries
    let season_index = IndexModel::builder()
        .keys(doc! { "year": -1, "season": 1 })
        .build();

    // Index on updated_at for sync/update queries
    let updated_index = IndexModel::builder()
        .keys(doc! { "updated_at": -1 })
        .build();

    collection.create_indexes(vec![
        anilist_id_index,
        mal_id_index,
        title_index,
        score_index,
        popularity_index,
        media_type_index,
        status_index,
        season_index,
        updated_index,
    ]).await
        .map_err(|e| DatabaseError::Query(format!("Failed to create anime indexes: {}", e)))?;

    debug!("Created indexes for anime_anilist collection");
    Ok(())
}

// ========================================================================
// Database Operations for AniListAnimeData
// ========================================================================

/// Insert or update anime in database
pub async fn upsert_anime(db: &Database, data: &AniListAnimeData) -> Result<(), DatabaseError> {
    let collection = db.collection::<AniListAnimeData>(COLLECTION_NAME);
    let filter = doc! { "anilist_id": data.anilist_id };
    let options = ReplaceOptions::builder().upsert(true).build();

    collection.replace_one(filter, data)
        .with_options(options)
        .await
        .map_err(|e| DatabaseError::Query(format!("Failed to upsert anime: {}", e)))?;

    debug!(
        anilist_id = data.anilist_id,
        title = %data.titles.first().map(|t| t.title.as_str()).unwrap_or("Unknown"),
        "Anime upserted to anime_anilist collection"
    );
    Ok(())
}

/// Get anime by AniList ID
pub async fn get_anime_by_id(db: &Database, anilist_id: i32) -> Result<Option<AniListAnimeData>, DatabaseError> {
    let collection = db.collection::<AniListAnimeData>(COLLECTION_NAME);
    let filter = doc! { "anilist_id": anilist_id };

    collection.find_one(filter).await
        .map_err(|e| DatabaseError::Query(format!("Failed to get anime: {}", e)))
}

/// Get anime by MAL ID
pub async fn get_anime_by_mal_id(db: &Database, mal_id: i32) -> Result<Option<AniListAnimeData>, DatabaseError> {
    let collection = db.collection::<AniListAnimeData>(COLLECTION_NAME);
    let filter = doc! { "mal_id": mal_id };

    collection.find_one(filter).await
        .map_err(|e| DatabaseError::Query(format!("Failed to get anime by MAL ID: {}", e)))
}

/// Check if anime exists in database
pub async fn anime_exists(db: &Database, anilist_id: i32) -> Result<bool, DatabaseError> {
    let collection = db.collection::<AniListAnimeData>(COLLECTION_NAME);
    let filter = doc! { "anilist_id": anilist_id };

    let count = collection.count_documents(filter).await
        .map_err(|e| DatabaseError::Query(format!("Failed to check existence: {}", e)))?;

    Ok(count > 0)
}

/// Search anime by title (text search)
pub async fn search_anime_by_title(
    db: &Database,
    query: &str,
    limit: i64,
) -> Result<Vec<AniListAnimeData>, DatabaseError> {
    let collection = db.collection::<AniListAnimeData>(COLLECTION_NAME);
    
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

    Ok(results)
}

/// Get anime count in database
pub async fn get_anime_count(db: &Database) -> Result<u64, DatabaseError> {
    let collection = db.collection::<AniListAnimeData>(COLLECTION_NAME);
    
    collection.count_documents(doc! {}).await
        .map_err(|e| DatabaseError::Query(format!("Failed to count anime: {}", e)))
}

/// Delete anime by AniList ID
pub async fn delete_anime(db: &Database, anilist_id: i32) -> Result<bool, DatabaseError> {
    let collection = db.collection::<AniListAnimeData>(COLLECTION_NAME);
    let filter = doc! { "anilist_id": anilist_id };

    let result = collection.delete_one(filter).await
        .map_err(|e| DatabaseError::Query(format!("Failed to delete anime: {}", e)))?;

    Ok(result.deleted_count > 0)
}