use serde::{Deserialize, Serialize};
use chrono::{DateTime, NaiveTime, Utc};
use mongodb::bson;

// ========================================================================
// Main Anime Data Model (combines MAL and Jikan data)
// ========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimeData {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<bson::oid::ObjectId>,
    
    pub mal_id: i32,
    pub url: String,
    pub images: Images,
    pub trailer: Trailer,
    pub approved: bool,
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
    pub rating: Option<Rating>,
    pub score: Option<f32>,
    pub scored_by: i32,
    pub rank: Option<i32>,
    pub members: i32,
    pub favorites: i32,
    pub popularity: Option<i32>,
    pub synopsis: String,
    pub background: Option<String>,
    pub season: Option<Season>,
    pub year: Option<i32>,
    pub broadcast: Broadcast,
    pub producers: Vec<Producer>,
    pub licensors: Vec<Licensors>,
    pub studios: Vec<Studio>,
    pub genres: Vec<Genre>,
    pub explicit_genres: Vec<Genre>,
    pub themes: Vec<Themes>,
    pub demographics: Vec<Demographic>,
    pub relations: Vec<Relation>,
    pub theme: Theme,
    pub external: Vec<External>,
    pub streaming: Vec<Streaming>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    
    // Extended data (fetched separately)
    #[serde(default)]
    pub characters: Vec<Character>,
    #[serde(default)]
    pub staffs: Vec<Staff>,
    #[serde(default)]
    pub episodes: Vec<Episode>,
    #[serde(default)]
    pub videos: Option<Videos>,
    #[serde(default)]
    pub pictures: Vec<Images>,
    #[serde(default)]
    pub statistics: Option<Statistics>,
    pub more_info: Option<String>,
    #[serde(default)]
    pub recommendations: Vec<Recommendation>,
}

