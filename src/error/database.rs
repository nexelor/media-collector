#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("database connection failed: {0}")]
    ConnexionFailed(String),

    #[error("query failed: {0}")]
    Query(String),
}