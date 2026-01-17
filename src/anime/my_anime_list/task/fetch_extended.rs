use std::sync::Arc;
use serde::Deserialize;
use tracing::{info, debug};

use crate::global::queue::{TaskData, TaskPriority, TaskStatus};
use crate::global::{
    database::DatabaseInstance,
    error::AppError,
    queue::Task,
};
use crate::anime::my_anime_list::{
    model::*,
    database::update_anime_extended_data,
};

// ========================================================================
// Fetch Characters Task (Jikan)
// ========================================================================

pub struct FetchCharactersTask {
    id: String,
    anime_id: u32,
    client_with_limiter: crate::global::http::ClientWithLimiter,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct JikanCharactersResponse {
    data: Vec<JikanCharacter>,
}

#[derive(Debug, Deserialize)]
struct JikanCharacter {
    character: JikanCharacterInfo,
    role: String,
    #[serde(default)]
    voice_actors: Vec<JikanVoiceActor>,
}

#[derive(Debug, Deserialize)]
struct JikanCharacterInfo {
    mal_id: i32,
    url: String,
    images: JikanImages,
    name: String,
}

#[derive(Debug, Deserialize)]
struct JikanVoiceActor {
    person: JikanPersonInfo,
    language: String,
}

#[derive(Debug, Deserialize)]
struct JikanPersonInfo {
    mal_id: i32,
    url: String,
    images: JikanImages,
    name: String,
}

impl FetchCharactersTask {
    pub fn new(
        anime_id: u32,
        client_with_limiter: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("fetch_characters_{}", anime_id);
        Self {
            id,
            anime_id,
            client_with_limiter,
            created_at: chrono::Utc::now(),
        }
    }
}

#[async_trait::async_trait]
impl Task for FetchCharactersTask {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> &str {
        "fetch_characters"
    }

    fn priority(&self) -> TaskPriority {
        TaskPriority::Low
    }

    fn to_data(&self) -> TaskData {
        TaskData {
            id: self.id(),
            name: self.name().to_string(),
            priority: self.priority(),
            status: TaskStatus::Pending,
            created_at: self.created_at,
            payload: serde_json::json!({ "anime_id": self.anime_id }),
        }
    }

    async fn execute(&self, db: Arc<DatabaseInstance>, _client: reqwest::Client) -> Result<(), AppError> {
        info!(
            task = %self.name(),
            anime_id = self.anime_id,
            "Fetching characters from Jikan API"
        );

        let url = format!("https://api.jikan.moe/v4/anime/{}/characters", self.anime_id);
        
        // Respect Jikan rate limit
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

        let response = self.client_with_limiter
            .fetch_json::<JikanCharactersResponse>(&url, None)
            .await?;

        // Convert to our model
        let characters: Vec<Character> = response.data.into_iter().map(|c| {
            Character {
                character: CharacterInfo {
                    mal_id: c.character.mal_id,
                    url: c.character.url,
                    images: convert_jikan_images(c.character.images),
                    name: c.character.name,
                },
                role: c.role,
                voice_actors: c.voice_actors.into_iter().map(|va| VoiceActor {
                    person: VoiceActorInfo {
                        mal_id: va.person.mal_id,
                        url: va.person.url,
                        images: convert_jikan_images(va.person.images),
                        name: va.person.name,
                    },
                    language: va.language,
                }).collect(),
            }
        }).collect();

        info!(
            task = %self.name(),
            anime_id = self.anime_id,
            count = characters.len(),
            "Fetched characters, updating anime_mal collection"
        );

        update_anime_extended_data(db.db(), self.anime_id, "characters", &characters).await?;

        Ok(())
    }
}

// ========================================================================
// Fetch Staff Task (Jikan)
// ========================================================================

pub struct FetchStaffTask {
    id: String,
    anime_id: u32,
    client_with_limiter: crate::global::http::ClientWithLimiter,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct JikanStaffResponse {
    data: Vec<JikanStaff>,
}

#[derive(Debug, Deserialize)]
struct JikanStaff {
    person: JikanPersonInfo,
    positions: Vec<String>,
}

impl FetchStaffTask {
    pub fn new(
        anime_id: u32,
        client_with_limiter: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("fetch_staff_{}", anime_id);
        Self {
            id,
            anime_id,
            client_with_limiter,
            created_at: chrono::Utc::now(),
        }
    }
}

#[async_trait::async_trait]
impl Task for FetchStaffTask {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> &str {
        "fetch_staff"
    }

    fn priority(&self) -> TaskPriority {
        TaskPriority::Low
    }

