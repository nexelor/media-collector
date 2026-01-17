use std::str::FromStr;

use chrono::{DateTime, NaiveTime, Utc};

use crate::global::model::ParseEnumError;

#[derive(Debug)]
pub struct Anime {
    pub id: Option<i32>,
    pub mal_id: i32, // Jikan
    pub url: String, // Jikan
    pub images: Images, // Jikan
    pub trailer: Trailer, // Jikan
    pub approved: bool, // Jikan
    pub titles: Vec<Title>, // Jikan
    pub media_type: Option<MediaType>, // Jikan
    pub nsfw: Option<NSFW>, // Mal
    pub source: Option<Source>, // Mal
    pub num_episode: i32, // Jikan
    pub average_episode_duration: i32, // Mal
    pub status: Option<Status>, // Jikan
    pub airing: bool, // Jikan
    pub aired: Aired, // Jikan
    pub duration: String, // Jikan
    pub rating: Option<Rating>, // Jikan
    pub score: Option<f32>, // Jikan
    pub scored_by: i32, // Jikan
    pub rank: Option<i32>, // Jikan
    pub members: i32, // Jikan
    pub favorites: i32, // Jikan
    pub popularity: Option<i32>, // Jikan,
    pub synopsis: String, // Jikan
    pub background: Option<String>, // Jikan
    pub season: Option<Season>, // Jikan
    pub year: Option<i32>, // Jikan
    pub broadcast: Broadcast, // Jikan
    pub producers: Vec<Producer>, // Jikan
    pub licensors: Vec<Licensors>, // Jikan
    pub studios: Vec<Studio>, // Jikan
    pub genres: Vec<Genre>, // Jikan
    pub explicit_genres: Vec<Genre>, // Jikan
    pub themes: Vec<Themes>, // Jikan
    pub demographics: Vec<Demographic>, // Jikan
    pub relations: Vec<Relation>, // Jikan
    pub theme: Theme, // Jikan
    pub external: Vec<External>, // Jikan
    pub streaming: Vec<Streaming>, // Jikan
    pub created_at: DateTime<Utc>, // Mal
    pub updated_at: DateTime<Utc>, // Mal
    
    pub characters: Vec<Character>, // Jikan
    pub staffs: Vec<Staff>, // Jikan
    pub episodes: Vec<Episode>, // Jikan
    pub videos: Videos, // Jikan
    pub pictures: Vec<Images>, // Jikan
    pub statistics: Statistics, // Jikan
    pub more_info: Option<String>, // Jikan
    pub recommendations: Vec<Recommendation>, // Jikan

    // pub reviews: Vec<Review>, // Jikan, not to sure to scrape that or not
}

#[derive(Debug)]
pub struct Title {
    pub id: Option<i32>,
    pub _type: String,
    pub title: String,
}

#[derive(Debug)]
pub struct Images {
    pub id: Option<i32>,
    pub jpg: Image,
    pub webp: Image,
}

#[derive(Debug)]
pub struct Image {
    pub image_url: String,
    pub small_image_url: String,
    pub large_image_url: String,
}

#[derive(Debug)]
pub enum Rating {
    G, // "G - All Ages",
    PG, // "PG - Children",
    PG_13, // "PG-13 - Teens 13 or older",
    R_17_PLUS, // "R - 17+ (violence & profanity)",
    R_PLUS, // "R+ - Mild Nudity",
    RX, // "Rx - Hentai"
}

impl FromStr for Rating {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "G - All Ages" => Ok(Self::G),
            "PG - Children" => Ok(Self::PG),
            "PG-13 - Teens 13 or older" => Ok(Self::PG_13),
            "R - 17+ (violence & profanity)" => Ok(Self::R_17_PLUS),
            "R+ - Mild Nudity" => Ok(Self::R_PLUS),
            "Rx - Hentai" => Ok(Self::RX),
            _ => Err(ParseEnumError {
                enum_name: "Rating",
                value: s.into(),
                expected: &["G - All Ages", "PG - Children", "PG-13 - Teens 13 or older",
                    "R - 17+ (violence & profanity)", "R+ - Mild Nudity", "Rx - Hentai"],
            }),
        }
    }
}

#[derive(Debug)]
pub enum Status {
    FINISHED_AIRING,
    CURRENTLY_AIRING,
    NOT_YET_AIRED,
}

impl FromStr for Status {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Finished Airing" => Ok(Self::FINISHED_AIRING),
            "Currently Airing" => Ok(Self::CURRENTLY_AIRING),
            "Not yet aired" => Ok(Self::NOT_YET_AIRED),
            _ => Err(ParseEnumError {
                enum_name: "Status",
                value: s.into(),
                expected: &["Finished Airing", "Currently Airing", "Not yet aired"],
            }),
        }
    }
}

