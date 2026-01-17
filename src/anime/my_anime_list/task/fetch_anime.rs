use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn};

use crate::global::queue::{TaskData, TaskPriority, TaskStatus};
use crate::global::{
    database::DatabaseInstance,
    error::AppError,
    queue::Task,
    http::RequestConfig,
};
use crate::anime::my_anime_list::{
    model::{AnimeData, MalAnimeResponse, JikanAnimeResponse},
    database::upsert_anime,
    converter::{mal_to_anime_data, merge_jikan_data},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchAnimePayload {
    pub anime_id: u32,
    pub with_jikan: bool,
}

/// Task to fetch anime data from MyAnimeList API
pub struct FetchAnimeTask {
    id: String,
    anime_id: u32,
    api_key: String,
    mal_client: crate::global::http::ClientWithLimiter,
    jikan_client: crate::global::http::ClientWithLimiter,
    created_at: chrono::DateTime<chrono::Utc>,
    /// Whether to also fetch Jikan data for enrichment
    with_jikan: bool,
}

impl FetchAnimeTask {
    pub fn new(
        anime_id: u32,
        api_key: String,
        mal_client: crate::global::http::ClientWithLimiter,
        jikan_client: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("mal_search_{}", uuid::Uuid::new_v4());
        Self {
            id,
            anime_id,
            api_key,
            mal_client,
            jikan_client,
            created_at: chrono::Utc::now(),
            with_jikan: false,
        }
    }

    /// Create a fetch task that also fetches Jikan data for enrichment
    pub fn with_jikan(mut self) -> Self {
        self.with_jikan = true;
        self
    }
}

#[async_trait::async_trait]
impl Task for FetchAnimeTask {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> &str {
        "fetch_anime_mal"
    }

    fn priority(&self) -> TaskPriority {
        TaskPriority::Normal
    }

    fn to_data(&self) -> TaskData {
        let payload = FetchAnimePayload {
            anime_id:self.anime_id,
            with_jikan: self.with_jikan,
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
            anime_id = self.anime_id,
            fetch_jikan = self.with_jikan,
            "Fetching anime from MyAnimeList API"
        );

        // Step 1: Fetch from MyAnimeList API
        let mal_url = format!(
            "https://api.myanimelist.net/v2/anime/{}?fields=id,title,main_picture,alternative_titles,start_date,end_date,synopsis,mean,rank,popularity,num_list_users,num_scoring_users,nsfw,genres,created_at,updated_at,media_type,status,num_episodes,start_season,broadcast,source,average_episode_duration,rating,studios,pictures,background,related_anime,related_manga,statistics",
            self.anime_id
        );

        let config = RequestConfig::new().with_header("X-MAL-CLIENT-ID", &self.api_key);

        debug!(task = %self.name(), url = %mal_url, "Fetching from MAL API");
        let mal_response = self.mal_client
            .fetch_json::<MalAnimeResponse>(&mal_url, Some(config))
            .await?;

        info!(
            task = %self.name(),
            anime_id = mal_response.id,
            title = %mal_response.title,
            "Successfully fetched from MAL API"
        );

        // Convert MAL response to unified AnimeData
        let mut anime_data: AnimeData = mal_to_anime_data(
            mal_response, 
            Some(format!("https://myanimelist.net/anime/{}", self.anime_id))
        );

        // Step 2: Optionally fetch from Jikan API for enrichment
        if self.with_jikan {
            match self.fetch_jikan_data(self.anime_id).await {
                Ok(jikan_response) => {
                    info!(
                        task = %self.name(),
                        anime_id = self.anime_id,
                        "Successfully fetched Jikan data, merging..."
                    );
                    anime_data = merge_jikan_data(anime_data, jikan_response.data);
                }
                Err(e) => {
                    warn!(
                        task = %self.name(),
                        anime_id = self.anime_id,
                        error = %e,
                        "Failed to fetch Jikan data, continuing with MAL data only"
                    );
                }
            }
        }

        // Step 3: Store in database
        debug!(task = %self.name(), anime_id = anime_data.mal_id, "Storing anime in database");
        upsert_anime(db.db(), &anime_data).await?;
        
        info!(
            task = %self.name(),
            anime_id = anime_data.mal_id,
            title = %anime_data.titles.first().map(|t| t.title.as_str()).unwrap_or("Unknown"),
            "Anime stored successfully"
        );

        Ok(())
    }
}

impl FetchAnimeTask {
    /// Fetch anime data from Jikan API (no authentication required)
    async fn fetch_jikan_data(&self, mal_id: u32) -> Result<JikanAnimeResponse, AppError> {
        let jikan_url = format!("https://api.jikan.moe/v4/anime/{}/full", mal_id);
        
        debug!(
            task = %self.name(),
            url = %jikan_url,
            "Fetching from Jikan API"
        );

        // Use Jikan client with its own rate limiter (no manual delay needed)
        let response = self.jikan_client  // CHANGED: use jikan_client
            .fetch_json::<JikanAnimeResponse>(&jikan_url, None)
            .await?;

        Ok(response)
    }
}