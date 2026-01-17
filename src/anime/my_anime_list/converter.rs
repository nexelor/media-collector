use chrono::{DateTime, Utc, NaiveDateTime};
use tracing::{debug, warn};

use super::model::*;

/// Convert MyAnimeList API response to unified AnimeData
pub fn mal_to_anime_data(mal: MalAnimeResponse, jikan_url: Option<String>) -> AnimeData {
    let mal_id = mal.id;
    
    // Build default images from main_picture
    let images = if let Some(pic) = &mal.main_picture {
        Images {
            jpg: Image {
                image_url: pic.large.clone().unwrap_or_default(),
                small_image_url: pic.medium.clone().unwrap_or_default(),
                large_image_url: pic.large.clone().unwrap_or_default(),
            },
            webp: Image {
                image_url: String::new(),
                small_image_url: String::new(),
                large_image_url: String::new(),
            },
        }
    } else {
        default_images()
    };

    // Build titles
    let mut titles = vec![Title {
        id: None,
        title_type: "Default".to_string(),
        title: mal.title.clone(),
    }];

    if let Some(alt_titles) = &mal.alternative_titles {
        if let Some(en) = &alt_titles.en {
            titles.push(Title {
                id: None,
                title_type: "English".to_string(),
                title: en.clone(),
            });
        }
        if let Some(ja) = &alt_titles.ja {
            titles.push(Title {
                id: None,
                title_type: "Japanese".to_string(),
                title: ja.clone(),
            });
        }
        if let Some(synonyms) = &alt_titles.synonyms {
            for syn in synonyms {
                titles.push(Title {
                    id: None,
                    title_type: "Synonym".to_string(),
                    title: syn.clone(),
                });
            }
        }
    }

    // Parse dates
    let (aired_from, aired_to) = parse_mal_dates(&mal.start_date, &mal.end_date);
    
    // Convert studios
    let studios = mal.studios.iter().map(|s| Studio {
        id: None,
        mal_id: s.id,
        studio_type: "anime_studios".to_string(),
        name: s.name.clone(),
        url: format!("https://myanimelist.net/anime/producer/{}", s.id),
    }).collect();

    // Convert genres
    let genres = mal.genres.iter().map(|g| Genre {
        id: None,
        mal_id: g.id,
        genre_type: "anime_genres".to_string(),
        name: g.name.clone(),
        url: format!("https://myanimelist.net/anime/genre/{}", g.id),
    }).collect();

    // Parse broadcast
    let broadcast = if let Some(b) = &mal.broadcast {
        Broadcast {
            day: b.day_of_the_week.as_ref().and_then(|d| parse_day_of_week(d)),
            time: b.start_time.clone(),
            timezone: Some("Asia/Tokyo".to_string()),
            string: Some(format!("{} at {}", 
                b.day_of_the_week.as_deref().unwrap_or("Unknown"),
                b.start_time.as_deref().unwrap_or("Unknown")
            )),
        }
    } else {
        Broadcast {
            day: None,
            time: None,
            timezone: None,
            string: None,
        }
    };

    // Parse season
    let (season, year) = if let Some(s) = &mal.start_season {
        (parse_season(&s.season), Some(s.year))
    } else {
        (None, None)
    };

    // Convert statistics
    let statistics = mal.statistics.as_ref().map(|s| Statistics {
        watching: s.status.watching.unwrap_or(0),
        completed: s.status.completed.unwrap_or(0),
        on_hold: s.status.on_hold.unwrap_or(0),
        dropped: s.status.dropped.unwrap_or(0),
        plan_to_watch: s.status.plan_to_watch.unwrap_or(0),
        total: s.num_list_users,
        scores: vec![],
    });

    AnimeData {
        id: None,
        mal_id,
        url: jikan_url.unwrap_or_else(|| format!("https://myanimelist.net/anime/{}", mal_id)),
        images,
        trailer: Trailer {
            youtube_id: None,
            url: None,
            embed_url: None,
        },
        approved: true,
        titles,
        media_type: mal.media_type.as_ref().and_then(|m| parse_media_type(m)),
        nsfw: mal.nsfw.as_ref().and_then(|n| parse_nsfw(n)),
        source: mal.source.as_ref().and_then(|s| parse_source(s)),
        num_episodes: mal.num_episodes.unwrap_or(0),
        average_episode_duration: mal.average_episode_duration.unwrap_or(0),
        status: mal.status.as_ref().and_then(|s| parse_status(s)),
        airing: mal.status.as_deref() == Some("currently_airing"),
        aired: Aired {
            from: aired_from,
            to: aired_to,
        },
        duration: format!("{} min per ep", mal.average_episode_duration.unwrap_or(0) / 60),
        rating: mal.rating.as_ref().and_then(|r| parse_rating(r)),
        score: mal.mean,
        scored_by: mal.num_scoring_users.unwrap_or(0),
        rank: mal.rank,
        members: mal.num_list_users.unwrap_or(0),
        favorites: 0,
        popularity: mal.popularity,
        synopsis: mal.synopsis.unwrap_or_default(),
        background: mal.background,
        season,
        year,
        broadcast,
        producers: vec![],
        licensors: vec![],
        studios,
        genres,
        explicit_genres: vec![],
        themes: vec![],
        demographics: vec![],
        relations: convert_mal_relations(&mal.related_anime, &mal.related_manga),
        theme: Theme::default(),
        external: vec![],
        streaming: vec![],
        created_at: parse_mal_timestamp(&mal.created_at).unwrap_or_else(Utc::now),
        updated_at: parse_mal_timestamp(&mal.updated_at).unwrap_or_else(Utc::now),
        characters: vec![],
        staffs: vec![],
        episodes: vec![],
        videos: None,
        pictures: mal.pictures.iter().map(|p| Images {
            jpg: Image {
                image_url: p.large.clone().unwrap_or_default(),
                small_image_url: p.medium.clone().unwrap_or_default(),
                large_image_url: p.large.clone().unwrap_or_default(),
            },
            webp: Image {
                image_url: String::new(),
                small_image_url: String::new(),
                large_image_url: String::new(),
            },
        }).collect(),
        statistics,
        more_info: None,
        recommendations: vec![],
    }
}

