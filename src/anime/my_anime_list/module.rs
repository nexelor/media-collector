use std::sync::Arc;
use tracing::{info, warn};

use crate::global::config::AppConfig;
use crate::global::error::AppError;
use crate::global::http::ClientWithLimiter;
use crate::global::queue::TaskQueue;
use crate::picture::PictureFetcherModule;

use super::task::{
    FetchAnimeTask, SearchAnimeTask, UpdateAnimeTask, BatchFetchTask,
    FetchCharactersTask, FetchEpisodesTask, FetchStaffTask,
    FetchVideosTask, FetchStatisticsTask, FetchMoreInfoTask,
    FetchRecommendationsTask, FetchPicturesTask,
};

pub struct MyAnimeListModule {
    mal_client: ClientWithLimiter,
    jikan_client: ClientWithLimiter,
    config: Arc<AppConfig>,
    queue: TaskQueue,
    picture_module: Option<Arc<PictureFetcherModule>>,
}

impl MyAnimeListModule {
    pub fn new(
        mal_client: ClientWithLimiter,
        jikan_client: ClientWithLimiter,
        config: Arc<AppConfig>,
        queue: TaskQueue,
    ) -> Option<Self> {
        if !config.can_start_child_module("my_anime_list", true) {
            return None;
        }

        Some(Self { 
            mal_client,
            jikan_client,
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

    pub fn is_available(config: &AppConfig) -> bool {
        config.can_start_child_module("my_anime_list", true)
    }

    /// Queue a task to fetch anime by ID
    pub async fn queue_fetch_anime(&self, anime_id: u32, with_jikan: bool) -> Result<(), AppError> {
        self.queue_fetch_anime_with_options(anime_id, with_jikan, false, false).await
    }
    
    /// Queue a task to fetch anime with picture downloads
    pub async fn queue_fetch_anime_with_pictures(
        &self,
        anime_id: u32,
        with_jikan: bool,
    ) -> Result<(), AppError> {
        self.queue_fetch_anime_with_options(anime_id, with_jikan, true, false).await
    }
    
    /// Queue a full fetch task (base + extended + pictures)
    pub async fn queue_fetch_anime_full(
        &self,
        anime_id: u32,
    ) -> Result<(), AppError> {
        self.queue_fetch_anime_with_options(anime_id, true, true, true).await
    }
    
    /// Internal method to queue fetch with all options
    async fn queue_fetch_anime_with_options(
        &self,
        anime_id: u32,
        with_jikan: bool,
        with_pictures: bool,
        full_fetch: bool,
    ) -> Result<(), AppError> {
        let api_key = self.config.get_api_key("my_anime_list")
            .expect("API key should be validated during module creation");

        let mut task = FetchAnimeTask::new(
            anime_id,
            api_key,
            self.mal_client.clone(),
            self.jikan_client.clone(),
        );

        if with_jikan {
            task = task.with_jikan();
        }
        
        if full_fetch {
            // Full fetch automatically enables jikan and pictures
            if let Some(picture_module) = &self.picture_module {
                task = task.full_fetch(picture_module.clone());
            } else {
                warn!(
                    module = "my_anime_list",
                    anime_id = anime_id,
                    "Picture module not available for full fetch, using basic fetch"
                );
            }
        } else if with_pictures {
            if let Some(picture_module) = &self.picture_module {
                task = task.with_pictures(picture_module.clone());
            } else {
                info!(
                    module = "my_anime_list",
                    anime_id = anime_id,
                    "Picture module not available, fetching without pictures"
                );
            }
        }

        info!(
            module = "my_anime_list",
            anime_id = anime_id,
            with_jikan = with_jikan,
            with_pictures = with_pictures,
            full_fetch = full_fetch,
            "Queueing fetch anime task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue a task to search for anime
    pub async fn queue_search_anime(&self, query: String, limit: Option<u32>) -> Result<(), AppError> {
        let api_key = self.config.get_api_key("my_anime_list")
            .expect("API key should be validated during module creation");

        let task = SearchAnimeTask::new(
            query.clone(),
            limit,
            api_key,
            self.mal_client.clone(),
        );

        info!(
            module = "my_anime_list",
            query = %query,
            "Queueing search anime task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue a task to update an existing anime
    pub async fn queue_update_anime(&self, anime_id: u32, with_jikan: bool) -> Result<(), AppError> {
        let api_key = self.config.get_api_key("my_anime_list")
            .expect("API key should be validated during module creation");

        let mut task = UpdateAnimeTask::new(
            anime_id,
            api_key,
            self.mal_client.clone(),
            self.jikan_client.clone(),
        );

        if with_jikan {
            task = task.with_jikan();
        }

        info!(
            module = "my_anime_list",
            anime_id = anime_id,
            "Queueing update anime task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue a batch fetch task
    pub async fn queue_batch_fetch(
        &self, 
        anime_ids: Vec<u32>, 
        with_jikan: bool,
        with_pictures: bool,
        full_fetch: bool,
    ) -> Result<(), AppError> {
        info!(
            module = "my_anime_list",
            count = anime_ids.len(),
            with_jikan = with_jikan,
            with_pictures = with_pictures,
            full_fetch = full_fetch,
            "Queueing batch fetch"
        );

        // Queue individual fetch tasks for each anime
        for anime_id in anime_ids {
            self.queue_fetch_anime_with_options(
                anime_id,
                with_jikan,
                with_pictures,
                full_fetch,
            ).await?;
        }

        Ok(())
    }

    pub async fn queue_fetch_characters(&self, anime_id: u32) -> Result<(), AppError> {
        let task = FetchCharactersTask::new(anime_id, self.jikan_client.clone());
        info!(module = "my_anime_list", anime_id = anime_id, "Queueing fetch characters task");
        self.queue.enqueue(Box::new(task)).await
    }

    pub async fn queue_fetch_staff(&self, anime_id: u32) -> Result<(), AppError> {
        let task = FetchStaffTask::new(anime_id, self.jikan_client.clone());
        info!(module = "my_anime_list", anime_id = anime_id, "Queueing fetch staff task");
        self.queue.enqueue(Box::new(task)).await
    }

    pub async fn queue_fetch_episodes(&self, anime_id: u32) -> Result<(), AppError> {
        let task = FetchEpisodesTask::new(anime_id, self.jikan_client.clone());
        info!(module = "my_anime_list", anime_id = anime_id, "Queueing fetch episodes task");
        self.queue.enqueue(Box::new(task)).await
    }

    pub async fn queue_fetch_videos(&self, anime_id: u32) -> Result<(), AppError> {
        let task = FetchVideosTask::new(anime_id, self.jikan_client.clone());
        info!(module = "my_anime_list", anime_id = anime_id, "Queueing fetch videos task");
        self.queue.enqueue(Box::new(task)).await
    }

    pub async fn queue_fetch_statistics(&self, anime_id: u32) -> Result<(), AppError> {
        let task = FetchStatisticsTask::new(anime_id, self.jikan_client.clone());
        info!(module = "my_anime_list", anime_id = anime_id, "Queueing fetch statistics task");
        self.queue.enqueue(Box::new(task)).await
    }

    pub async fn queue_fetch_more_info(&self, anime_id: u32) -> Result<(), AppError> {
        let task = FetchMoreInfoTask::new(anime_id, self.jikan_client.clone());
        info!(module = "my_anime_list", anime_id = anime_id, "Queueing fetch more info task");
        self.queue.enqueue(Box::new(task)).await
    }

    pub async fn queue_fetch_recommendations(&self, anime_id: u32) -> Result<(), AppError> {
        let task = FetchRecommendationsTask::new(anime_id, self.jikan_client.clone());
        info!(module = "my_anime_list", anime_id = anime_id, "Queueing fetch recommendations task");
        self.queue.enqueue(Box::new(task)).await
    }

    pub async fn queue_fetch_pictures(&self, anime_id: u32) -> Result<(), AppError> {
        let task = FetchPicturesTask::new(anime_id, self.jikan_client.clone());
        info!(module = "my_anime_list", anime_id = anime_id, "Queueing fetch pictures task");
        self.queue.enqueue(Box::new(task)).await
    }

    /// Fetch complete anime data (basic + extended + picture downloads)
    pub async fn queue_fetch_complete(&self, anime_id: u32, with_jikan: bool) -> Result<(), AppError> {
        info!(
            module = "my_anime_list",
            anime_id = anime_id,
            with_jikan = with_jikan,
            "Queueing complete anime fetch (basic + extended data + pictures)"
        );

        // Queue basic fetch with pictures
        self.queue_fetch_anime_with_pictures(anime_id, with_jikan).await?;

        // Queue extended data
        self.queue_fetch_all_extended_data(anime_id).await?;

        Ok(())
    }

    /// Queue all extended data tasks for an anime
    pub async fn queue_fetch_all_extended_data(&self, anime_id: u32) -> Result<(), AppError> {
        info!(module = "my_anime_list", anime_id = anime_id, "Queueing all extended data tasks");
        
        self.queue_fetch_characters(anime_id).await?;
        self.queue_fetch_staff(anime_id).await?;
        self.queue_fetch_episodes(anime_id).await?;
        self.queue_fetch_videos(anime_id).await?;
        self.queue_fetch_statistics(anime_id).await?;
        self.queue_fetch_more_info(anime_id).await?;
        self.queue_fetch_recommendations(anime_id).await?;
        self.queue_fetch_pictures(anime_id).await?;

        Ok(())
    }
}