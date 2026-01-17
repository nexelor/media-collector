use std::sync::Arc;

use crate::global::{
    config::AppConfig,
    database::DatabaseInstance,
    http::HttpClientManager,
};
use crate::anime::module::AnimeModule;
use crate::picture::PictureFetcherModule;

/// Application state shared across API handlers
#[derive(Clone)]
pub struct ApiState {
    pub config: Arc<AppConfig>,
    pub db: Arc<DatabaseInstance>,
    pub http_manager: Arc<HttpClientManager>,
    
    // Module references
    pub anime_module: Option<Arc<AnimeModule>>,
    pub picture_module: Option<Arc<PictureFetcherModule>>,
}

impl ApiState {
    pub fn new(
        config: Arc<AppConfig>,
        db: Arc<DatabaseInstance>,
        http_manager: Arc<HttpClientManager>,
    ) -> Self {
        Self {
            config,
            db,
            http_manager,
            anime_module: None,
            picture_module: None,
        }
    }

    pub fn with_anime_module(mut self, module: Arc<AnimeModule>) -> Self {
        self.anime_module = Some(module);
        self
    }

    pub fn with_picture_module(mut self, module: Arc<PictureFetcherModule>) -> Self {
        self.picture_module = Some(module);
        self
    }
}