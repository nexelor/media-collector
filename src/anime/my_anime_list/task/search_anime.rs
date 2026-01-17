use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::global::{
    database::DatabaseInstance,
    error::AppError,
    queue::{Task, TaskPriority, TaskData, TaskStatus},
    http::RequestConfig,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAnimePayload {
    pub query: String,
    pub limit: Option<u32>,
}

pub struct SearchAnimeTask {
    id: String,
    query: String,
    limit: u32,
    api_key: String,
    client_with_limiter: crate::global::http::ClientWithLimiter,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl SearchAnimeTask {
    pub fn new(
        query: String,
        limit: Option<u32>,
        api_key: String,
        client_with_limiter: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("mal_search_{}", uuid::Uuid::new_v4());
        Self {
            id,
            query,
            limit: limit.unwrap_or(10),
            api_key,
            client_with_limiter,
            created_at: chrono::Utc::now(),
        }
    }
}

#[async_trait::async_trait]
impl Task for SearchAnimeTask {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> &str {
        "search_anime_mal"
    }

    fn priority(&self) -> TaskPriority {
        TaskPriority::Normal
    }

    fn to_data(&self) -> TaskData {
        let payload = SearchAnimePayload {
            query: self.query.clone(),
            limit: Some(self.limit),
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
            query = %self.query,
            limit = self.limit,
            "Searching anime on MyAnimeList"
        );

        let url = format!(
            "https://api.myanimelist.net/v2/anime?q={}&limit={}&fields=id,title,main_picture,alternative_titles,start_date,end_date,synopsis,mean,rank,popularity,num_list_users,num_scoring_users,nsfw,genres,created_at,updated_at,media_type,status,num_episodes,start_season,broadcast,source,average_episode_duration,rating,studios,pictures,background,related_anime,related_manga,statistics",
            urlencoding::encode(&self.query),
            self.limit
        );

        let config = RequestConfig::new().with_header("X-MAL-CLIENT-ID", &self.api_key);

        #[derive(Deserialize)]
        struct SearchResponse {
            data: Vec<SearchResult>,
        }

        #[derive(Deserialize)]
        struct SearchResult {
            node: crate::anime::my_anime_list::model::MalAnimeResponse,
        }

        let response = self.client_with_limiter
            .fetch_json::<SearchResponse>(&url, Some(config))
            .await?;

        info!(
            task = %self.name(),
            query = %self.query,
            results = response.data.len(),
            "Search completed"
        );

        // Convert and store results in database (anime_mal collection)
        for result in response.data {
            let anime_data = crate::anime::my_anime_list::converter::mal_to_anime_data(
                result.node,
                None
            );
            crate::anime::my_anime_list::database::insert_anime(db.db(), &anime_data).await?;
        }

        Ok(())
    }
}