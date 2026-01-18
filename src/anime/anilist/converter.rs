use chrono::{DateTime, NaiveDate, Utc};
use tracing::{debug, warn};

use super::model::*;
use crate::anime::my_anime_list::model::*;

/// Convert AniList media to AniList-specific data structure
pub fn anilist_to_anime_data(anilist: AniListMedia) -> AniListAnimeData {
    let anilist_id = anilist.id;
    let mal_id = anilist.id_mal;
    
    // Build images from cover image
    let images = if let Some(cover) = &anilist.cover_image {
        Images {
            jpg: Image {
                image_url: cover.extra_large.clone().unwrap_or_default(),
                small_image_url: cover.medium.clone().unwrap_or_default(),
                large_image_url: cover.large.clone().unwrap_or_default(),
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
    let mut titles = Vec::new();
    
    if let Some(romaji) = &anilist.title.romaji {
        titles.push(Title {
            id: None,
            title_type: "Default".to_string(),
            title: romaji.clone(),
        });
    }
    
    if let Some(english) = &anilist.title.english {
        titles.push(Title {
            id: None,
            title_type: "English".to_string(),
            title: english.clone(),
        });
    }
    
    if let Some(native) = &anilist.title.native {
        titles.push(Title {
            id: None,
            title_type: "Japanese".to_string(),
            title: native.clone(),
        });
    }
    
    for synonym in &anilist.synonyms {
        titles.push(Title {
            id: None,
            title_type: "Synonym".to_string(),
            title: synonym.clone(),
        });
    }

    // Ensure we have at least one title
    if titles.is_empty() {
        titles.push(Title {
            id: None,
            title_type: "Default".to_string(),
            title: format!("Anime {}", anilist_id),
        });
    }

    // Parse dates
    let aired = Aired {
        from: anilist.start_date.as_ref().and_then(|d| fuzzy_date_to_datetime(d)),
        to: anilist.end_date.as_ref().and_then(|d| fuzzy_date_to_datetime(d)),
    };

    // Convert genres
    let genres = anilist.genres.iter().enumerate().map(|(i, g)| Genre {
        id: None,
        mal_id: i as i32 + 1,
        genre_type: "anime_genres".to_string(),
        name: g.clone(),
        url: format!("https://anilist.co/search/anime?genres={}", urlencoding::encode(g)),
    }).collect();

    // Convert studios
    let studios = if let Some(studio_conn) = &anilist.studios {
        studio_conn.edges.as_ref().map(|edges| {
            edges.iter().filter_map(|edge| {
                edge.node.as_ref().map(|node| Studio {
                    id: None,
                    mal_id: node.id,
                    studio_type: "anime_studios".to_string(),
                    name: node.name.clone(),
                    url: node.site_url.clone().unwrap_or_default(),
                })
            }).collect()
        }).unwrap_or_default()
    } else {
        vec![]
    };

    // Convert trailer
    let trailer = if let Some(t) = &anilist.trailer {
        Trailer {
            youtube_id: t.id.clone(),
            url: t.id.as_ref().map(|id| format!("https://www.youtube.com/watch?v={}", id)),
            embed_url: t.id.as_ref().map(|id| format!("https://www.youtube.com/embed/{}", id)),
        }
    } else {
        Trailer {
            youtube_id: None,
            url: None,
            embed_url: None,
        }
    };

    // Convert broadcast
    let broadcast = Broadcast {
        day: None,
        time: None,
        timezone: None,
        string: None,
    };

    // Convert season
    let season = anilist.season.as_ref().and_then(|s| parse_anilist_season(s));

    // Convert relations
    let relations = convert_anilist_relations(&anilist.relations);

    // Convert characters
    let characters = convert_anilist_characters(&anilist.characters);

    // Convert staff
    let staffs = convert_anilist_staff(&anilist.staff);

    // Convert themes (from tags)
    let themes = anilist.tags.iter().map(|tag| Themes {
        id: Some(tag.id),
        mal_id: tag.id,
        theme_type: tag.category.clone().unwrap_or_else(|| "theme".to_string()),
        name: tag.name.clone(),
        url: format!("https://anilist.co/search/anime?genres={}", urlencoding::encode(&tag.name)),
    }).collect();

    // Convert tags to AniList-specific format
    let tags = anilist.tags.iter().map(|tag| AniListTag {
        id: tag.id,
        name: tag.name.clone(),
        description: tag.description.clone(),
        category: tag.category.clone(),
        rank: tag.rank.unwrap_or(0),
        is_spoiler: tag.is_general_spoiler.unwrap_or(false) || tag.is_media_spoiler.unwrap_or(false),
    }).collect();

    // Convert external links
    let external = anilist.external_links.iter().map(|link| External {
        id: Some(link.id),
        name: link.site.clone(),
        url: link.url.clone(),
    }).collect();

    // Convert streaming episodes to streaming links
    let streaming = anilist.streaming_episodes.iter().filter_map(|ep| {
        ep.site.as_ref().map(|site| Streaming {
            id: None,
            name: site.clone(),
            url: ep.url.clone().unwrap_or_default(),
        })
    }).collect();

    // Convert statistics
    let statistics = anilist.stats.as_ref().map(|stats| {
        let score_dist = stats.score_distribution.as_ref().map(|dist| {
            dist.iter().filter_map(|s| {
                Some(StatisticsScore {
                    score: s.score?,
                    votes: s.amount?,
                    percentage: 0.0,
                })
            }).collect()
        }).unwrap_or_default();

        let status_dist = stats.status_distribution.as_ref();
        
        Statistics {
            watching: status_dist.as_ref()
                .and_then(|d| d.iter().find(|s| s.status.as_deref() == Some("CURRENT")))
                .and_then(|s| s.amount)
                .unwrap_or(0),
            completed: status_dist.as_ref()
                .and_then(|d| d.iter().find(|s| s.status.as_deref() == Some("COMPLETED")))
                .and_then(|s| s.amount)
                .unwrap_or(0),
            on_hold: status_dist.as_ref()
                .and_then(|d| d.iter().find(|s| s.status.as_deref() == Some("PAUSED")))
                .and_then(|s| s.amount)
                .unwrap_or(0),
            dropped: status_dist.as_ref()
                .and_then(|d| d.iter().find(|s| s.status.as_deref() == Some("DROPPED")))
                .and_then(|s| s.amount)
                .unwrap_or(0),
            plan_to_watch: status_dist.as_ref()
                .and_then(|d| d.iter().find(|s| s.status.as_deref() == Some("PLANNING")))
                .and_then(|s| s.amount)
                .unwrap_or(0),
            total: anilist.popularity.unwrap_or(0),
            scores: score_dist,
        }
    });

    // Calculate score (AniList uses 0-100 scale, convert to 0-10)
    let score = anilist.mean_score.map(|s| s as f32 / 10.0);

    // Next airing episode
    let next_airing_episode = anilist.next_airing_episode.as_ref().map(|ep| AniListAiringSchedule {
        airing_at: ep.airing_at,
        time_until_airing: ep.time_until_airing,
        episode: ep.episode,
    });

    AniListAnimeData {
        id: None,
        anilist_id,
        mal_id,
        url: anilist.site_url.clone().unwrap_or_else(|| format!("https://anilist.co/anime/{}", anilist_id)),
        images,
        trailer,
        titles,
        media_type: anilist.format.as_ref().and_then(|f| parse_anilist_format(f)),
        nsfw: if anilist.is_adult.unwrap_or(false) { Some(NSFW::Black) } else { Some(NSFW::White) },
        source: anilist.source.as_ref().and_then(|s| parse_anilist_source(s)),
        num_episodes: anilist.episodes.unwrap_or(0),
        average_episode_duration: anilist.duration.map(|d| d * 60).unwrap_or(0),
        status: anilist.status.as_ref().and_then(|s| parse_anilist_status(s)),
        airing: anilist.status.as_deref() == Some("RELEASING"),
        aired,
        duration: format!("{} min per ep", anilist.duration.unwrap_or(0)),
        score,
        popularity: anilist.popularity.unwrap_or(0),
        favorites: anilist.favourites.unwrap_or(0),
        synopsis: strip_html(&anilist.description.unwrap_or_default()),
        background: None,
        season,
        year: anilist.season_year,
        broadcast,
        studios,
        genres,
        themes,
        demographics: vec![],
        relations,
        theme: Theme::default(),
        external,
        streaming,
        characters,
        staffs,
        statistics,
        country_of_origin: anilist.country_of_origin,
        is_licensed: anilist.is_licensed.unwrap_or(false),
        hashtag: anilist.hashtag,
        trending: anilist.trending.unwrap_or(0),
        tags,
        next_airing_episode,
        banner_image: anilist.banner_image,
        cover_color: anilist.cover_image.as_ref().and_then(|c| c.color.clone()),
        created_at: Utc::now(),
        updated_at: anilist.updated_at.map(|t| DateTime::from_timestamp(t, 0).unwrap_or_else(Utc::now)).unwrap_or_else(Utc::now),
    }
}

// Keep all the helper functions from the previous converter (they're the same)
// ... (fuzzy_date_to_datetime, parse_anilist_format, etc.)

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

fn fuzzy_date_to_datetime(date: &FuzzyDate) -> Option<DateTime<Utc>> {
    let year = date.year?;
    let month = date.month.unwrap_or(1);
    let day = date.day.unwrap_or(1);
    
    NaiveDate::from_ymd_opt(year, month as u32, day as u32)
        .map(|d| d.and_hms_opt(0, 0, 0))
        .flatten()
        .map(|ndt| DateTime::from_naive_utc_and_offset(ndt, Utc))
}

fn parse_anilist_format(format: &str) -> Option<MediaType> {
    match format {
        "TV" => Some(MediaType::TV),
        "TV_SHORT" => Some(MediaType::TV),
        "OVA" => Some(MediaType::OVA),
        "MOVIE" => Some(MediaType::Movie),
        "SPECIAL" => Some(MediaType::Special),
        "ONA" => Some(MediaType::ONA),
        "MUSIC" => Some(MediaType::Music),
        _ => Some(MediaType::Unknown),
    }
}

fn parse_anilist_status(status: &str) -> Option<Status> {
    match status {
        "FINISHED" => Some(Status::FinishedAiring),
        "RELEASING" => Some(Status::CurrentlyAiring),
        "NOT_YET_RELEASED" => Some(Status::NotYetAired),
        "CANCELLED" => Some(Status::FinishedAiring),
        _ => None,
    }
}

fn parse_anilist_source(source: &str) -> Option<Source> {
    match source {
        "ORIGINAL" => Some(Source::Original),
        "MANGA" => Some(Source::Manga),
        "LIGHT_NOVEL" => Some(Source::LightNovel),
        "VISUAL_NOVEL" => Some(Source::VisualNovel),
        "VIDEO_GAME" => Some(Source::Game),
        "OTHER" => Some(Source::Other),
        "NOVEL" => Some(Source::Novel),
        "DOUJINSHI" => Some(Source::Manga),
        "ANIME" => Some(Source::Original),
        "WEB_NOVEL" => Some(Source::Novel),
        "LIVE_ACTION" => Some(Source::Other),
        "GAME" => Some(Source::Game),
        "COMIC" => Some(Source::Manga),
        "MULTIMEDIA_PROJECT" => Some(Source::Other),
        "PICTURE_BOOK" => Some(Source::PictureBook),
        _ => Some(Source::Other),
    }
}

fn parse_anilist_season(season: &str) -> Option<Season> {
    match season.to_uppercase().as_str() {
        "SPRING" => Some(Season::Spring),
        "SUMMER" => Some(Season::Summer),
        "FALL" | "AUTUMN" => Some(Season::Fall),
        "WINTER" => Some(Season::Winter),
        _ => None,
    }
}

fn convert_anilist_relations(relations: &Option<MediaConnection>) -> Vec<Relation> {
    relations.as_ref()
        .and_then(|conn| conn.edges.as_ref())
        .map(|edges| {
            let mut relation_map: std::collections::HashMap<String, Vec<RelationEntry>> = 
                std::collections::HashMap::new();

            for edge in edges {
                if let (Some(rel_type), Some(node)) = (&edge.relation_type, &edge.node) {
                    let entry = RelationEntry {
                        id: None,
                        mal_id: node.id,
                        entry_type: node.media_type.clone().unwrap_or_else(|| "ANIME".to_string()).to_lowercase(),
                        name: node.title.as_ref()
                            .and_then(|t| t.romaji.clone().or_else(|| t.english.clone()))
                            .unwrap_or_else(|| format!("Unknown {}", node.id)),
                        url: format!("https://anilist.co/anime/{}", node.id),
                    };

                    relation_map.entry(rel_type.clone())
                        .or_insert_with(Vec::new)
                        .push(entry);
                }
            }

            relation_map.into_iter().map(|(relation, entry)| Relation {
                id: None,
                relation,
                entry,
            }).collect()
        })
        .unwrap_or_default()
}

fn convert_anilist_characters(characters: &Option<CharacterConnection>) -> Vec<Character> {
    characters.as_ref()
        .and_then(|conn| conn.edges.as_ref())
        .map(|edges| {
            edges.iter().filter_map(|edge| {
                edge.node.as_ref().map(|node| {
                    let images = Images {
                        jpg: Image {
                            image_url: node.image.as_ref().and_then(|i| i.large.clone()).unwrap_or_default(),
                            small_image_url: node.image.as_ref().and_then(|i| i.medium.clone()).unwrap_or_default(),
                            large_image_url: node.image.as_ref().and_then(|i| i.large.clone()).unwrap_or_default(),
                        },
                        webp: Image {
                            image_url: String::new(),
                            small_image_url: String::new(),
                            large_image_url: String::new(),
                        },
                    };

                    let voice_actors = edge.voice_actors.iter().map(|va| {
                        let va_images = Images {
                            jpg: Image {
                                image_url: va.image.as_ref().and_then(|i| i.large.clone()).unwrap_or_default(),
                                small_image_url: va.image.as_ref().and_then(|i| i.medium.clone()).unwrap_or_default(),
                                large_image_url: va.image.as_ref().and_then(|i| i.large.clone()).unwrap_or_default(),
                            },
                            webp: Image {
                                image_url: String::new(),
                                small_image_url: String::new(),
                                large_image_url: String::new(),
                            },
                        };

                        VoiceActor {
                            person: VoiceActorInfo {
                                mal_id: va.id,
                                url: va.site_url.clone().unwrap_or_default(),
                                images: va_images,
                                name: va.name.as_ref().and_then(|n| n.full.clone()).unwrap_or_default(),
                            },
                            language: va.language.clone().unwrap_or_else(|| "Unknown".to_string()),
                        }
                    }).collect();

                    Character {
                        character: CharacterInfo {
                            mal_id: node.id,
                            url: node.site_url.clone().unwrap_or_default(),
                            images,
                            name: node.name.as_ref().and_then(|n| n.full.clone()).unwrap_or_default(),
                        },
                        role: edge.role.clone().unwrap_or_else(|| "BACKGROUND".to_string()),
                        voice_actors,
                    }
                })
            }).collect()
        })
        .unwrap_or_default()
}

fn convert_anilist_staff(staff: &Option<StaffConnection>) -> Vec<Staff> {
    staff.as_ref()
        .and_then(|conn| conn.edges.as_ref())
        .map(|edges| {
            edges.iter().filter_map(|edge| {
                edge.node.as_ref().map(|node| {
                    let images = Images {
                        jpg: Image {
                            image_url: node.image.as_ref().and_then(|i| i.large.clone()).unwrap_or_default(),
                            small_image_url: node.image.as_ref().and_then(|i| i.medium.clone()).unwrap_or_default(),
                            large_image_url: node.image.as_ref().and_then(|i| i.large.clone()).unwrap_or_default(),
                        },
                        webp: Image {
                            image_url: String::new(),
                            small_image_url: String::new(),
                            large_image_url: String::new(),
                        },
                    };

                    Staff {
                        person: StaffInfo {
                            mal_id: node.id,
                            url: node.site_url.clone().unwrap_or_default(),
                            images,
                            name: node.name.as_ref().and_then(|n| n.full.clone()).unwrap_or_default(),
                        },
                        positions: edge.role.as_ref().map(|r| vec![r.clone()]).unwrap_or_default(),
                    }
                })
            }).collect()
        })
        .unwrap_or_default()
}

fn strip_html(html: &str) -> String {
    // Simple HTML tag removal
    let re = regex::Regex::new(r"<[^>]*>").unwrap_or_else(|_| regex::Regex::new(r"").unwrap());
    re.replace_all(html, "").to_string()
}