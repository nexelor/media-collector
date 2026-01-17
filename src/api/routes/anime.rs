use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::api::state::ApiState;
use crate::anime::my_anime_list;

// ========================================================================
// Request/Response Types
// ========================================================================

#[derive(Debug, Deserialize)]
pub struct FetchAnimeRequest {
    pub anime_id: u32,
    #[serde(default)]
    pub with_jikan: bool,
}

#[derive(Debug, Deserialize)]
pub struct SearchAnimeRequest {
    pub query: String,
    #[serde(default = "default_limit")]
    pub limit: u32,
}

fn default_limit() -> u32 {
    10
}

#[derive(Debug, Deserialize)]
pub struct UpdateAnimeRequest {
    pub anime_id: u32,
    #[serde(default)]
    pub with_jikan: bool,
}

#[derive(Debug, Deserialize)]
pub struct BatchFetchRequest {
    pub anime_ids: Vec<u32>,
    #[serde(default)]
    pub with_jikan: bool,
}

#[derive(Debug, Deserialize)]
pub struct FetchExtendedDataRequest {
    pub anime_id: u32,
    #[serde(default)]
    pub fetch_characters: bool,
    #[serde(default)]
    pub fetch_staff: bool,
    #[serde(default)]
    pub fetch_episodes: bool,
}

#[derive(Serialize)]
pub struct TaskQueuedResponse {
    pub message: String,
    pub task_type: String,
}

#[derive(Serialize)]
pub struct AnimeResponse {
    pub anime: my_anime_list::model::AnimeData,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ========================================================================
// Handlers
// ========================================================================

/// Fetch anime from MyAnimeList
/// POST /api/anime/fetch
/// Body: { "anime_id": 1, "with_jikan": true }
pub async fn fetch_anime(
    State(state): State<ApiState>,
    Json(request): Json<FetchAnimeRequest>,
) -> Result<Json<TaskQueuedResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        anime_id = request.anime_id,
        with_jikan = request.with_jikan,
        "API request: fetch anime"
    );

    let anime_module = state.anime_module.as_ref()
        .ok_or_else(|| {
            error!("Anime module not available");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Anime module is not enabled".to_string(),
                })
            )
        })?;

    // Get MyAnimeList module
    let mal_client = state.http_manager.my_anime_list().clone();
    let jikan_client = state.http_manager.jikan().clone();
    
    let mal_module = my_anime_list::module::MyAnimeListModule::new(
        mal_client,
        jikan_client,
        state.config.clone(),
        anime_module.queue().clone(),
    ).ok_or_else(|| {
        error!("MyAnimeList module not configured");
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "MyAnimeList module is not properly configured".to_string(),
            })
        )
    })?;

    // Queue the task
    mal_module
        .queue_fetch_anime(request.anime_id, request.with_jikan)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to queue fetch anime task");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to queue task: {}", e),
                })
            )
        })?;

    Ok(Json(TaskQueuedResponse {
        message: format!("Anime {} queued for fetching", request.anime_id),
        task_type: "fetch_anime".to_string(),
    }))
}

/// Search anime on MyAnimeList
/// POST /api/anime/search
/// Body: { "query": "naruto", "limit": 10 }
pub async fn search_anime(
    State(state): State<ApiState>,
    Json(request): Json<SearchAnimeRequest>,
) -> Result<Json<TaskQueuedResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        query = %request.query,
        limit = request.limit,
        "API request: search anime"
    );

    let anime_module = state.anime_module.as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Anime module is not enabled".to_string(),
                })
            )
        })?;

    let mal_client = state.http_manager.my_anime_list().clone();
    let jikan_client = state.http_manager.jikan().clone();
    
    let mal_module = my_anime_list::module::MyAnimeListModule::new(
        mal_client,
        jikan_client,
        state.config.clone(),
        anime_module.queue().clone(),
    ).ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "MyAnimeList module is not properly configured".to_string(),
            })
        )
    })?;

    mal_module
        .queue_search_anime(request.query.clone(), Some(request.limit))
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to queue search task");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to queue task: {}", e),
                })
            )
        })?;

    Ok(Json(TaskQueuedResponse {
        message: format!("Search for '{}' queued", request.query),
        task_type: "search_anime".to_string(),
    }))
}

