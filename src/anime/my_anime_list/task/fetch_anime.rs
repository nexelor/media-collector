use std::sync::Arc;
use tracing::{info, debug};

use crate::global::queue::{TaskData, TaskPriority, TaskStatus};
use crate::global::{
    database::DatabaseInstance,
    error::AppError,
    queue::Task,
    http::RequestConfig,
};
use crate::anime::my_anime_list::{model::AnimeData, database::insert_anime};

/// Task to fetch anime data from MyAnimeList API
pub struct FetchAnimeTask {
    id: String,
    anime_id: u32,
    api_key: String,
    client_with_limiter: crate::global::http::ClientWithLimiter,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl FetchAnimeTask {
    pub fn new(
        anime_id: u32,
        api_key: String,
        client_with_limiter: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("mal_search_{}", uuid::Uuid::new_v4());
        Self {
            id,
            anime_id,
            api_key,
            client_with_limiter,
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
        "fetch_anime_mal"
    }

    fn priority(&self) -> TaskPriority {
        TaskPriority::Normal
    }

    fn to_data(&self) -> TaskData {
        TaskData {
            id: self.id(),
            name: self.name().to_string(),
            priority: self.priority(),
            status: TaskStatus::Pending,
            created_at: self.created_at,
            payload: serde_json::json!({ "anime_id": self.anime_id }),
        }
    }

    async fn execute(&self, db: Arc<DatabaseInstance>, _client: reqwest::Client) -> Result<(), AppError> {
        info!(
            task = %self.name(),
            anime_id = self.anime_id,
            "Fetching anime from MyAnimeList API"
        );

        let url = format!(
            "https://api.myanimelist.net/v2/anime/{}?fields=id,title,main_picture,alternative_titles,start_date,end_date,synopsis,mean,rank,popularity,num_list_users,num_scoring_users,nsfw,genres,created_at,updated_at,media_type,status,num_episodes,start_season,broadcast,source,average_episode_duration,rating,studios,pictures,background,related_anime,related_manga,statistics",
            self.anime_id
        );

        let config = RequestConfig::new().with_header("X-MAL-CLIENT-ID", &self.api_key);

        // Use the client with rate limiter
        let anime = self.client_with_limiter
            .fetch_json::<AnimeData>(&url, Some(config))
            .await?;

        info!(
            task = %self.name(),
            anime_id = anime.id,
            title = %anime.title,
            score = ?anime.score,
            "Successfully fetched anime"
        );

        // Store in database
        debug!(task = %self.name(), anime_id = anime.id, "Storing anime in database");
        insert_anime(db.db(), &anime).await?;
        debug!(task = %self.name(), anime_id = anime.id, "Anime stored successfully");

        Ok(())
    }
}