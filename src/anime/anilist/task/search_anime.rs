use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{anime::anilist::database::upsert_anime, global::{
    database::DatabaseInstance, error::AppError, http::RequestConfig, queue::{Task, TaskData, TaskPriority, TaskStatus}
}};
use crate::anime::anilist::{
    model::{GraphQLRequest, GraphQLResponse, PageData},
    converter::anilist_to_anime_data,
    queries,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchAnimePayload {
    pub query: String,
    pub page: u32,
    pub per_page: u32,
}

pub struct SearchAnimeTask {
    id: String,
    query: String,
    page: u32,
    per_page: u32,
    client: crate::global::http::ClientWithLimiter,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl SearchAnimeTask {
    pub fn new(
        query: String,
        page: Option<u32>,
        per_page: Option<u32>,
        client: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("anilist_search_{}", uuid::Uuid::new_v4());
        Self {
            id,
            query,
            page: page.unwrap_or(1),
            per_page: per_page.unwrap_or(10),
            client,
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
        "search_anime_anilist"
    }

    fn priority(&self) -> TaskPriority {
        TaskPriority::Normal
    }

    fn to_data(&self) -> TaskData {
        let payload = SearchAnimePayload {
            query: self.query.clone(),
            page: self.page,
            per_page: self.per_page,
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
            page = self.page,
            per_page = self.per_page,
            "Searching anime on AniList"
        );

        let graphql_request = GraphQLRequest {
            query: queries::SEARCH_ANIME_QUERY.to_string(),
            variables: Some(serde_json::json!({
                "search": self.query,
                "page": self.page,
                "perPage": self.per_page
            })),
        };

        let url = "https://graphql.anilist.co";
        
        let config = RequestConfig::new()
            .with_header("Content-Type", "application/json")
            .with_header("Accept", "application/json");

        let response = self.client.client
            .post(url)
            .json(&graphql_request)
            .send()
            .await
            .map_err(|e| AppError::Module(format!("AniList API request failed: {}", e)))?;

        let graphql_response = response
            .json::<GraphQLResponse<PageData>>()
            .await
            .map_err(|e| AppError::Module(format!("Failed to parse AniList response: {}", e)))?;

        if !graphql_response.errors.is_empty() {
            let error_messages: Vec<String> = graphql_response.errors
                .iter()
                .map(|e| e.message.clone())
                .collect();
            return Err(AppError::Module(format!(
                "AniList GraphQL errors: {}",
                error_messages.join(", ")
            )));
        }

        let page_data = graphql_response.data
            .ok_or_else(|| AppError::Module("No data returned from AniList".to_string()))?;

        info!(
            task = %self.name(),
            query = %self.query,
            results = page_data.page.media.len(),
            "Search completed"
        );

        // Convert and store results
        for media in page_data.page.media {
            let anime_data = anilist_to_anime_data(media);
            upsert_anime(db.db(), &anime_data).await?;
        }

        Ok(())
    }
}