#[derive(Debug)]
pub enum Source {
    OTHER,
    ORIGINAL,
    MANGA,
    FOUR_KOMA_MANGA,
    WEB_MANGA,
    DIGITAL_MANGA,
    NOVEL,
    LIGHT_NOVEL,
    VISUAL_NOVEL,
    GAME,
    CARD_GAME,
    BOOK,
    PICTURE_BOOK,
    RADIO,
    MUSIC,
}

impl FromStr for Source {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "original" => Self::ORIGINAL,
            "manga" => Self::MANGA,
            "4_koma_manga" => Self::FOUR_KOMA_MANGA,
            "web_manga" => Self::WEB_MANGA,
            "digital_manga" => Self::DIGITAL_MANGA,
            "novel" => Self::NOVEL,
            "light_novel" => Self::LIGHT_NOVEL,
            "visual_novel" => Self::VISUAL_NOVEL,
            "game" => Self::GAME,
            "card_game" => Self::CARD_GAME,
            "book" => Self::BOOK,
            "picture_book" => Self::PICTURE_BOOK,
            "radio" => Self::RADIO,
            "music" => Self::MUSIC,
            _ => Self::OTHER,
        })
    }
}

#[derive(Debug)]
pub enum MediaType {
    TV,
    OVA,
    MOVIE,
    SPECIAL,
    ONA,
    MUSIC,
    UNKNOWN,
}

impl FromStr for MediaType {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "TV" => Self::TV,
            "OVA" => Self::OVA,
            "Movie" => Self::MOVIE,
            "Special" => Self::SPECIAL,
            "ONA" => Self::ONA,
            "Music" => Self::MUSIC,
            _ => Self::UNKNOWN,
        })
    }
}

#[derive(Debug)]
pub struct Studio {
    pub id: Option<i32>,
    pub mal_id: i32,
    pub _type: String, // TODO - Make this an enum with all the possible value
    pub name: String,
    pub url: String,
}

#[derive(Debug)]
pub struct Broadcast {
    pub id: Option<i32>,
    pub day: Option<DayOfTheWeek>,
    pub time: Option<NaiveTime>,
    pub timezone: Option<String>, // TODO Convert this to an enum
    pub string: Option<String>,
}

#[derive(Debug)]
pub enum DayOfTheWeek {
    SUNDAY,
    MONDAY,
    TUESDAY,
    WEDNESDAY,
    THURSDAY,
    FRIDAY,
    SATURDAYS,
    OTHER, // Fallback of there is no match
}

impl FromStr for DayOfTheWeek {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "Sunday" => Self::SUNDAY,
            "Monday" => Self::MONDAY,
            "Tuesday" => Self::TUESDAY,
            "Wednesday" => Self::WEDNESDAY,
            "Thursday" => Self::THURSDAY,
            "Friday" => Self::FRIDAY,
            "Saturdays" => Self::SATURDAYS,
            _ => Self::OTHER,
        })
    }
}

#[derive(Debug)]
pub struct Genre {
    pub id: Option<i32>,
    pub mal_id: i32,
    pub _type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug)]
pub struct Themes {
    pub id: Option<i32>,
    pub mal_id: i32,
    pub _type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug)]
pub struct Streaming {
    pub id: Option<i32>,
    pub name: String,
    pub url: String,
}

#[derive(Debug)]
pub struct Licensors {
    pub id: Option<i32>,
    pub mal_id: i32,
    pub _type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug)]
pub struct External {
    pub id: Option<i32>,
    pub name: String,
    pub url: String,
}

#[derive(Debug)]
pub struct Producer {
    pub id: Option<i32>,
    pub mal_id: i32,
    pub _type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug)]
pub struct Aired {
    pub id: Option<i32>,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

#[derive(Debug)]
pub enum Season {
    SPRING,
    SUMMER,
    FALL,
    WINTER,
}

impl FromStr for Season {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "spring" => Ok(Self::SPRING),
            "summer" => Ok(Self::SUMMER),
            "fall" | "autumn" => Ok(Self::FALL),
            "winter" => Ok(Self::WINTER),
            _ => Err(ParseEnumError {
                enum_name: "Season",
                value: s.into(),
                expected: &["spring", "summer", "fall", "winter"],
            }),
        }
    }
}

#[derive(Debug)]
pub struct Relation {
    pub id: Option<i32>,
    pub relation: String, // TODO - Convert this to an enum
    pub entry: Vec<RelationEntry>,
}

