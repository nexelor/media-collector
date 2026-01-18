use std::sync::Arc;
use serde::Deserialize;
use tracing::{debug, info, warn};

use crate::anime::my_anime_list::database::get_anime_by_id;
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
    jikan_client: crate::global::http::ClientWithLimiter,
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
        jikan_client: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("fetch_characters_{}", anime_id);
        Self {
            id,
            anime_id,
            jikan_client,
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
        // tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

        let response = self.jikan_client
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

        // Get existing anime and update it
        if let Some(mut anime) = get_anime_by_id(db.db(), self.anime_id as i32).await? {
            anime.characters = characters;
            crate::anime::my_anime_list::database::upsert_anime(db.db(), &anime).await?;
        } else {
            warn!(
                task = %self.name(),
                anime_id = self.anime_id,
                "Anime not found in database, cannot update characters"
            );
        }

        Ok(())
    }
}

// ========================================================================
// Fetch Staff Task (Jikan)
// ========================================================================

pub struct FetchStaffTask {
    id: String,
    anime_id: u32,
    jikan_client: crate::global::http::ClientWithLimiter,
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
        jikan_client: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("fetch_staff_{}", anime_id);
        Self {
            id,
            anime_id,
            jikan_client,
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

        let response = self.jikan_client
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

        // Get existing anime and update it
        if let Some(mut anime) = get_anime_by_id(db.db(), self.anime_id as i32).await? {
            anime.staffs = staff;
            crate::anime::my_anime_list::database::upsert_anime(db.db(), &anime).await?;
        } else {
            warn!(
                task = %self.name(),
                anime_id = self.anime_id,
                "Anime not found in database, cannot update staff"
            );
        }

        Ok(())
    }
}

// ========================================================================
// Fetch Episodes Task (Jikan)
// ========================================================================

pub struct FetchEpisodesTask {
    id: String,
    anime_id: u32,
    jikan_client: crate::global::http::ClientWithLimiter,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct JikanEpisodesResponse {
    data: Vec<JikanEpisode>,
    pagination: JikanPagination,
}

#[derive(Debug, Deserialize)]
struct JikanPagination {
    last_visible_page: i32,
    has_next_page: bool,
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
        jikan_client: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("fetch_episodes_{}", anime_id);
        Self {
            id,
            anime_id,
            jikan_client,
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
            "Fetching episodes from Jikan API (paginated)"
        );

        let mut all_episodes = Vec::new();
        let mut page = 1;
        let mut has_next_page = true;

        // Fetch all pages
        while has_next_page {
            let url = format!(
                "https://api.jikan.moe/v4/anime/{}/episodes?page={}",
                self.anime_id, page
            );
            
            debug!(
                task = %self.name(),
                anime_id = self.anime_id,
                page = page,
                "Fetching episode page"
            );
            
            // Respect rate limit between pages
            tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

            let response = match self.jikan_client
                .fetch_json::<JikanEpisodesResponse>(&url, None)
                .await
            {
                Ok(resp) => resp,
                Err(e) => {
                    warn!(
                        task = %self.name(),
                        anime_id = self.anime_id,
                        page = page,
                        error = %e,
                        "Failed to fetch episode page, stopping pagination"
                    );
                    break;
                }
            };

            let episodes_in_page = response.data.len();
            all_episodes.extend(response.data);
            has_next_page = response.pagination.has_next_page;

            info!(
                task = %self.name(),
                anime_id = self.anime_id,
                page = page,
                episodes_in_page = episodes_in_page,
                total_so_far = all_episodes.len(),
                has_next_page = has_next_page,
                "Fetched episode page"
            );

            if has_next_page {
                page += 1;
            }
        }

