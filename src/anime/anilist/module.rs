use std::sync::Arc;
use tracing::info;

use crate::{global::config::AppConfig, picture::PictureFetcherModule};
use crate::global::database::DatabaseInstance;
use crate::global::error::AppError;
use crate::global::http::ClientWithLimiter;
use crate::global::queue::TaskQueue;

use super::task::{FetchAnimeTask, SearchAnimeTask};

pub struct AniListModule {
    client: ClientWithLimiter,
    config: Arc<AppConfig>,
    queue: TaskQueue,
    picture_module: Option<Arc<PictureFetcherModule>>,
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
            queue,
            picture_module: None,
        })
    }

    /// Set the picture module reference
    pub fn with_picture_module(mut self, picture_module: Arc<PictureFetcherModule>) -> Self {
        self.picture_module = Some(picture_module);
        self
    }

    /// Check if this module is enabled and properly configured
    pub fn is_available(config: &AppConfig) -> bool {
        config.can_start_child_module("anilist", false)
    }

    /// Queue a task to fetch anime by MAL ID
    pub async fn queue_fetch_by_mal_id(&self, mal_id: u32) -> Result<(), AppError> {
        self.queue_fetch_by_mal_id_with_options(mal_id, false).await
    }
    
    /// Queue a task to fetch anime by MAL ID with pictures
    pub async fn queue_fetch_by_mal_id_with_pictures(&self, mal_id: u32) -> Result<(), AppError> {
        self.queue_fetch_by_mal_id_with_options(mal_id, true).await
    }
    
    async fn queue_fetch_by_mal_id_with_options(
        &self,
        mal_id: u32,
        with_pictures: bool,
    ) -> Result<(), AppError> {
        let mut task = FetchAnimeTask::by_mal_id(mal_id, self.client.clone());
        
        if with_pictures {
            if let Some(picture_module) = &self.picture_module {
                task = task.with_pictures(picture_module.clone());
            } else {
                info!(
                    module = "anilist",
                    mal_id = mal_id,
                    "Picture module not available, fetching without pictures"
                );
            }
        }

        info!(
            module = "anilist",
            mal_id = mal_id,
            with_pictures = with_pictures,
            "Queueing fetch anime by MAL ID task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue a task to fetch anime by AniList ID
    pub async fn queue_fetch_by_anilist_id(&self, anilist_id: u32) -> Result<(), AppError> {
        self.queue_fetch_by_anilist_id_with_options(anilist_id, false).await
    }
    
    /// Queue a task to fetch anime by AniList ID with pictures
    pub async fn queue_fetch_by_anilist_id_with_pictures(&self, anilist_id: u32) -> Result<(), AppError> {
        self.queue_fetch_by_anilist_id_with_options(anilist_id, true).await
    }
    
    async fn queue_fetch_by_anilist_id_with_options(
        &self,
        anilist_id: u32,
        with_pictures: bool,
    ) -> Result<(), AppError> {
        let mut task = FetchAnimeTask::by_anilist_id(anilist_id, self.client.clone());
        
        if with_pictures {
            if let Some(picture_module) = &self.picture_module {
                task = task.with_pictures(picture_module.clone());
            } else {
                info!(
                    module = "anilist",
                    anilist_id = anilist_id,
                    "Picture module not available, fetching without pictures"
                );
            }
        }

        info!(
            module = "anilist",
            anilist_id = anilist_id,
            with_pictures = with_pictures,
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