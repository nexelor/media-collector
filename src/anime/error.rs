#[derive(Debug, thiserror::Error)]
pub enum AnimeError {
    #[error("anime not found")]
    NotFound,
}