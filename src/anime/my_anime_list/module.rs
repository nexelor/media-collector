use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use tracing::{info, warn};

use super::model::AnimeData;
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
    client: ClientWithLimiter,
    config: Arc<AppConfig>,
    queue: TaskQueue,
}

impl MyAnimeListModule {
    /// Create a new MyAnimeList module
    /// Returns None if the module cannot be started due to missing configuration
    pub fn new(client: ClientWithLimiter, config: Arc<AppConfig>, queue: TaskQueue) -> Option<Self> {
        // Validate configuration - requires API key
        if !config.can_start_child_module("my_anime_list", true) {
            return None;
        }

        Some(Self { client, config, queue })
    }

    /// Check if this module is enabled and properly configured
    pub fn is_available(config: &AppConfig) -> bool {
        config.can_start_child_module("my_anime_list", true)
    }

    /// Queue a task to fetch anime by ID
    pub async fn queue_fetch_anime(&self, anime_id: u32) -> Result<(), AppError> {
        let api_key = self.config.get_api_key("my_anime_list")
            .expect("API key should be validated during module creation");

        let task = FetchAnimeTask::new(
            anime_id,
            api_key,
            self.client.clone(),
        );

        info!(
            module = "my_anime_list",
            anime_id = anime_id,
            "Queueing fetch anime task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue a task to search for anime
    pub async fn queue_search_anime(&self, query: String, limit: Option<u32>) -> Result<(), AppError> {
        let api_key = self.config.get_api_key("my_anime_list")
            .expect("API key should be validated during module creation");

        let task = super::task::search_anime::SearchAnimeTask::new(
            query.clone(),
            limit,
            api_key,
            self.client.clone(),
        );

        info!(
            module = "my_anime_list",
            query = %query,
            "Queueing search anime task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue a task to update an existing anime
    pub async fn queue_update_anime(&self, anime_id: u32) -> Result<(), AppError> {
        let api_key = self.config.get_api_key("my_anime_list")
            .expect("API key should be validated during module creation");

        let task = super::task::update_anime::UpdateAnimeTask::new(
            anime_id,
            api_key,
            self.client.clone(),
        );

        info!(
            module = "my_anime_list",
            anime_id = anime_id,
            "Queueing update anime task"
        );

        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue a batch fetch task
    pub async fn queue_batch_fetch(&self, anime_ids: Vec<u32>) -> Result<(), AppError> {
        let api_key = self.config.get_api_key("my_anime_list")
            .expect("API key should be validated during module creation");

        let task = super::task::batch_fetch::BatchFetchTask::new(
            anime_ids.clone(),
            api_key,
            self.client.clone(),
        );

        info!(
            module = "my_anime_list",
            count = anime_ids.len(),
            "Queueing batch fetch task"
        );

        self.queue.enqueue(Box::new(task)).await
    }
}

impl ChildModule for MyAnimeListModule {
    type Input = FetchAnimeInput;
    type Output = AnimeData;
    
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

            info!(
                module = %self.name(),
                anime_id = input.anime_id,
                "Fetching anime from MyAnimeList API"
            );
            
            // In a real implementation, use the actual MyAnimeList API
            // Example URL: https://api.myanimelist.net/v2/anime/{id}
            let url = format!("https://api.myanimelist.net/v2/anime/{}?fields=id,title,main_picture,alternative_titles,start_date,end_date,synopsis,mean,rank,popularity,num_list_users,num_scoring_users,nsfw,genres,created_at,updated_at,media_type,status,num_episodes,start_season,broadcast,source,average_episode_duration,rating,studios,pictures,background,related_anime,related_manga,statistics", input.anime_id);
            
            // Get API key from config
            let api_key = self.config.get_api_key("my_anime_list")
                .expect("API key should be validated during module creation");

            let config = RequestConfig::new().with_header("X-MAL-CLIENT-ID", api_key);
            
            // Use the custom fetch_json method with automatic retry
            let anime = self.client.fetch_json::<AnimeData>(&url, Some(config)).await?;
            
            info!(
                module = %self.name(),
                anime_id = anime.id,
                title = %anime.title,
                score = ?anime.score,
                "Successfully fetched anime"
            );
            
            // In production, you would also store in database here:
            // store_anime_in_db(db, &anime).await?;

            Ok(anime)
        })
    }
}