/// Merge Jikan data into existing AnimeData (enrichment)
pub fn merge_jikan_data(mut anime: AnimeData, jikan: JikanAnime) -> AnimeData {
    debug!(mal_id = anime.mal_id, "Merging Jikan data into anime");

    // Update URL
    anime.url = jikan.url;

    // Update images with higher quality Jikan images
    anime.images = Images {
        jpg: Image {
            image_url: jikan.images.jpg.image_url.clone().unwrap_or_default(),
            small_image_url: jikan.images.jpg.small_image_url.clone().unwrap_or_default(),
            large_image_url: jikan.images.jpg.large_image_url.clone().unwrap_or_default(),
        },
        webp: Image {
            image_url: jikan.images.webp.image_url.clone().unwrap_or_default(),
            small_image_url: jikan.images.webp.small_image_url.clone().unwrap_or_default(),
            large_image_url: jikan.images.webp.large_image_url.clone().unwrap_or_default(),
        },
    };

    // Update trailer
    anime.trailer = Trailer {
        youtube_id: jikan.trailer.youtube_id,
        url: jikan.trailer.url,
        embed_url: jikan.trailer.embed_url,
    };

    // Update approval status
    anime.approved = jikan.approved;

    // Merge titles (Jikan has more comprehensive title data)
    anime.titles = jikan.titles.iter().map(|t| Title {
        id: None,
        title_type: t.title_type.clone(),
        title: t.title.clone(),
    }).collect();

    // Update aired dates with Jikan data
    anime.aired = Aired {
        from: jikan.aired.from.as_ref().and_then(|s| parse_jikan_date(s)),
        to: jikan.aired.to.as_ref().and_then(|s| parse_jikan_date(s)),
    };

    // Update duration
    if let Some(duration) = jikan.duration {
        anime.duration = duration;
    }

    // Update favorites
    anime.favorites = jikan.favorites.unwrap_or(0);

    // Update background
    if jikan.background.is_some() {
        anime.background = jikan.background;
    }

    // Update broadcast with Jikan data (more detailed)
    anime.broadcast = Broadcast {
        day: jikan.broadcast.day.as_ref().and_then(|d| parse_day_of_week(d)),
        time: jikan.broadcast.time.clone(),
        timezone: jikan.broadcast.timezone.clone(),
        string: jikan.broadcast.string.clone(),
    };

    // Convert producers
    anime.producers = jikan.producers.iter().map(|p| Producer {
        id: None,
        mal_id: p.mal_id,
        producer_type: p.entity_type.clone(),
        name: p.name.clone(),
        url: p.url.clone(),
    }).collect();

    // Convert licensors
    anime.licensors = jikan.licensors.iter().map(|l| Licensors {
        id: None,
        mal_id: l.mal_id,
        licensor_type: l.entity_type.clone(),
        name: l.name.clone(),
        url: l.url.clone(),
    }).collect();

    // Merge studios (combine MAL and Jikan)
    let jikan_studios: Vec<Studio> = jikan.studios.iter().map(|s| Studio {
        id: None,
        mal_id: s.mal_id,
        studio_type: s.entity_type.clone(),
        name: s.name.clone(),
        url: s.url.clone(),
    }).collect();
    anime.studios = jikan_studios;

    // Update explicit genres
    anime.explicit_genres = jikan.explicit_genres.iter().map(|g| Genre {
        id: None,
        mal_id: g.mal_id,
        genre_type: g.entity_type.clone(),
        name: g.name.clone(),
        url: g.url.clone(),
    }).collect();

    // Update themes
    anime.themes = jikan.themes.iter().map(|t| Themes {
        id: None,
        mal_id: t.mal_id,
        theme_type: t.entity_type.clone(),
        name: t.name.clone(),
        url: t.url.clone(),
    }).collect();

    // Update demographics
    anime.demographics = jikan.demographics.iter().map(|d| Demographic {
        id: None,
        mal_id: d.mal_id,
        demographic_type: d.entity_type.clone(),
        name: d.name.clone(),
        url: d.url.clone(),
    }).collect();

    // Update relations
    anime.relations = jikan.relations.iter().map(|r| Relation {
        id: None,
        relation: r.relation.clone(),
        entry: r.entry.iter().map(|e| RelationEntry {
            id: None,
            mal_id: e.mal_id,
            entry_type: e.entry_type.clone(),
            name: e.name.clone(),
            url: e.url.clone(),
        }).collect(),
    }).collect();

    // Update theme songs
    anime.theme = Theme {
        openings: jikan.theme.openings.clone(),
        endings: jikan.theme.endings.clone(),
    };

    // Update external links
    anime.external = jikan.external.iter().map(|e| External {
        id: None,
        name: e.name.clone(),
        url: e.url.clone(),
    }).collect();

    // Update streaming links
    anime.streaming = jikan.streaming.iter().map(|s| Streaming {
        id: None,
        name: s.name.clone(),
        url: s.url.clone(),
    }).collect();

    anime
}

