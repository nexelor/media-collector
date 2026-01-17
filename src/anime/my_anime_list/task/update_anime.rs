use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::global::{
    database::DatabaseInstance,
    error::AppError,
    queue::{Task, TaskPriority, TaskData, TaskStatus},
    http::RequestConfig,
};
use crate::anime::my_anime_list::model::AnimeData;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAnimePayload {
    pub anime_id: u32,
}

pub struct UpdateAnimeTask {
    id: String,
    anime_id: u32,
    api_key: String,
    client_with_limiter: crate::global::http::ClientWithLimiter,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl UpdateAnimeTask {
    pub fn new(
        anime_id: u32,
        api_key: String,
        client_with_limiter: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("mal_update_{}", anime_id);
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
impl Task for UpdateAnimeTask {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> &str {
        "update_anime_mal"
    }

    fn priority(&self) -> TaskPriority {
        TaskPriority::High  // Updates are higher priority
    }

    fn to_data(&self) -> TaskData {
        let payload = UpdateAnimePayload {
            anime_id: self.anime_id,
        };

        TaskData {
            id: self.id.clone(),
            name: self.name().to_string(),
            priority: self.priority(),
            status: TaskStatus::Pending,
            created_at: self.created_at,
            payload: serde_json::to_value(payload).unwrap(),
        }
    }

    async fn execute(&self, db: Arc<DatabaseInstance>, _client: reqwest::Client) -> Result<(), AppError> {
        info!(
            task = %self.name(),
            anime_id = self.anime_id,
            "Updating anime from MyAnimeList"
        );

        let url = format!(
            "https://api.myanimelist.net/v2/anime/{}?fields=id,title,main_picture,alternative_titles,start_date,end_date,synopsis,mean,rank,popularity,num_list_users,num_scoring_users,nsfw,genres,created_at,updated_at,media_type,status,num_episodes,start_season,broadcast,source,average_episode_duration,rating,studios,pictures,background,related_anime,related_manga,statistics",
            self.anime_id
        );

        let config = RequestConfig::new().with_header("X-MAL-CLIENT-ID", &self.api_key);

        let anime = self.client_with_limiter
            .fetch_json::<AnimeData>(&url, Some(config))
            .await?;

        info!(
            task = %self.name(),
            anime_id = anime.id,
            title = %anime.title,
            "Update completed"
        );

        // Update in database (replace existing)
        use mongodb::bson::doc;
        let collection = db.db().collection::<AnimeData>("anime");
        let filter = doc! { "id": self.anime_id };
        let options = mongodb::options::ReplaceOptions::builder().upsert(true).build();
        
        collection.replace_one(filter, anime).with_options(options)
            .await
            .map_err(|e| AppError::Module(format!("Failed to update anime: {}", e)))?;

        Ok(())
    }
}