        // Convert to our model
        let episodes: Vec<Episode> = all_episodes.into_iter().map(|e| {
            Episode {
                mal_id: e.mal_id,
                url: e.url,
                title: e.title,
                title_japanese: e.title_japanese,
                title_romanji: e.title_romanji,
                duration: None, // Jikan episodes endpoint doesn't include duration
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
            total_episodes = episodes.len(),
            total_pages = page,
            "Fetched all episodes, updating anime"
        );

        // Get existing anime and update it
        if let Some(mut anime) = get_anime_by_id(db.db(), self.anime_id as i32).await? {
            anime.episodes = episodes;
            crate::anime::my_anime_list::database::upsert_anime(db.db(), &anime).await?;
        } else {
            warn!(
                task = %self.name(),
                anime_id = self.anime_id,
                "Anime not found in database, cannot update episodes"
            );
        }

        Ok(())
    }
}

// ========================================================================
// Fetch Videos Task (Jikan) - NEW
// ========================================================================

pub struct FetchVideosTask {
    id: String,
    anime_id: u32,
    jikan_client: crate::global::http::ClientWithLimiter,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct JikanVideosResponse {
    data: JikanVideosData,
}

#[derive(Debug, Deserialize)]
struct JikanVideosData {
    #[serde(default)]
    promo: Vec<JikanVideoPromo>,
    #[serde(default)]
    episodes: Vec<JikanVideoEpisode>,
    #[serde(default)]
    music_videos: Vec<JikanVideoMusic>,
}

#[derive(Debug, Deserialize)]
struct JikanVideoPromo {
    pub title: Option<String>,
    pub trailer: JikanVideoTrailer,
}

#[derive(Debug, Deserialize)]
struct JikanVideoTrailer {
    pub youtube_id: Option<String>,
    pub url: Option<String>,
    pub embed_url: Option<String>,
    pub images: Option<JikanImages>,
}

#[derive(Debug, Deserialize)]
struct JikanVideoEpisode {
    pub mal_id: i32,
    pub url: Option<String>,
    pub title: Option<String>,
    pub episode: Option<String>,
    pub images: Option<JikanImages>,
}

#[derive(Debug, Deserialize)]
struct JikanVideoMusic {
    pub title: Option<String>,
    pub video: JikanVideoTrailer,
    pub meta: JikanVideoMusicMeta,
}

#[derive(Debug, Deserialize)]
struct JikanVideoMusicMeta {
    pub title: Option<String>,
    pub author: Option<String>,
}

impl FetchVideosTask {
    pub fn new(
        anime_id: u32,
        jikan_client: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("fetch_videos_{}", anime_id);
        Self {
            id,
            anime_id,
            jikan_client,
            created_at: chrono::Utc::now(),
        }
    }
}

#[async_trait::async_trait]
impl Task for FetchVideosTask {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> &str {
        "fetch_videos"
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
            "Fetching videos from Jikan API"
        );

        let url = format!("https://api.jikan.moe/v4/anime/{}/videos", self.anime_id);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

        let response = self.jikan_client
            .fetch_json::<JikanVideosResponse>(&url, None)
            .await?;

        let videos = Videos {
            promo: response.data.promo.into_iter().map(|p| VideoPromoInfo {
                title: p.title.unwrap_or_else(|| "Unknown".to_string()),
                trailer: VideoTrailer {
                    youtube_id: p.trailer.youtube_id,
                    url: p.trailer.url,
                    embed_url: p.trailer.embed_url,
                    images: p.trailer.images.map(|img| convert_jikan_images(img)),
                },
            }).collect(),
            episodes: response.data.episodes.into_iter().map(|e| VideoEpisodeInfo {
                mal_id: e.mal_id,
                url: e.url.unwrap_or_default(),
                title: e.title.unwrap_or_else(|| "Unknown".to_string()),
                episode: e.episode.unwrap_or_else(|| "Unknown".to_string()),
                images: e.images.map(|img| convert_jikan_images(img)).unwrap_or_else(default_images),
            }).collect(),
            music_videos: response.data.music_videos.into_iter().map(|m| VideoMusicInfo {
                title: m.title.unwrap_or_else(|| "Unknown".to_string()),
                video: VideoTrailer {
                    youtube_id: m.video.youtube_id,
                    url: m.video.url,
                    embed_url: m.video.embed_url,
                    images: m.video.images.map(|img| convert_jikan_images(img)),
                },
                meta: VideoMusicMeta {
                    title: m.meta.title,
                    author: m.meta.author,
                },
            }).collect(),
        };

        info!(
            task = %self.name(),
            anime_id = self.anime_id,
            promo_count = videos.promo.len(),
            episode_count = videos.episodes.len(),
            music_count = videos.music_videos.len(),
            "Fetched videos, updating anime"
        );

        // Get existing anime and update it
        if let Some(mut anime) = get_anime_by_id(db.db(), self.anime_id as i32).await? {
            anime.videos = Some(videos);
            crate::anime::my_anime_list::database::upsert_anime(db.db(), &anime).await?;
        } else {
            warn!(
                task = %self.name(),
                anime_id = self.anime_id,
                "Anime not found in database, cannot update videos"
            );
        }

