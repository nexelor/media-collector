use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use crate::anime::my_anime_list::model::{
    Images, Title, Trailer, MediaType, NSFW, Source, Status, 
    Aired, Season, Broadcast, Genre, Themes, Studio, Demographic,
    Relation, Theme, External, Streaming, Character, Staff, Statistics
};

// ========================================================================
// AniList GraphQL Request/Response Models
// ========================================================================

#[derive(Debug, Clone, Serialize)]
pub struct GraphQLRequest {
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variables: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GraphQLResponse<T> {
    pub data: Option<T>,
    #[serde(default)]
    pub errors: Vec<GraphQLError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GraphQLError {
    pub message: String,
    #[serde(default)]
    pub locations: Vec<ErrorLocation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ErrorLocation {
    pub line: i32,
    pub column: i32,
}

// ========================================================================
// AniList Media Response
// ========================================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct MediaData {
    pub media: AniListMedia,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AniListMedia {
    pub id: i32,
    pub id_mal: Option<i32>,
    pub title: MediaTitle,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    pub format: Option<String>,
    pub status: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<FuzzyDate>,
    pub end_date: Option<FuzzyDate>,
    pub season: Option<String>,
    pub season_year: Option<i32>,
    pub episodes: Option<i32>,
    pub duration: Option<i32>,
    pub country_of_origin: Option<String>,
    pub is_licensed: Option<bool>,
    pub source: Option<String>,
    pub hashtag: Option<String>,
    pub trailer: Option<MediaTrailer>,
    pub updated_at: Option<i64>,
    pub cover_image: Option<MediaCoverImage>,
    pub banner_image: Option<String>,
    #[serde(default)]
    pub genres: Vec<String>,
    #[serde(default)]
    pub synonyms: Vec<String>,
    pub average_score: Option<i32>,
    pub mean_score: Option<i32>,
    pub popularity: Option<i32>,
    pub is_locked: Option<bool>,
    pub trending: Option<i32>,
    pub favourites: Option<i32>,
    #[serde(default)]
    pub tags: Vec<MediaTag>,
    #[serde(default)]
    pub relations: Option<MediaConnection>,
    #[serde(default)]
    pub characters: Option<CharacterConnection>,
    #[serde(default)]
    pub staff: Option<StaffConnection>,
    #[serde(default)]
    pub studios: Option<StudioConnection>,
    pub is_favourite: Option<bool>,
    pub is_favourite_blocked: Option<bool>,
    pub is_adult: Option<bool>,
    pub next_airing_episode: Option<AiringSchedule>,
    #[serde(default)]
    pub external_links: Vec<MediaExternalLink>,
    #[serde(default)]
    pub streaming_episodes: Vec<MediaStreamingEpisode>,
    pub rankings: Option<Vec<MediaRank>>,
    pub stats: Option<MediaStats>,
    pub site_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
    pub user_preferred: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FuzzyDate {
    pub year: Option<i32>,
    pub month: Option<i32>,
    pub day: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaTrailer {
    pub id: Option<String>,
    pub site: Option<String>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaCoverImage {
    pub extra_large: Option<String>,
    pub large: Option<String>,
    pub medium: Option<String>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaTag {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub rank: Option<i32>,
    pub is_general_spoiler: Option<bool>,
    pub is_media_spoiler: Option<bool>,
    pub is_adult: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MediaConnection {
    pub edges: Option<Vec<MediaEdge>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaEdge {
    pub id: Option<i32>,
    pub relation_type: Option<String>,
    pub node: Option<MediaNode>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaNode {
    pub id: i32,
    pub id_mal: Option<i32>,
    pub title: Option<MediaTitle>,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    pub format: Option<String>,
    pub cover_image: Option<MediaCoverImage>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterConnection {
    pub edges: Option<Vec<CharacterEdge>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterEdge {
    pub id: Option<i32>,
    pub role: Option<String>,
    pub node: Option<CharacterNode>,
    #[serde(default, rename = "voiceActors")]
    pub voice_actors: Vec<StaffNode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterNode {
    pub id: i32,
    pub name: Option<CharacterName>,
    pub image: Option<CharacterImage>,
    #[serde(rename = "siteUrl")]
    pub site_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterName {
    pub first: Option<String>,
    pub middle: Option<String>,
    pub last: Option<String>,
    pub full: Option<String>,
    pub native: Option<String>,
    #[serde(default)]
    pub alternative: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CharacterImage {
    pub large: Option<String>,
    pub medium: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StaffConnection {
    pub edges: Option<Vec<StaffEdge>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StaffEdge {
    pub id: Option<i32>,
    pub role: Option<String>,
    pub node: Option<StaffNode>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StaffNode {
    pub id: i32,
    pub name: Option<StaffName>,
    pub language: Option<String>,
    pub image: Option<StaffImage>,
    #[serde(rename = "siteUrl")]
    pub site_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StaffName {
    pub first: Option<String>,
    pub middle: Option<String>,
    pub last: Option<String>,
    pub full: Option<String>,
    pub native: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StaffImage {
    pub large: Option<String>,
    pub medium: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StudioConnection {
    pub edges: Option<Vec<StudioEdge>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioEdge {
    pub is_main: bool,
    pub node: Option<StudioNode>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StudioNode {
    pub id: i32,
    pub name: String,
    pub is_animation_studio: bool,
    pub site_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiringSchedule {
    pub airing_at: i64,
    pub time_until_airing: i64,
    pub episode: i32,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaExternalLink {
    pub id: i32,
    pub url: String,
    pub site: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaStreamingEpisode {
    pub title: Option<String>,
    pub thumbnail: Option<String>,
    pub url: Option<String>,
    pub site: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaRank {
    pub id: i32,
    pub rank: i32,
    #[serde(rename = "type")]
    pub rank_type: String,
    pub format: String,
    pub year: Option<i32>,
    pub season: Option<String>,
    pub all_time: Option<bool>,
    pub context: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaStats {
    pub score_distribution: Option<Vec<ScoreDistribution>>,
    pub status_distribution: Option<Vec<StatusDistribution>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScoreDistribution {
    pub score: Option<i32>,
    pub amount: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StatusDistribution {
    pub status: Option<String>,
    pub amount: Option<i32>,
}

// ========================================================================
// Search Response
// ========================================================================

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PageData {
    pub page: Page,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub page_info: Option<PageInfo>,
    #[serde(default)]
    pub media: Vec<AniListMedia>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageInfo {
    pub total: Option<i32>,
    pub per_page: Option<i32>,
    pub current_page: Option<i32>,
    pub last_page: Option<i32>,
    pub has_next_page: Option<bool>,
}

/// AniList-specific anime data structure stored in anime_anilist collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListAnimeData {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    
    // AniList-specific IDs
    pub anilist_id: i32,
    pub mal_id: Option<i32>,
    
    // Basic Info
    pub url: String,
    pub images: Images,
    pub trailer: Trailer,
    pub titles: Vec<Title>,
    pub media_type: Option<MediaType>,
    pub nsfw: Option<NSFW>,
    pub source: Option<Source>,
    pub num_episodes: i32,
    pub average_episode_duration: i32,
    pub status: Option<Status>,
    pub airing: bool,
    pub aired: Aired,
    pub duration: String,
    pub score: Option<f32>,
    pub popularity: i32,
    pub favorites: i32,
    pub synopsis: String,
    pub background: Option<String>,
    pub season: Option<Season>,
    pub year: Option<i32>,
    pub broadcast: Broadcast,
    
    // Related entities
    pub studios: Vec<Studio>,
    pub genres: Vec<Genre>,
    pub themes: Vec<Themes>,
    pub demographics: Vec<Demographic>,
    pub relations: Vec<Relation>,
    pub theme: Theme,
    pub external: Vec<External>,
    pub streaming: Vec<Streaming>,
    
    // Extended data
    #[serde(default)]
    pub characters: Vec<Character>,
    #[serde(default)]
    pub staffs: Vec<Staff>,
    pub statistics: Option<Statistics>,
    
    // AniList-specific fields
    pub country_of_origin: Option<String>,
    pub is_licensed: bool,
    pub hashtag: Option<String>,
    pub trending: i32,
    #[serde(default)]
    pub tags: Vec<AniListTag>,
    pub next_airing_episode: Option<AniListAiringSchedule>,
    pub banner_image: Option<String>,
    pub cover_color: Option<String>,
    
    // Timestamps
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListTag {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub rank: i32,
    pub is_spoiler: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AniListAiringSchedule {
    pub airing_at: i64,
    pub time_until_airing: i64,
    pub episode: i32,
}