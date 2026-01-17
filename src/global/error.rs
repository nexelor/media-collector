#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Database(#[from] DatabaseError),

    #[error("module error: {0}")]
    Module(String),

    #[error(transparent)]
    Anime(#[from] crate::anime::error::AnimeError),

    #[error(transparent)]
    Http(#[from] HttpError),
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("mongodb connection failed")]
    ConnexionFailed(#[from] mongodb::error::Error),

    #[error("query failed: {0}")]
    Query(String),
}

#[derive(Debug, thiserror::Error)]
pub enum HttpError {
    #[error("resource not found: {0}")]
    NotFound(String),

    #[error("rate limit exceeded, retry after {retry_after:?}")]
    RateLimited {
        retry_after: Option<std::time::Duration>,
        message: String,
    },

    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("failed to deserialize response: {0}")]
    DeserializationFailed(String),

    #[error("unexpected status code {status}: {message}")]
    UnexpectedStatus {
        status: u16,
        message: String,
    },

    #[error("max retries exceeded")]
    MaxRetriesExceeded,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing required API key for module: {0}")]
    MissingApiKey(String),

    #[error("invalid configuration: {0}")]
    Invalid(String),

    #[error("failed to load configuration: {0}")]
    LoadFailed(String),
}