        Ok(())
    }
}

// ========================================================================
// Fetch Statistics Task (Jikan) - NEW
// ========================================================================

pub struct FetchStatisticsTask {
    id: String,
    anime_id: u32,
    jikan_client: crate::global::http::ClientWithLimiter,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct JikanStatisticsResponse {
    data: JikanStatistics,
}

#[derive(Debug, Deserialize)]
struct JikanStatistics {
    watching: i32,
    completed: i32,
    on_hold: i32,
    dropped: i32,
    plan_to_watch: i32,
    total: i32,
    #[serde(default)]
    scores: Vec<JikanScore>,
}

#[derive(Debug, Deserialize)]
struct JikanScore {
    score: i32,
    votes: i32,
    percentage: f32,
}

impl FetchStatisticsTask {
    pub fn new(
        anime_id: u32,
        jikan_client: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("fetch_statistics_{}", anime_id);
        Self {
            id,
            anime_id,
            jikan_client,
            created_at: chrono::Utc::now(),
        }
    }
}

#[async_trait::async_trait]
impl Task for FetchStatisticsTask {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> &str {
        "fetch_statistics"
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
            "Fetching statistics from Jikan API"
        );

        let url = format!("https://api.jikan.moe/v4/anime/{}/statistics", self.anime_id);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

        let response = self.jikan_client
            .fetch_json::<JikanStatisticsResponse>(&url, None)
            .await?;

        let statistics = Statistics {
            watching: response.data.watching,
            completed: response.data.completed,
            on_hold: response.data.on_hold,
            dropped: response.data.dropped,
            plan_to_watch: response.data.plan_to_watch,
            total: response.data.total,
            scores: response.data.scores.into_iter().map(|s| StatisticsScore {
                score: s.score,
                votes: s.votes,
                percentage: s.percentage,
            }).collect(),
        };

        info!(
            task = %self.name(),
            anime_id = self.anime_id,
            "Fetched statistics, updating anime"
        );

        // Get existing anime and update it
        if let Some(mut anime) = get_anime_by_id(db.db(), self.anime_id as i32).await? {
            anime.statistics = Some(statistics);
            crate::anime::my_anime_list::database::upsert_anime(db.db(), &anime).await?;
        } else {
            warn!(
                task = %self.name(),
                anime_id = self.anime_id,
                "Anime not found in database, cannot update statistics"
            );
        }

        Ok(())
    }
}

// ========================================================================
// Fetch More Info Task (Jikan) - NEW
// ========================================================================

pub struct FetchMoreInfoTask {
    id: String,
    anime_id: u32,
    jikan_client: crate::global::http::ClientWithLimiter,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct JikanMoreInfoResponse {
    data: Option<JikanMoreInfo>,
}

#[derive(Debug, Deserialize)]
struct JikanMoreInfo {
    moreinfo: Option<String>,
}

impl FetchMoreInfoTask {
    pub fn new(
        anime_id: u32,
        jikan_client: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("fetch_moreinfo_{}", anime_id);
        Self {
            id,
            anime_id,
            jikan_client,
            created_at: chrono::Utc::now(),
        }
    }
}

#[async_trait::async_trait]
impl Task for FetchMoreInfoTask {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> &str {
        "fetch_more_info"
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
            "Fetching more info from Jikan API"
        );

        let url = format!("https://api.jikan.moe/v4/anime/{}/moreinfo", self.anime_id);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

        let response = self.jikan_client
            .fetch_json::<JikanMoreInfoResponse>(&url, None)
            .await?;

        if let Some(more_info_data) = response.data {
            if let Some(more_info) = more_info_data.moreinfo {
                info!(
                    task = %self.name(),
                    anime_id = self.anime_id,
                    "Fetched more info, updating anime"
                );

                // Get existing anime and update it
                if let Some(mut anime) = get_anime_by_id(db.db(), self.anime_id as i32).await? {
                    anime.more_info = Some(more_info);
                    crate::anime::my_anime_list::database::upsert_anime(db.db(), &anime).await?;
                } else {
                    warn!(
                        task = %self.name(),
                        anime_id = self.anime_id,
                        "Anime not found in database, cannot update more info"
                    );
                }
            }
        }

