use std::sync::Arc;
use tracing::info;

use crate::global::config::AppConfig;
use crate::global::database::DatabaseInstance;
use crate::global::error::AppError;
use crate::global::http::ClientWithLimiter;
use crate::global::queue::TaskQueue;

use super::task::{FetchAnimeTask, SearchAnimeTask};

pub struct AniListModule {
    client: ClientWithLimiter,
    config: Arc<AppConfig>,
    queue: TaskQueue,
}

impl AniListModule {
    /// Create a new AniList module
    /// AniList doesn't require an API key, so we don't check for it
    pub fn new(
        client: ClientWithLimiter,
        config: Arc<AppConfig>,
        queue: TaskQueue,
    ) -> Option<Self> {
        // Check if module is enabled (no API key required for AniList)
        if !config.can_start_child_module("anilist", false) {
            return None;
        }

        Some(Self { 
            client,
            config, 
            queue 
        })
    }

    /// Check if this module is enabled and properly configured
    pub fn is_available(config: &AppConfig) -> bool {
        config.can_start_child_module("anilist", false)
    }

    /// Queue a task to fetch anime by MAL ID
    pub async fn queue_fetch_by_mal_id(&self, mal_id: u32) -> Result<(), AppError> {
        let task = FetchAnimeTask::by_mal_id(mal_id, self.client.clone());

        info!(
            module = "anilist",
            mal_id = mal_id,
            "Queueing fetch anime by MAL ID task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue a task to fetch anime by AniList ID
    pub async fn queue_fetch_by_anilist_id(&self, anilist_id: u32) -> Result<(), AppError> {
        let task = FetchAnimeTask::by_anilist_id(anilist_id, self.client.clone());

        info!(
            module = "anilist",
            anilist_id = anilist_id,
            "Queueing fetch anime by AniList ID task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue a task to search for anime
    pub async fn queue_search_anime(
        &self, 
        query: String, 
        page: Option<u32>,
        per_page: Option<u32>
    ) -> Result<(), AppError> {
        let task = SearchAnimeTask::new(
            query.clone(),
            page,
            per_page,
            self.client.clone(),
        );

        info!(
            module = "anilist",
            query = %query,
            "Queueing search anime task"
        );

        self.queue.enqueue(Box::new(task)).await
    }
}