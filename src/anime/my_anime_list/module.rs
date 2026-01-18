use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use tracing::{info, warn};

use super::model::AnimeData;
use crate::anime::my_anime_list::task::{BatchFetchTask, FetchCharactersTask, FetchEpisodesTask, FetchMoreInfoTask, FetchPicturesTask, FetchRecommendationsTask, FetchStaffTask, FetchStatisticsTask, FetchVideosTask, SearchAnimeTask, UpdateAnimeTask};
use crate::anime::my_anime_list::task::fetch_anime::FetchAnimeTask;
use crate::global::config::AppConfig;
use crate::global::database::DatabaseInstance;
use crate::global::error::AppError;
use crate::global::http::{ClientWithLimiter, RequestConfig};
use crate::global::module::ChildModule;
use crate::global::queue::TaskQueue;

#[derive(Debug, Clone)]
pub struct FetchAnimeInput {
    pub anime_id: u32,
}

pub struct MyAnimeListModule {
    mal_client: ClientWithLimiter,
    jikan_client: ClientWithLimiter,
    config: Arc<AppConfig>,
    queue: TaskQueue,
}

impl MyAnimeListModule {
    /// Create a new MyAnimeList module
    /// Returns None if the module cannot be started due to missing configuration
    pub fn new(
        mal_client: ClientWithLimiter,
        jikan_client: ClientWithLimiter,  // NEW parameter
        config: Arc<AppConfig>,
        queue: TaskQueue,
    ) -> Option<Self> {
        if !config.can_start_child_module("my_anime_list", true) {
            return None;
        }

        Some(Self { 
            mal_client,    // CHANGED
            jikan_client,  // NEW
            config, 
            queue 
        })
    }

    /// Check if this module is enabled and properly configured
    pub fn is_available(config: &AppConfig) -> bool {
        config.can_start_child_module("my_anime_list", true)
    }