#[derive(Debug)]
pub struct RelationEntry {
    pub id: Option<i32>,
    pub mal_id: i32,
    pub _type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug)]
pub struct Theme {
    pub openings: Vec<String>,
    pub endings: Vec<String>,
}

#[derive(Debug)]
pub struct Trailer {
    pub youtube_id: Option<String>,
    pub url: Option<String>,
    pub embed_url: Option<String>,
}

#[derive(Debug)]
pub struct Demographic {
    pub id: Option<i32>,
    pub mal_id: i32,
    pub _type: String,
    pub name: String,
    pub url: String,
}

#[derive(Debug)]
pub enum NSFW {
    WHITE,
    GRAY,
    BLACK,
}

impl FromStr for NSFW {
    type Err = ParseEnumError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "white" => Ok(Self::WHITE),
            "gray" => Ok(Self::GRAY),
            "black" => Ok(Self::BLACK),
            _ => Err(ParseEnumError {
                enum_name: "NSFW",
                value: s.into(),
                expected: &["white", "gray", "black"],
            }),
        }
    }
}

#[derive(Debug)]
pub struct Statistics {
    pub id: Option<i32>,
    pub watching: i32,
    pub completed: i32,
    pub on_hold: i32,
    pub dropped: i32,
    pub plan_to_watch: i32,
    pub total: i32,
    pub scores: StatisticsScore,
}

#[derive(Debug)]
pub struct StatisticsScore {
    pub id: Option<i32>,
    pub score: i32,
    pub votes: i32,
    pub percentage: f32,
}

#[derive(Debug)]
pub struct CharacterInfo {
    pub id: Option<i32>,
    pub mal_id: i32,
    pub url: String,
    pub images: Images,
    pub name: String,
}

#[derive(Debug)]
pub struct Character {
    pub character: CharacterInfo,
    pub role: String, // TODO - Convert to an enum
    pub voice_actors: Vec<VoiceActor>,
}

#[derive(Debug)]
pub struct VoiceActorInfo {
    pub id: Option<i32>,
    pub mal_id: i32,
    pub url: String,
    pub images: Images,
    pub name: String,
}

#[derive(Debug)]
pub struct VoiceActor {
    pub id: Option<i32>,
    pub person: VoiceActorInfo,
    pub language: String
}

#[derive(Debug)]
pub struct StaffInfo {
    pub id: Option<i32>,
    pub mal_id: i32,
    pub url: String,
    pub images: Images,
    pub name: String,
}

#[derive(Debug)]
pub struct Staff {
    pub id: Option<i32>,
    pub person: StaffInfo,
    pub positions: Vec<String>,
}

#[derive(Debug)]
pub struct Episode {
    pub id: Option<i32>,
    pub mal_id: i32,
    pub url: Option<String>,
    pub title: String,
    pub title_japanese: Option<String>,
    pub title_romanji: Option<String>,
    pub duration: Option<i32>,
    pub aired: DateTime<Utc>,
    pub score: Option<f32>,
    pub filler: bool,
    pub recap: bool,
    pub images: EpisodeImage,
}

#[derive(Debug)]
pub struct EpisodeImage {
    pub jpg: EpisodeImageUrl,
}

#[derive(Debug)]
pub struct EpisodeImageUrl {
    pub image_url: Option<String>,
}

#[derive(Debug)]
pub struct VideoPromoInfo {
    pub title: String,
    pub trailer: VideoTrailer,
}

#[derive(Debug)]
pub struct VideoTrailer {
    pub id: Option<i32>,
    pub youtube_id: Option<String>,
    pub url: Option<String>,
    pub embed_url: Option<String>,
}

#[derive(Debug)]
pub struct VideoMusicMeta {
    pub id: Option<i32>,
    pub title: Option<String>,
    pub author: Option<String>,
}

#[derive(Debug)]
pub struct VideoMusicInfo {
    pub title: String,
    pub video: VideoTrailer,
    pub meta: VideoMusicMeta,
}

#[derive(Debug)]
pub struct Videos {
    pub id: Option<i32>,
    pub promo: Vec<VideoPromoInfo>,
    pub music_videos: Vec<VideoMusicInfo>,
}

#[derive(Debug)]
pub struct RecommendationInfo {
    pub id: Option<i32>,
    pub mal_id: i32,
    pub url: String,
    pub images: Images,
    pub title: String,
}

#[derive(Debug)]
pub struct Recommendation {
    pub id: Option<i32>,
    pub entry: RecommendationInfo,
    pub url: String,
    pub votes: i32,
}