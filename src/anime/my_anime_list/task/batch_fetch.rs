use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::global::{
    database::DatabaseInstance,
    error::AppError,
    queue::{Task, TaskPriority, TaskData, TaskStatus},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchFetchPayload {
    pub anime_ids: Vec<u32>,
}

pub struct BatchFetchTask {
    id: String,
    anime_ids: Vec<u32>,
    api_key: String,
    client_with_limiter: crate::global::http::ClientWithLimiter,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl BatchFetchTask {
    pub fn new(
        anime_ids: Vec<u32>,
        api_key: String,
        client_with_limiter: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("mal_batch_{}", uuid::Uuid::new_v4());
        Self {
            id,
            anime_ids,
            api_key,
            client_with_limiter,
            created_at: chrono::Utc::now(),
        }
    }
}

#[async_trait::async_trait]
impl Task for BatchFetchTask {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> &str {
        "batch_fetch_mal"
    }

    fn priority(&self) -> TaskPriority {
        TaskPriority::Low  // Batch operations are lower priority
    }

    fn to_data(&self) -> TaskData {
        let payload = BatchFetchPayload {
            anime_ids: self.anime_ids.clone(),
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
            count = self.anime_ids.len(),
            "Batch fetching anime from MyAnimeList"
        );

        let mut successful = 0;
        let mut failed = 0;

        for anime_id in &self.anime_ids {
            // Create individual fetch task
            let fetch_task = super::fetch_anime::FetchAnimeTask::new(
                *anime_id,
                self.api_key.clone(),
                self.client_with_limiter.clone(),
            );

            match fetch_task.execute(db.clone(), _client.clone()).await {
                Ok(_) => successful += 1,
                Err(e) => {
                    tracing::warn!(anime_id = anime_id, error = %e, "Failed to fetch anime in batch");
                    failed += 1;
                }
            }
        }

        info!(
            task = %self.name(),
            total = self.anime_ids.len(),
            successful = successful,
            failed = failed,
            "Batch fetch completed"
        );

        Ok(())
    }
}