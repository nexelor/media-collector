use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::{info, error};

use crate::api::state::ApiState;

// ========================================================================
// Request/Response Types
// ========================================================================

#[derive(Debug, Deserialize)]
pub struct FetchPictureRequest {
    pub url: String,
    pub filename: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BatchFetchPicturesRequest {
    pub urls: Vec<String>,
}

#[derive(Serialize)]
pub struct TaskQueuedResponse {
    pub message: String,
    pub task_type: String,
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
/// Body: { "url": "https://example.com/image.jpg", "filename": "custom_name.jpg" }
pub async fn fetch_picture(
    State(state): State<ApiState>,
    Json(request): Json<FetchPictureRequest>,
) -> Result<Json<TaskQueuedResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        url = %request.url,
        filename = ?request.filename,
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

    Ok(Json(TaskQueuedResponse {
        message: format!("Picture queued for fetching: {}", request.url),
        task_type: "fetch_picture".to_string(),
    }))
}

/// Batch fetch multiple pictures
/// POST /api/picture/batch
/// Body: { "urls": ["https://example.com/1.jpg", "https://example.com/2.jpg"] }
pub async fn batch_fetch(
    State(state): State<ApiState>,
    Json(request): Json<BatchFetchPicturesRequest>,
) -> Result<Json<TaskQueuedResponse>, (StatusCode, Json<ErrorResponse>)> {
    info!(
        count = request.urls.len(),
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

    picture_module
        .queue_fetch_pictures(request.urls.clone())
        .await
        .map_err(|e| {
            error!(error = %e, "Failed to queue picture batch fetch task");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Failed to queue tasks: {}", e),
                })
            )
        })?;

    Ok(Json(TaskQueuedResponse {
        message: format!("{} pictures queued for fetching", request.urls.len()),
        task_type: "batch_fetch_pictures".to_string(),
    }))
}