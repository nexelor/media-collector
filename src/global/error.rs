#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Database(#[from] DatabaseError),

    // #[error(transparent)]
    // MyAnimeList(#[from] MyAnimeListError),
}

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("mongodb connection failed")]
    ConnexionFailed(#[from] mongodb::error::Error),

    #[error("query failed: {0}")]
    Query(String),
}