        Ok(())
    }
}

// ========================================================================
// Fetch Recommendations Task (Jikan) - NEW
// ========================================================================

pub struct FetchRecommendationsTask {
    id: String,
    anime_id: u32,
    jikan_client: crate::global::http::ClientWithLimiter,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct JikanRecommendationsResponse {
    data: Vec<JikanRecommendation>,
}

#[derive(Debug, Deserialize)]
struct JikanRecommendation {
    entry: JikanRecommendationEntry,
    url: String,
    votes: i32,
}

#[derive(Debug, Deserialize)]
struct JikanRecommendationEntry {
    mal_id: i32,
    url: String,
    images: JikanImages,
    title: String,
}

impl FetchRecommendationsTask {
    pub fn new(
        anime_id: u32,
        jikan_client: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("fetch_recommendations_{}", anime_id);
        Self {
            id,
            anime_id,
            jikan_client,
            created_at: chrono::Utc::now(),
        }
    }
}

#[async_trait::async_trait]
impl Task for FetchRecommendationsTask {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> &str {
        "fetch_recommendations"
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
            "Fetching recommendations from Jikan API"
        );

        let url = format!("https://api.jikan.moe/v4/anime/{}/recommendations", self.anime_id);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

        let response = self.jikan_client
            .fetch_json::<JikanRecommendationsResponse>(&url, None)
            .await?;

        let recommendations: Vec<Recommendation> = response.data.into_iter().map(|r| {
            Recommendation {
                entry: RecommendationInfo {
                    mal_id: r.entry.mal_id,
                    url: r.entry.url,
                    images: convert_jikan_images(r.entry.images),
                    title: r.entry.title,
                },
                url: r.url,
                votes: r.votes,
            }
        }).collect();

        info!(
            task = %self.name(),
            anime_id = self.anime_id,
            count = recommendations.len(),
            "Fetched recommendations, updating anime"
        );

        // Get existing anime and update it
        if let Some(mut anime) = get_anime_by_id(db.db(), self.anime_id as i32).await? {
            anime.recommendations = recommendations;
            crate::anime::my_anime_list::database::upsert_anime(db.db(), &anime).await?;
        } else {
            warn!(
                task = %self.name(),
                anime_id = self.anime_id,
                "Anime not found in database, cannot update recommendations"
            );
        }

        Ok(())
    }
}

// ========================================================================
// Fetch Pictures Task (Jikan) - NEW
// ========================================================================

pub struct FetchPicturesTask {
    id: String,
    anime_id: u32,
    jikan_client: crate::global::http::ClientWithLimiter,
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct JikanPicturesResponse {
    data: Vec<JikanPicture>,
}

#[derive(Debug, Deserialize)]
struct JikanPicture {
    jpg: JikanImage,
    webp: JikanImage,
}

impl FetchPicturesTask {
    pub fn new(
        anime_id: u32,
        jikan_client: crate::global::http::ClientWithLimiter,
    ) -> Self {
        let id = format!("fetch_pictures_{}", anime_id);
        Self {
            id,
            anime_id,
            jikan_client,
            created_at: chrono::Utc::now(),
        }
    }
}

#[async_trait::async_trait]
impl Task for FetchPicturesTask {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> &str {
        "fetch_pictures"
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
            "Fetching pictures from Jikan API"
        );

        let url = format!("https://api.jikan.moe/v4/anime/{}/pictures", self.anime_id);
        
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

        let response = self.jikan_client
            .fetch_json::<JikanPicturesResponse>(&url, None)
            .await?;

        let pictures: Vec<Images> = response.data.into_iter().map(|p| {
            convert_jikan_images(JikanImages {
                jpg: p.jpg,
                webp: p.webp,
            })
        }).collect();

        info!(
            task = %self.name(),
            anime_id = self.anime_id,
            count = pictures.len(),
            "Fetched pictures, updating anime"
        );

        // Get existing anime and update it
        if let Some(mut anime) = get_anime_by_id(db.db(), self.anime_id as i32).await? {
            anime.pictures = pictures;
            // Also update main image if we have pictures
            if let Some(first_picture) = anime.pictures.first() {
                anime.images = first_picture.clone();
            }
            crate::anime::my_anime_list::database::upsert_anime(db.db(), &anime).await?;
        } else {
            warn!(
                task = %self.name(),
                anime_id = self.anime_id,
                "Anime not found in database, cannot update pictures"
            );
        }

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

fn default_images() -> Images {
    Images {
        jpg: Image {
            image_url: String::new(),
            small_image_url: String::new(),
            large_image_url: String::new(),
        },
        webp: Image {
            image_url: String::new(),
            small_image_url: String::new(),
            large_image_url: String::new(),
        },
    }
}

fn parse_jikan_date(date_str: &str) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(date_str)
        .ok()
        .map(|dt| dt.with_timezone(&chrono::Utc))
}