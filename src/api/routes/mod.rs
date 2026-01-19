pub mod anime;
pub mod picture;
pub mod health;

use axum::{
    Router, routing::{delete, get, post}
};

use crate::api::state::ApiState;

/// Create the main API router
pub fn create_router(state: ApiState) -> Router {
    Router::new()
        // Health check
        .route("/health", get(health::health_check))
        .route("/stats", get(health::get_stats))
        
        // Anime routes
        .route("/api/anime/fetch", post(anime::fetch_anime))
        .route("/api/anime/search", post(anime::search_anime))
        .route("/api/anime/update", post(anime::update_anime))
        .route("/api/anime/batch", post(anime::batch_fetch))
        .route("/api/anime/extended", post(anime::fetch_extended_data))
        .route("/api/anime/{id}", get(anime::get_anime))
        
        .route("/api/anime/anilist/fetch", post(anime::fetch_from_anilist))

        // Picture routes
        .route("/api/picture/fetch", post(picture::fetch_picture))
        .route("/api/picture/batch", post(picture::batch_fetch))
        .route("/api/picture", get(picture::get_picture))
        .route("/api/picture", delete(picture::delete_picture))
        .route("/api/picture/list", get(picture::list_pictures))
        .route("/api/picture/stats", get(picture::get_stats))
        
        .with_state(state)
}