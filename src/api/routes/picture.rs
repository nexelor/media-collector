use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::api::state::ApiState;
use crate::picture::{database, model::PictureStats};

// ========================================================================
// Request/Response Types
// ========================================================================

#[derive(Debug, Deserialize)]
pub struct FetchPictureRequest {
    pub url: String,
    pub filename: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BatchFetchPicturesRequest {
    pub urls: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GetPicturesQuery {
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub tag: Option<String>,
    pub status: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_limit() -> i64 {
    50
}

#[derive(Serialize)]
pub struct TaskQueuedResponse {
    pub message: String,
    pub task_type: String,
}

#[derive(Serialize)]
pub struct PictureResponse {
    pub picture: crate::picture::model::PictureMetadata,
}

#[derive(Serialize)]
pub struct PicturesResponse {
    pub pictures: Vec<crate::picture::model::PictureMetadata>,
    pub count: usize,
}

#[derive(Serialize)]
pub struct StatsResponse {
    pub stats: PictureStats,
}

#[derive(Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

// ========================================================================
// Handlers
// ========================================================================

/// Fetch a picture from URL
/// POST /api/picture/fetch
/// Body: { "url": "https://example.com/image.jpg", "filename": "custom_name.jpg", "tags": ["anime", "cover"], "entity_type": "anime", "entity_id": "123" }
pub async fn fetch_picture(
    State(state): State<ApiState>,
    Json(request): Json<FetchPictureRequest>,
) -> Result<Json<TaskQueuedResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        url = %request.url,
        filename = ?request.filename,
        tags = ?request.tags,
        "API request: fetch picture"
    );

    let picture_module = state.picture_module.as_ref()
        .ok_or_else(|| {
            error!("Picture module not available");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Picture module is not enabled".to_string(),
                })
            )
        })?;

    // Queue with entity and tags if provided
    if let (Some(entity_type), Some(entity_id)) = (request.entity_type, request.entity_id) {
        picture_module
            .queue_fetch_picture_for_entity(
                request.url.clone(),
                request.filename,
                entity_type,
                entity_id,
                request.tags,
            )
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to queue picture fetch task");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to queue task: {}", e),
                    })
                )
            })?;
    } else if !request.tags.is_empty() {
        picture_module
            .queue_fetch_picture_with_tags(
                request.url.clone(),
                request.filename,
                request.tags,
            )
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to queue picture fetch task");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to queue task: {}", e),
                    })
                )
            })?;
    } else {
        picture_module
            .queue_fetch_picture(request.url.clone(), request.filename)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to queue picture fetch task");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to queue task: {}", e),
                    })
                )
            })?;
    }

    Ok(Json(TaskQueuedResponse {
        message: format!("Picture queued for fetching: {}", request.url),
        task_type: "fetch_picture".to_string(),
    }))
}

/// Batch fetch multiple pictures
/// POST /api/picture/batch
/// Body: { "urls": ["https://example.com/1.jpg", "https://example.com/2.jpg"], "tags": ["anime"] }
pub async fn batch_fetch(
    State(state): State<ApiState>,
    Json(request): Json<BatchFetchPicturesRequest>,
) -> Result<Json<TaskQueuedResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        count = request.urls.len(),
        tags = ?request.tags,
        "API request: batch fetch pictures"
    );

    if request.urls.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "No URLs provided".to_string(),
            })
        ));
    }

    let picture_module = state.picture_module.as_ref()
        .ok_or_else(|| {
            error!("Picture module not available");
            (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(ErrorResponse {
                    error: "Picture module is not enabled".to_string(),
                })
            )
        })?;

    // Queue each picture
    for url in &request.urls {
        if !request.tags.is_empty() {
            picture_module
                .queue_fetch_picture_with_tags(url.clone(), None, request.tags.clone())
                .await
                .map_err(|e| {
                    error!(error = %e, url = %url, "Failed to queue picture");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: format!("Failed to queue picture {}: {}", url, e),
                        })
                    )
                })?;
        } else {
            picture_module
                .queue_fetch_picture(url.clone(), None)
                .await
                .map_err(|e| {
                    error!(error = %e, url = %url, "Failed to queue picture");
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: format!("Failed to queue picture {}: {}", url, e),
                        })
                    )
                })?;
        }
    }

    Ok(Json(TaskQueuedResponse {
        message: format!("{} pictures queued for fetching", request.urls.len()),
        task_type: "batch_fetch_pictures".to_string(),
    }))
}

/// Get picture metadata by URL
/// GET /api/picture?url=https://example.com/image.jpg
pub async fn get_picture(
    State(state): State<ApiState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<PictureResponse>, (StatusCode, Json<ErrorResponse>)> {
    let url = params.get("url")
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Missing 'url' query parameter".to_string(),
                })
            )
        })?;

    info!(url = %url, "API request: get picture");

    let picture = database::get_picture_by_url(state.db.db(), url)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to get picture from database");
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
                    error: format!("Picture not found: {}", url),
                })
            )
        })?;

    Ok(Json(PictureResponse { picture }))
}

/// Get pictures with filters
/// GET /api/picture/list?entity_type=anime&entity_id=123&tag=cover&status=Completed&limit=50
pub async fn list_pictures(
    State(state): State<ApiState>,
    Query(query): Query<GetPicturesQuery>,
) -> Result<Json<PicturesResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        entity_type = ?query.entity_type,
        entity_id = ?query.entity_id,
        tag = ?query.tag,
        status = ?query.status,
        "API request: list pictures"
    );

    let pictures = if let (Some(entity_type), Some(entity_id)) = (&query.entity_type, &query.entity_id) {
        database::get_pictures_by_entity(state.db.db(), entity_type, entity_id)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to get pictures");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Database error: {}", e),
                    })
                )
            })?
    } else if let Some(tag) = &query.tag {
        database::get_pictures_by_tag(state.db.db(), tag, query.limit)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to get pictures");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Database error: {}", e),
                    })
                )
            })?
    } else if let Some(status) = &query.status {
        database::get_pictures_by_status(state.db.db(), status, query.limit)
            .await
            .map_err(|e| {
                error!(error = %e, "Failed to get pictures");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Database error: {}", e),
                    })
                )
            })?
    } else {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Must provide entity_type+entity_id, tag, or status".to_string(),
            })
        ));
    };

    let count = pictures.len();
    Ok(Json(PicturesResponse { pictures, count }))
}

/// Get picture statistics
/// GET /api/picture/stats
pub async fn get_stats(
    State(state): State<ApiState>,
) -> Result<Json<StatsResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!("API request: get picture stats");

    let stats = database::get_picture_stats(state.db.db())
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to get picture stats");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                })
            )
        })?;

    Ok(Json(StatsResponse { stats }))
}

/// Delete picture metadata
/// DELETE /api/picture?url=https://example.com/image.jpg
pub async fn delete_picture(
    State(state): State<ApiState>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<TaskQueuedResponse>, (StatusCode, Json<ErrorResponse>)> {
    let url = params.get("url")
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: "Missing 'url' query parameter".to_string(),
                })
            )
        })?;

    info!(url = %url, "API request: delete picture");

    let deleted = database::delete_picture(state.db.db(), url)
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to delete picture");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                })
            )
        })?;

    if !deleted {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: format!("Picture not found: {}", url),
            })
        ));
    }

    Ok(Json(TaskQueuedResponse {
        message: format!("Picture metadata deleted: {}", url),
        task_type: "delete_picture".to_string(),
    }))
}