// ========================================================================
// Helper Functions
// ========================================================================

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

fn parse_mal_dates(start: &Option<String>, end: &Option<String>) -> (Option<DateTime<Utc>>, Option<DateTime<Utc>>) {
    let from = start.as_ref().and_then(|s| parse_mal_date(s));
    let to = end.as_ref().and_then(|s| parse_mal_date(s));
    (from, to)
}

fn parse_mal_date(date_str: &str) -> Option<DateTime<Utc>> {
    NaiveDateTime::parse_from_str(&format!("{} 00:00:00", date_str), "%Y-%m-%d %H:%M:%S")
        .ok()
        .map(|ndt| DateTime::from_naive_utc_and_offset(ndt, Utc))
}

fn parse_jikan_date(date_str: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(date_str)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
}

fn parse_mal_timestamp(timestamp: &Option<String>) -> Option<DateTime<Utc>> {
    timestamp.as_ref().and_then(|s| {
        DateTime::parse_from_rfc3339(s)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
    })
}

fn parse_media_type(s: &str) -> Option<MediaType> {
    match s.to_lowercase().as_str() {
        "tv" => Some(MediaType::TV),
        "ova" => Some(MediaType::OVA),
        "movie" => Some(MediaType::Movie),
        "special" => Some(MediaType::Special),
        "ona" => Some(MediaType::ONA),
        "music" => Some(MediaType::Music),
        _ => Some(MediaType::Unknown),
    }
}