    /// Queue a task to fetch anime by ID
    pub async fn queue_fetch_anime(&self, anime_id: u32, with_jikan: bool) -> Result<(), AppError> {
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

        info!(
            module = "my_anime_list",
            anime_id = anime_id,
            with_jikan = with_jikan,
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
    pub async fn queue_batch_fetch(&self, anime_ids: Vec<u32>, with_jikan: bool) -> Result<(), AppError> {
        let api_key = self.config.get_api_key("my_anime_list")
            .expect("API key should be validated during module creation");

        let mut task = BatchFetchTask::new(
            anime_ids.clone(),
            api_key,
            self.mal_client.clone(),
            self.jikan_client.clone(),
        );

        if with_jikan {
            task = task.with_jikan();
        }

        info!(
            module = "my_anime_list",
            count = anime_ids.len(),
            with_jikan = with_jikan,
            "Queueing batch fetch task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue a task to fetch characters for an anime
    pub async fn queue_fetch_characters(&self, anime_id: u32) -> Result<(), AppError> {
        let task = FetchCharactersTask::new(
            anime_id,
            self.jikan_client.clone(),
        );

        info!(
            module = "my_anime_list",
            anime_id = anime_id,
            "Queueing fetch characters task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue a task to fetch staff for an anime
    pub async fn queue_fetch_staff(&self, anime_id: u32) -> Result<(), AppError> {
        let task = FetchStaffTask::new(
            anime_id,
            self.jikan_client.clone(),
        );

        info!(
            module = "my_anime_list",
            anime_id = anime_id,
            "Queueing fetch staff task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue a task to fetch episodes for an anime
    pub async fn queue_fetch_episodes(&self, anime_id: u32) -> Result<(), AppError> {
        let task = FetchEpisodesTask::new(
            anime_id,
            self.jikan_client.clone(),
        );

        info!(
            module = "my_anime_list",
            anime_id = anime_id,
            "Queueing fetch episodes task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Fetch complete anime data (basic + extended)
    /// This will queue the basic fetch task and all extended data tasks
    pub async fn queue_fetch_complete(&self, anime_id: u32, with_jikan: bool) -> Result<(), AppError> {
        info!(
            module = "my_anime_list",
            anime_id = anime_id,
            with_jikan = with_jikan,
            "Queueing complete anime fetch (basic + extended data)"
        );

        // Queue basic fetch
        self.queue_fetch_anime(anime_id, with_jikan).await?;

        // Queue extended data (these will execute after basic fetch due to lower priority)
        self.queue_fetch_all_extended_data(anime_id).await?;

        Ok(())
    }

    /// Queue a task to fetch videos for an anime
    pub async fn queue_fetch_videos(&self, anime_id: u32) -> Result<(), AppError> {
        let task = FetchVideosTask::new(
            anime_id,
            self.jikan_client.clone(),
        );

        info!(
            module = "my_anime_list",
            anime_id = anime_id,
            "Queueing fetch videos task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue a task to fetch statistics for an anime
    pub async fn queue_fetch_statistics(&self, anime_id: u32) -> Result<(), AppError> {
        let task = FetchStatisticsTask::new(
            anime_id,
            self.jikan_client.clone(),
        );

        info!(
            module = "my_anime_list",
            anime_id = anime_id,
            "Queueing fetch statistics task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue a task to fetch more info for an anime
    pub async fn queue_fetch_more_info(&self, anime_id: u32) -> Result<(), AppError> {
        let task = FetchMoreInfoTask::new(
            anime_id,
            self.jikan_client.clone(),
        );

        info!(
            module = "my_anime_list",
            anime_id = anime_id,
            "Queueing fetch more info task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue a task to fetch recommendations for an anime
    pub async fn queue_fetch_recommendations(&self, anime_id: u32) -> Result<(), AppError> {
        let task = FetchRecommendationsTask::new(
            anime_id,
            self.jikan_client.clone(),
        );

        info!(
            module = "my_anime_list",
            anime_id = anime_id,
            "Queueing fetch recommendations task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue a task to fetch pictures for an anime
    pub async fn queue_fetch_pictures(&self, anime_id: u32) -> Result<(), AppError> {
        let task = FetchPicturesTask::new(
            anime_id,
            self.jikan_client.clone(),
        );

        info!(
            module = "my_anime_list",
            anime_id = anime_id,
            "Queueing fetch pictures task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue all extended data tasks for an anime (updated version)
    pub async fn queue_fetch_all_extended_data(&self, anime_id: u32) -> Result<(), AppError> {
        info!(
            module = "my_anime_list",
            anime_id = anime_id,
            "Queueing all extended data tasks"
        );

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

impl ChildModule for MyAnimeListModule {
    type Input = FetchAnimeInput;
    // type Output = AnimeData;
    type Output = ();
    
    fn name(&self) -> &str {
        "my_anime_list"
    }
    
    fn execute(&self, db: Arc<DatabaseInstance>, client: reqwest::Client, input: Self::Input)
        -> Pin<Box<dyn Future<Output = Result<Self::Output, AppError>> + Send + '_>>
    {
        Box::pin(async move {
            // println!("[{}] Fetching anime {}", self.name(), input.anime_id);
            
            // // Example: Make actual API request to MyAnimeList
            // // let url = format!("https://api.myanimelist.net/v2/anime/{}", input.anime_id);
            // // let response = client
            // //     .get(&url)
            // //     .header("X-MAL-CLIENT-ID", "your_client_id")
            // //     .send()
            // //     .await
            // //     .map_err(|e| AppError::Module(format!("API request failed: {}", e)))?;
            
            // // Simulate API call delay
            // tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            // // tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
            
            // // In a real implementation, you would:
            // // 1. Check database first
            // // 2. If not found, fetch from API using the client
            // // 3. Store in database
            // // 4. Return data
            
            // let anime = AnimeData {
            //     id: input.anime_id,
            //     title: format!("Anime {}", input.anime_id),
            //     score: Some(8.5),
            // };
            
            // println!("[{}] Successfully fetched: {}", self.name(), anime.title);
            
            // Ok(anime)

            // info!(
            //     module = %self.name(),
            //     anime_id = input.anime_id,
            //     "Fetching anime from MyAnimeList API"
            // );
            
            // // In a real implementation, use the actual MyAnimeList API
            // // Example URL: https://api.myanimelist.net/v2/anime/{id}
            // let url = format!("https://api.myanimelist.net/v2/anime/{}?fields=id,title,main_picture,alternative_titles,start_date,end_date,synopsis,mean,rank,popularity,num_list_users,num_scoring_users,nsfw,genres,created_at,updated_at,media_type,status,num_episodes,start_season,broadcast,source,average_episode_duration,rating,studios,pictures,background,related_anime,related_manga,statistics", input.anime_id);
            
            // // Get API key from config
            // let api_key = self.config.get_api_key("my_anime_list")
            //     .expect("API key should be validated during module creation");

            // let config = RequestConfig::new().with_header("X-MAL-CLIENT-ID", api_key);
            
            // // Use the custom fetch_json method with automatic retry
            // let anime = self.client.fetch_json::<AnimeData>(&url, Some(config)).await?;
            
            // info!(
            //     module = %self.name(),
            //     anime_id = anime.id,
            //     title = %anime.title,
            //     score = ?anime.score,
            //     "Successfully fetched anime"
            // );
            
            // In production, you would also store in database here:
            // store_anime_in_db(db, &anime).await?;

            // Ok(anime)
            Ok(())
        })
    }
}