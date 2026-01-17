// src/anime/my_anime_list/module.rs
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;

use super::model::AnimeData;
use crate::global::database::DatabaseInstance;
use crate::global::error::AppError;
use crate::global::http::{ClientWithLimiter, RequestConfig};
use crate::global::module::ChildModule;

#[derive(Debug, Clone)]
pub struct FetchAnimeInput {
    pub anime_id: u32,
}

pub struct MyAnimeListModule {
    client: ClientWithLimiter,
}

impl MyAnimeListModule {
    pub fn new(client: ClientWithLimiter) -> Self {
        Self { client }
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

            println!("[{}] Fetching anime {}", self.name(), input.anime_id);
            
            // In a real implementation, use the actual MyAnimeList API
            // Example URL: https://api.myanimelist.net/v2/anime/{id}
            let url = format!("https://api.myanimelist.net/v2/anime/{}?fields=id,title,main_picture,alternative_titles,start_date,end_date,synopsis,mean,rank,popularity,num_list_users,num_scoring_users,nsfw,genres,created_at,updated_at,media_type,status,num_episodes,start_season,broadcast,source,average_episode_duration,rating,studios,pictures,background,related_anime,related_manga,statistics", input.anime_id);
            let config = RequestConfig::new().with_header("X-MAL-CLIENT-ID", "4c88492f0f5cebd9d7cfdd640de1370e");
            
            // Use the custom fetch_json method with automatic retry
            let anime = self.client.fetch_json::<AnimeData>(&url, Some(config)).await?;
            
            println!("[{}] Successfully fetched: {}", self.name(), anime.title);
            
            // In production, you would also store in database here:
            // store_anime_in_db(db, &anime).await?;

            Ok(anime)
        })
    }
}