    fn to_data(&self) -> TaskData {
        TaskData {
            id: self.id(),
            name: self.name().to_string(),
            priority: self.priority(),
            status: TaskStatus::Pending,
            created_at: self.created_at,
            payload: serde_json::json!({ "anime_id": self.anime_id }),
        }
    }

    async fn execute(&self, db: Arc<DatabaseInstance>, _client: reqwest::Client) -> Result<(), AppError> {
        info!(
            task = %self.name(),
            anime_id = self.anime_id,
            "Fetching staff from Jikan API"
        );

        let url = format!("https://api.jikan.moe/v4/anime/{}/staff", self.anime_id);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

        let response = self.client_with_limiter
            .fetch_json::<JikanStaffResponse>(&url, None)
            .await?;

        let staff: Vec<Staff> = response.data.into_iter().map(|s| {
            Staff {
                person: StaffInfo {
                    mal_id: s.person.mal_id,
                    url: s.person.url,
                    images: convert_jikan_images(s.person.images),
                    name: s.person.name,
                },
                positions: s.positions,
            }
        }).collect();

        info!(
            task = %self.name(),
            anime_id = self.anime_id,
            count = staff.len(),
            "Fetched staff, updating anime_mal collection"
        );

        update_anime_extended_data(db.db(), self.anime_id, "staffs", &staff).await?;

        Ok(())
    }
}

// ========================================================================
// Fetch Episodes Task (Jikan)
// ========================================================================

pub struct FetchEpisodesTask {
    id: String,
    anime_id: u32,
    client_with_limiter: crate::global::http::ClientWithLimiter,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct JikanEpisodesResponse {
    data: Vec<JikanEpisode>,
}

#[derive(Debug, Deserialize)]
struct JikanEpisode {
    mal_id: i32,
    url: Option<String>,
    title: String,
    title_japanese: Option<String>,
    title_romanji: Option<String>,
    duration: Option<i32>,
    aired: Option<String>,
    score: Option<f32>,
    filler: bool,
    recap: bool,
    forum_url: Option<String>,
}

impl FetchEpisodesTask {
    pub fn new(
        anime_id: u32,
        client_with_limiter: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("fetch_episodes_{}", anime_id);
        Self {
            id,
            anime_id,
            client_with_limiter,
            created_at: chrono::Utc::now(),
        }
    }
}

#[async_trait::async_trait]
impl Task for FetchEpisodesTask {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> &str {
        "fetch_episodes"
    }

    fn priority(&self) -> TaskPriority {
        TaskPriority::Low
    }

    fn to_data(&self) -> TaskData {
        TaskData {
            id: self.id(),
            name: self.name().to_string(),
            priority: self.priority(),
            status: TaskStatus::Pending,
            created_at: self.created_at,
            payload: serde_json::json!({ "anime_id": self.anime_id }),
        }
    }

    async fn execute(&self, db: Arc<DatabaseInstance>, _client: reqwest::Client) -> Result<(), AppError> {
        info!(
            task = %self.name(),
            anime_id = self.anime_id,
            "Fetching episodes from Jikan API"
        );

        let url = format!("https://api.jikan.moe/v4/anime/{}/episodes", self.anime_id);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

        let response = self.client_with_limiter
            .fetch_json::<JikanEpisodesResponse>(&url, None)
            .await?;

        let episodes: Vec<Episode> = response.data.into_iter().map(|e| {
            Episode {
                mal_id: e.mal_id,
                url: e.url,
                title: e.title,
                title_japanese: e.title_japanese,
                title_romanji: e.title_romanji,
                duration: e.duration,
                aired: e.aired.as_ref().and_then(|s| parse_jikan_date(s)),
                score: e.score,
                filler: e.filler,
                recap: e.recap,
                forum_url: e.forum_url,
            }
        }).collect();

        info!(
            task = %self.name(),
            anime_id = self.anime_id,
            count = episodes.len(),
            "Fetched episodes, updating anime_mal collection"
        );

        update_anime_extended_data(db.db(), self.anime_id, "episodes", &episodes).await?;

        Ok(())
    }
}

// ========================================================================
// Helper Functions
// ========================================================================

fn convert_jikan_images(jikan_images: JikanImages) -> Images {
    Images {
        jpg: Image {
            image_url: jikan_images.jpg.image_url.unwrap_or_default(),
            small_image_url: jikan_images.jpg.small_image_url.unwrap_or_default(),
            large_image_url: jikan_images.jpg.large_image_url.unwrap_or_default(),
        },
        webp: Image {
            image_url: jikan_images.webp.image_url.unwrap_or_default(),
            small_image_url: jikan_images.webp.small_image_url.unwrap_or_default(),
            large_image_url: jikan_images.webp.large_image_url.unwrap_or_default(),
        },
    }
}

fn parse_jikan_date(date_str: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(date_str)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
}