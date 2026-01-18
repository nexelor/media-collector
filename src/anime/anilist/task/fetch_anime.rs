use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{info, debug};

use crate::anime::anilist::database::upsert_anime;
use crate::global::queue::{TaskData, TaskPriority, TaskStatus};
use crate::global::{
    database::DatabaseInstance,
    error::AppError,
    queue::Task,
    http::RequestConfig,
};
use crate::anime::anilist::{
    model::{GraphQLRequest, GraphQLResponse, MediaData},
    converter::anilist_to_anime_data,
    queries,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchAnimePayload {
    pub mal_id: Option<u32>,
    pub anilist_id: Option<u32>,
}

/// Task to fetch anime data from AniList API
pub struct FetchAnimeTask {
    id: String,
    mal_id: Option<u32>,
    anilist_id: Option<u32>,
    client: crate::global::http::ClientWithLimiter,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl FetchAnimeTask {
    /// Create a task to fetch by MAL ID
    pub fn by_mal_id(
        mal_id: u32,
        client: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("anilist_fetch_mal_{}", mal_id);
        Self {
            id,
            mal_id: Some(mal_id),
            anilist_id: None,
            client,
            created_at: chrono::Utc::now(),
        }
    }

    /// Create a task to fetch by AniList ID
    pub fn by_anilist_id(
        anilist_id: u32,
        client: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("anilist_fetch_{}", anilist_id);
        Self {
            id,
            mal_id: None,
            anilist_id: Some(anilist_id),
            client,
            created_at: chrono::Utc::now(),
        }
    }
}

#[async_trait::async_trait]
impl Task for FetchAnimeTask {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> &str {
        "fetch_anime_anilist"
    }

    fn priority(&self) -> TaskPriority {
        TaskPriority::Normal
    }

    fn to_data(&self) -> TaskData {
        let payload = FetchAnimePayload {
            mal_id: self.mal_id,
            anilist_id: self.anilist_id,
        };

        TaskData {
            id: self.id(),
            name: self.name().to_string(),
            priority: self.priority(),
            status: TaskStatus::Pending,
            created_at: self.created_at,
            payload: serde_json::json!(payload),
        }
    }

    async fn execute(&self, db: Arc<DatabaseInstance>, _client: reqwest::Client) -> Result<(), AppError> {
        info!(
            task = %self.name(),
            mal_id = ?self.mal_id,
            anilist_id = ?self.anilist_id,
            "Fetching anime from AniList API"
        );

        // Build GraphQL request
        let (query, variables) = if let Some(mal_id) = self.mal_id {
            (
                queries::ANIME_BY_MAL_ID_QUERY,
                serde_json::json!({ "malId": mal_id })
            )
        } else if let Some(anilist_id) = self.anilist_id {
            (
                queries::ANIME_BY_ID_QUERY,
                serde_json::json!({ "id": anilist_id })
            )
        } else {
            return Err(AppError::Module("No ID provided for AniList fetch".to_string()));
        };

        let graphql_request = GraphQLRequest {
            query: query.to_string(),
            variables: Some(variables),
        };

        debug!(task = %self.name(), "Sending GraphQL request to AniList");
        
        // AniList GraphQL endpoint
        let url = "https://graphql.anilist.co";
        
        let config = RequestConfig::new()
            .with_header("Content-Type", "application/json")
            .with_header("Accept", "application/json");

        // Make the request using POST with JSON body
        let response = self.client.client
            .post(url)
            .json(&graphql_request)
            .send()
            .await
            .map_err(|e| AppError::Module(format!("AniList API request failed: {}", e)))?;

        let graphql_response = response
            .json::<GraphQLResponse<MediaData>>()
            .await
            .map_err(|e| AppError::Module(format!("Failed to parse AniList response: {}", e)))?;

        // Check for GraphQL errors
        if !graphql_response.errors.is_empty() {
            let error_messages: Vec<String> = graphql_response.errors
                .iter()
                .map(|e| e.message.clone())
                .collect();
            return Err(AppError::Module(format!(
                "AniList GraphQL errors: {}",
                error_messages.join(", ")
            )));
        }

        let media_data = graphql_response.data
            .ok_or_else(|| AppError::Module("No data returned from AniList".to_string()))?;

        let anilist_media = media_data.media;

        info!(
            task = %self.name(),
            anilist_id = anilist_media.id,
            mal_id = ?anilist_media.id_mal,
            title = ?anilist_media.title.romaji,
            "Successfully fetched from AniList API"
        );

        // Convert to unified AnimeData
        let anime_data = anilist_to_anime_data(anilist_media);

        // Store in database
        debug!(task = %self.name(), anime_id = anime_data.anilist_id, "Storing anime in database");
        upsert_anime(db.db(), &anime_data).await?;
        
        info!(
            task = %self.name(),
            anilist_id = anime_data.anilist_id,
            mal_id = ?anime_data.mal_id,
            title = %anime_data.titles.first().map(|t| t.title.as_str()).unwrap_or("Unknown"),
            "Anime stored successfully in anime_anilist collection"
        );

        Ok(())
    }
}