fn parse_nsfw(s: &str) -> Option<NSFW> {
    match s {
        "white" => Some(NSFW::White),
        "gray" => Some(NSFW::Gray),
        "black" => Some(NSFW::Black),
        _ => None,
    }
}

fn parse_source(s: &str) -> Option<Source> {
    match s {
        "original" => Some(Source::Original),
        "manga" => Some(Source::Manga),
        "4_koma_manga" => Some(Source::FourKomaManga),
        "web_manga" => Some(Source::WebManga),
        "digital_manga" => Some(Source::DigitalManga),
        "novel" => Some(Source::Novel),
        "light_novel" => Some(Source::LightNovel),
        "visual_novel" => Some(Source::VisualNovel),
        "game" => Some(Source::Game),
        "card_game" => Some(Source::CardGame),
        "book" => Some(Source::Book),
        "picture_book" => Some(Source::PictureBook),
        "radio" => Some(Source::Radio),
        "music" => Some(Source::Music),
        _ => Some(Source::Other),
    }
}

fn parse_status(s: &str) -> Option<Status> {
    match s {
        "finished_airing" => Some(Status::FinishedAiring),
        "currently_airing" => Some(Status::CurrentlyAiring),
        "not_yet_aired" => Some(Status::NotYetAired),
        _ => None,
    }
}

fn parse_rating(s: &str) -> Option<Rating> {
    match s {
        "g" => Some(Rating::G),
        "pg" => Some(Rating::PG),
        "pg_13" => Some(Rating::PG13),
        "r" => Some(Rating::R17Plus),
        "r+" => Some(Rating::RPlus),
        "rx" => Some(Rating::Rx),
        _ => None,
    }
}

fn parse_season(s: &str) -> Option<Season> {
    match s.to_lowercase().as_str() {
        "spring" => Some(Season::Spring),
        "summer" => Some(Season::Summer),
        "fall" | "autumn" => Some(Season::Fall),
        "winter" => Some(Season::Winter),
        _ => None,
    }
}

fn parse_day_of_week(s: &str) -> Option<DayOfTheWeek> {
    match s.to_lowercase().as_str() {
        "sundays" | "sunday" => Some(DayOfTheWeek::Sundays),
        "mondays" | "monday" => Some(DayOfTheWeek::Mondays),
        "tuesdays" | "tuesday" => Some(DayOfTheWeek::Tuesdays),
        "wednesdays" | "wednesday" => Some(DayOfTheWeek::Wednesdays),
        "thursdays" | "thursday" => Some(DayOfTheWeek::Thursdays),
        "fridays" | "friday" => Some(DayOfTheWeek::Fridays),
        "saturdays" | "saturday" => Some(DayOfTheWeek::Saturdays),
        _ => Some(DayOfTheWeek::Other),
    }
}

fn convert_mal_relations(
    anime_relations: &[MalRelatedAnime],
    manga_relations: &[MalRelatedManga],
) -> Vec<Relation> {
    let mut relations: std::collections::HashMap<String, Vec<RelationEntry>> = std::collections::HashMap::new();

    // Process anime relations
    for rel in anime_relations {
        relations.entry(rel.relation_type_formatted.clone())
            .or_insert_with(Vec::new)
            .push(RelationEntry {
                id: None,
                mal_id: rel.node.id,
                entry_type: "anime".to_string(),
                name: rel.node.title.clone(),
                url: format!("https://myanimelist.net/anime/{}", rel.node.id),
            });
    }

    // Process manga relations
    for rel in manga_relations {
        relations.entry(rel.relation_type_formatted.clone())
            .or_insert_with(Vec::new)
            .push(RelationEntry {
                id: None,
                mal_id: rel.node.id,
                entry_type: "manga".to_string(),
                name: rel.node.title.clone(),
                url: format!("https://myanimelist.net/manga/{}", rel.node.id),
            });
    }

    // Convert to Vec<Relation>
    relations.into_iter().map(|(relation, entry)| Relation {
        id: None,
        relation,
        entry,
    }).collect()
}