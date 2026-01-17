use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::api::state::ApiState;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize)]
pub struct StatsResponse {
    database: DatabaseStats,
    modules: ModuleStats,
}

#[derive(Serialize)]
struct DatabaseStats {
    pending_tasks: u64,
    running_tasks: u64,
    completed_tasks: u64,
    failed_tasks: u64,
}

#[derive(Serialize)]
struct ModuleStats {
    anime_enabled: bool,
    picture_enabled: bool,
}

/// Health check endpoint
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Get application statistics
pub async fn get_stats(
    State(state): State<ApiState>,
) -> Result<Json<StatsResponse>, StatusCode> {
    let db_stats = state.db.get_stats().await
        .map_err(|e| {
            error!(error = %e, "Failed to get database stats");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let response = StatsResponse {
        database: DatabaseStats {
            pending_tasks: db_stats.pending_tasks,
            running_tasks: db_stats.running_tasks,
            completed_tasks: db_stats.completed_tasks,
            failed_tasks: db_stats.failed_tasks,
        },
        modules: ModuleStats {
            anime_enabled: state.anime_module.is_some(),
            picture_enabled: state.picture_module.is_some(),
        },
    };

    Ok(Json(response))
}