// ========================================================================
// MyAnimeList API Response Models
// ========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalAnimeResponse {
    pub id: i32,
    pub title: String,
    #[serde(default)]
    pub main_picture: Option<MalPicture>,
    #[serde(default)]
    pub alternative_titles: Option<MalAlternativeTitles>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    #[serde(default)]
    pub synopsis: Option<String>,
    pub mean: Option<f32>,
    pub rank: Option<i32>,
    pub popularity: Option<i32>,
    pub num_list_users: Option<i32>,
    pub num_scoring_users: Option<i32>,
    pub nsfw: Option<String>,
    #[serde(default)]
    pub genres: Vec<MalGenre>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub media_type: Option<String>,
    pub status: Option<String>,
    pub num_episodes: Option<i32>,
    #[serde(default)]
    pub start_season: Option<MalSeason>,
    #[serde(default)]
    pub broadcast: Option<MalBroadcast>,
    pub source: Option<String>,
    pub average_episode_duration: Option<i32>,
    pub rating: Option<String>,
    #[serde(default)]
    pub studios: Vec<MalStudio>,
    #[serde(default)]
    pub pictures: Vec<MalPicture>,
    pub background: Option<String>,
    #[serde(default)]
    pub related_anime: Vec<MalRelatedAnime>,
    #[serde(default)]
    pub related_manga: Vec<MalRelatedManga>,
    #[serde(default)]
    pub statistics: Option<MalStatistics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalPicture {
    pub medium: Option<String>,
    pub large: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalAlternativeTitles {
    pub synonyms: Option<Vec<String>>,
    pub en: Option<String>,
    pub ja: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalGenre {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalSeason {
    pub year: i32,
    pub season: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalBroadcast {
    pub day_of_the_week: Option<String>,
    pub start_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalStudio {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalRelatedAnime {
    pub node: MalNode,
    pub relation_type: String,
    pub relation_type_formatted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalRelatedManga {
    pub node: MalNode,
    pub relation_type: String,
    pub relation_type_formatted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalNode {
    pub id: i32,
    pub title: String,
    pub main_picture: Option<MalPicture>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalStatistics {
    pub num_list_users: i32,
    pub status: MalStatusStats,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MalStatusStats {
    pub watching: Option<i32>,
    pub completed: Option<i32>,
    pub on_hold: Option<i32>,
    pub dropped: Option<i32>,
    pub plan_to_watch: Option<i32>,
}

// ========================================================================
// Jikan API Response Models
// ========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanAnimeResponse {
    pub data: JikanAnime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanAnime {
    pub mal_id: i32,
    pub url: String,
    pub images: JikanImages,
    #[serde(default)]
    pub trailer: JikanTrailer,
    pub approved: bool,
    pub titles: Vec<JikanTitle>,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    pub source: Option<String>,
    pub episodes: Option<i32>,
    pub status: Option<String>,
    pub airing: bool,
    pub aired: JikanAired,
    pub duration: Option<String>,
    pub rating: Option<String>,
    pub score: Option<f32>,
    pub scored_by: Option<i32>,
    pub rank: Option<i32>,
    pub popularity: Option<i32>,
    pub members: Option<i32>,
    pub favorites: Option<i32>,
    pub synopsis: Option<String>,
    pub background: Option<String>,
    pub season: Option<String>,
    pub year: Option<i32>,
    #[serde(default)]
    pub broadcast: JikanBroadcast,
    #[serde(default)]
    pub producers: Vec<JikanEntity>,
    #[serde(default)]
    pub licensors: Vec<JikanEntity>,
    #[serde(default)]
    pub studios: Vec<JikanEntity>,
    #[serde(default)]
    pub genres: Vec<JikanEntity>,
    #[serde(default)]
    pub explicit_genres: Vec<JikanEntity>,
    #[serde(default)]
    pub themes: Vec<JikanEntity>,
    #[serde(default)]
    pub demographics: Vec<JikanEntity>,
    #[serde(default)]
    pub relations: Vec<JikanRelation>,
    #[serde(default)]
    pub theme: JikanTheme,
    #[serde(default)]
    pub external: Vec<JikanExternal>,
    #[serde(default)]
    pub streaming: Vec<JikanStreaming>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanImages {
    pub jpg: JikanImage,
    pub webp: JikanImage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanImage {
    pub image_url: Option<String>,
    pub small_image_url: Option<String>,
    pub large_image_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JikanTrailer {
    pub youtube_id: Option<String>,
    pub url: Option<String>,
    pub embed_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanTitle {
    #[serde(rename = "type")]
    pub title_type: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanAired {
    pub from: Option<String>,
    pub to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JikanBroadcast {
    pub day: Option<String>,
    pub time: Option<String>,
    pub timezone: Option<String>,
    pub string: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanEntity {
    pub mal_id: i32,
    #[serde(rename = "type")]
    pub entity_type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanRelation {
    pub relation: String,
    pub entry: Vec<JikanRelationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanRelationEntry {
    pub mal_id: i32,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JikanTheme {
    #[serde(default)]
    pub openings: Vec<String>,
    #[serde(default)]
    pub endings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanExternal {
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JikanStreaming {
    pub name: String,
    pub url: String,
}

// ========================================================================
// Shared Models (used in final AnimeData)
// ========================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Title {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    #[serde(rename = "type")]
    pub title_type: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Images {
    pub jpg: Image,
    pub webp: Image,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Image {
    pub image_url: String,
    pub small_image_url: String,
    pub large_image_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Rating {
    G,
    PG,
    #[serde(rename = "PG-13")]
    PG13,
    #[serde(rename = "R-17+")]
    R17Plus,
    #[serde(rename = "R+")]
    RPlus,
    Rx,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "finished_airing")]
    FinishedAiring,
    #[serde(rename = "currently_airing")]
    CurrentlyAiring,
    #[serde(rename = "not_yet_aired")]
    NotYetAired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Source {
    #[serde(rename = "other")]
    Other,
    #[serde(rename = "original")]
    Original,
    #[serde(rename = "manga")]
    Manga,
    #[serde(rename = "4_koma_manga")]
    FourKomaManga,
    #[serde(rename = "web_manga")]
    WebManga,
    #[serde(rename = "digital_manga")]
    DigitalManga,
    #[serde(rename = "novel")]
    Novel,
    #[serde(rename = "light_novel")]
    LightNovel,
    #[serde(rename = "visual_novel")]
    VisualNovel,
    #[serde(rename = "game")]
    Game,
    #[serde(rename = "card_game")]
    CardGame,
    #[serde(rename = "book")]
    Book,
    #[serde(rename = "picture_book")]
    PictureBook,
    #[serde(rename = "radio")]
    Radio,
    #[serde(rename = "music")]
    Music,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MediaType {
    TV,
    OVA,
    Movie,
    Special,
    ONA,
    Music,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Studio {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub mal_id: i32,
    #[serde(rename = "type")]
    pub studio_type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Broadcast {
    pub day: Option<DayOfTheWeek>,
    pub time: Option<String>,
    pub timezone: Option<String>,
    pub string: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DayOfTheWeek {
    Sundays,
    Mondays,
    Tuesdays,
    Wednesdays,
    Thursdays,
    Fridays,
    Saturdays,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub mal_id: i32,
    #[serde(rename = "type")]
    pub genre_type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Themes {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub mal_id: i32,
    #[serde(rename = "type")]
    pub theme_type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Streaming {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Licensors {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub mal_id: i32,
    #[serde(rename = "type")]
    pub licensor_type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct External {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Producer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub mal_id: i32,
    #[serde(rename = "type")]
    pub producer_type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aired {
    pub from: Option<DateTime<Utc>>,
    pub to: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Season {
    #[serde(rename = "spring")]
    Spring,
    #[serde(rename = "summer")]
    Summer,
    #[serde(rename = "fall")]
    Fall,
    #[serde(rename = "winter")]
    Winter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub relation: String,
    pub entry: Vec<RelationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationEntry {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub mal_id: i32,
    #[serde(rename = "type")]
    pub entry_type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Theme {
    #[serde(default)]
    pub openings: Vec<String>,
    #[serde(default)]
    pub endings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trailer {
    pub youtube_id: Option<String>,
    pub url: Option<String>,
    pub embed_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Demographic {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<i32>,
    pub mal_id: i32,
    #[serde(rename = "type")]
    pub demographic_type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NSFW {
    #[serde(rename = "white")]
    White,
    #[serde(rename = "gray")]
    Gray,
    #[serde(rename = "black")]
    Black,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub watching: i32,
    pub completed: i32,
    pub on_hold: i32,
    pub dropped: i32,
    pub plan_to_watch: i32,
    pub total: i32,
    #[serde(default)]
    pub scores: Vec<StatisticsScore>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsScore {
    pub score: i32,
    pub votes: i32,
    pub percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Character {
    pub character: CharacterInfo,
    pub role: String,
    #[serde(default)]
    pub voice_actors: Vec<VoiceActor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterInfo {
    pub mal_id: i32,
    pub url: String,
    pub images: Images,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceActor {
    pub person: VoiceActorInfo,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceActorInfo {
    pub mal_id: i32,
    pub url: String,
    pub images: Images,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Staff {
    pub person: StaffInfo,
    pub positions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StaffInfo {
    pub mal_id: i32,
    pub url: String,
    pub images: Images,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Episode {
    pub mal_id: i32,
    pub url: Option<String>,
    pub title: String,
    pub title_japanese: Option<String>,
    pub title_romanji: Option<String>,
    pub duration: Option<i32>,
    pub aired: Option<DateTime<Utc>>,
    pub score: Option<f32>,
    pub filler: bool,
    pub recap: bool,
    #[serde(default)]
    pub forum_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Videos {
    #[serde(default)]
    pub promo: Vec<VideoPromoInfo>,
    #[serde(default)]
    pub episodes: Vec<VideoEpisodeInfo>,
    #[serde(default)]
    pub music_videos: Vec<VideoMusicInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoPromoInfo {
    pub title: String,
    pub trailer: VideoTrailer,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoEpisodeInfo {
    pub mal_id: i32,
    pub url: String,
    pub title: String,
    pub episode: String,
    pub images: Images,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoTrailer {
    pub youtube_id: Option<String>,
    pub url: Option<String>,
    pub embed_url: Option<String>,
    pub images: Option<Images>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMusicInfo {
    pub title: String,
    pub video: VideoTrailer,
    pub meta: VideoMusicMeta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoMusicMeta {
    pub title: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub entry: RecommendationInfo,
    pub url: String,
    pub votes: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecommendationInfo {
    pub mal_id: i32,
    pub url: String,
    pub images: Images,
    pub title: String,
}