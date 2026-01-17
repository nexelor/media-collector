use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeData {
    pub id: u32,
    pub title: String,
    pub score: Option<f32>,
}