/// Update existing anime data
/// POST /api/anime/update
/// Body: { "anime_id": 1, "with_jikan": true }
pub async fn update_anime(
    State(state): State<ApiState>,
    Json(request): Json<UpdateAnimeRequest>,
) -> Result<Json<TaskQueuedResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        anime_id = request.anime_id,
        with_jikan = request.with_jikan,
        "API request: update anime"
    );

    let anime_module = state.anime_module.as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Anime module is not enabled".to_string(),
                })
            )
        })?;

    let mal_client = state.http_manager.my_anime_list().clone();
    let jikan_client = state.http_manager.jikan().clone();
    
    let mal_module = my_anime_list::module::MyAnimeListModule::new(
        mal_client,
        jikan_client,
        state.config.clone(),
        anime_module.queue().clone(),
    ).ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "MyAnimeList module is not properly configured".to_string(),
            })
        )
    })?;

    mal_module
        .queue_update_anime(request.anime_id, request.with_jikan)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to queue update task");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to queue task: {}", e),
                })
            )
        })?;

    Ok(Json(TaskQueuedResponse {
        message: format!("Anime {} queued for update", request.anime_id),
        task_type: "update_anime".to_string(),
    }))
}

/// Batch fetch multiple anime
/// POST /api/anime/batch
/// Body: { "anime_ids": [1, 2, 3], "with_jikan": true }
pub async fn batch_fetch(
    State(state): State<ApiState>,
    Json(request): Json<BatchFetchRequest>,
) -> Result<Json<TaskQueuedResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        count = request.anime_ids.len(),
        with_jikan = request.with_jikan,
        "API request: batch fetch anime"
    );

    let anime_module = state.anime_module.as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Anime module is not enabled".to_string(),
                })
            )
        })?;

    let mal_client = state.http_manager.my_anime_list().clone();
    let jikan_client = state.http_manager.jikan().clone();
    
    let mal_module = my_anime_list::module::MyAnimeListModule::new(
        mal_client,
        jikan_client,
        state.config.clone(),
        anime_module.queue().clone(),
    ).ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "MyAnimeList module is not properly configured".to_string(),
            })
        )
    })?;

    mal_module
        .queue_batch_fetch(request.anime_ids.clone(), request.with_jikan)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to queue batch fetch task");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to queue task: {}", e),
                })
            )
        })?;

    Ok(Json(TaskQueuedResponse {
        message: format!("{} anime queued for batch fetching", request.anime_ids.len()),
        task_type: "batch_fetch".to_string(),
    }))
}

/// Fetch extended data (characters, staff, episodes)
/// POST /api/anime/extended
/// Body: { "anime_id": 1, "fetch_characters": true, "fetch_staff": true, "fetch_episodes": true }
pub async fn fetch_extended_data(
    State(state): State<ApiState>,
    Json(request): Json<FetchExtendedDataRequest>,
) -> Result<Json<TaskQueuedResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        anime_id = request.anime_id,
        characters = request.fetch_characters,
        staff = request.fetch_staff,
        episodes = request.fetch_episodes,
        "API request: fetch extended data"
    );

    let anime_module = state.anime_module.as_ref()
        .ok_or_else(|| {
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Anime module is not enabled".to_string(),
                })
            )
        })?;

    let mal_client = state.http_manager.my_anime_list().clone();
    let jikan_client = state.http_manager.jikan().clone();
    
    let mal_module = my_anime_list::module::MyAnimeListModule::new(
        mal_client,
        jikan_client,
        state.config.clone(),
        anime_module.queue().clone(),
    ).ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse {
                error: "MyAnimeList module is not properly configured".to_string(),
            })
        )
    })?;

    let mut tasks_queued = Vec::new();

    if request.fetch_characters {
        mal_module.queue_fetch_characters(request.anime_id).await
            .map_err(|e| {
                error!(error = %e, "Failed to queue characters task");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to queue characters: {}", e),
                    })
                )
            })?;
        tasks_queued.push("characters");
    }

    if request.fetch_staff {
        mal_module.queue_fetch_staff(request.anime_id).await
            .map_err(|e| {
                error!(error = %e, "Failed to queue staff task");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to queue staff: {}", e),
                    })
                )
            })?;
        tasks_queued.push("staff");
    }

    if request.fetch_episodes {
        mal_module.queue_fetch_episodes(request.anime_id).await
            .map_err(|e| {
                error!(error = %e, "Failed to queue episodes task");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to queue episodes: {}", e),
                    })
                )
            })?;
        tasks_queued.push("episodes");
    }

    if tasks_queued.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "No data types selected for fetching".to_string(),
            })
        ));
    }

    Ok(Json(TaskQueuedResponse {
        message: format!("Extended data ({}) queued for anime {}", tasks_queued.join(", "), request.anime_id),
        task_type: "fetch_extended_data".to_string(),
    }))
}

/// Get anime by ID from database
/// GET /api/anime/:id
pub async fn get_anime(
    State(state): State<ApiState>,
    Path(anime_id): Path<i32>,
) -> Result<Json<AnimeResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(anime_id = anime_id, "API request: get anime");

    let anime = my_anime_list::database::get_anime_by_id(state.db.db(), anime_id)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to get anime from database");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                })
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(ErrorResponse {
                    error: format!("Anime {} not found", anime_id),
                })
            )
        })?;

    Ok(Json(AnimeResponse { anime }))
}