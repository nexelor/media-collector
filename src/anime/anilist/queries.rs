/// Query to fetch anime by MAL ID
pub const ANIME_BY_MAL_ID_QUERY: &str = r#"
query ($malId: Int) {
  Media(idMal: $malId, type: ANIME) {
    id
    idMal
    title {
      romaji
      english
      native
      userPreferred
    }
    type
    format
    status
    description
    startDate {
      year
      month
      day
    }
    endDate {
      year
      month
      day
    }
    season
    seasonYear
    episodes
    duration
    countryOfOrigin
    isLicensed
    source
    hashtag
    trailer {
      id
      site
      thumbnail
    }
    updatedAt
    coverImage {
      extraLarge
      large
      medium
      color
    }
    bannerImage
    genres
    synonyms
    averageScore
    meanScore
    popularity
    isLocked
    trending
    favourites
    tags {
      id
      name
      description
      category
      rank
      isGeneralSpoiler
      isMediaSpoiler
      isAdult
    }
    relations {
      edges {
        id
        relationType
        node {
          id
          idMal
          title {
            romaji
            english
            native
          }
          type
          format
          coverImage {
            large
            medium
          }
        }
      }
    }
    characters(sort: ROLE) {
      edges {
        id
        role
        node {
          id
          name {
            first
            middle
            last
            full
            native
            alternative
          }
          image {
            large
            medium
          }
          siteUrl
        }
        voiceActors(language: JAPANESE) {
          id
          name {
            first
            middle
            last
            full
            native
          }
          language
          image {
            large
            medium
          }
          siteUrl
        }
      }
    }
    staff {
      edges {
        id
        role
        node {
          id
          name {
            first
            middle
            last
            full
            native
          }
          image {
            large
            medium
          }
          siteUrl
        }
      }
    }
    studios {
      edges {
        isMain
        node {
          id
          name
          isAnimationStudio
          siteUrl
        }
      }
    }
    isFavourite
    isFavouriteBlocked
    isAdult
    nextAiringEpisode {
      airingAt
      timeUntilAiring
      episode
    }
    externalLinks {
      id
      url
      site
    }
    streamingEpisodes {
      title
      thumbnail
      url
      site
    }
    rankings {
      id
      rank
      type
      format
      year
      season
      allTime
      context
    }
    stats {
      scoreDistribution {
        score
        amount
      }
      statusDistribution {
        status
        amount
      }
    }
    siteUrl
  }
}
"#;

/// Query to fetch anime by AniList ID
pub const ANIME_BY_ID_QUERY: &str = r#"
query ($id: Int) {
  Media(id: $id, type: ANIME) {
    id
    idMal
    title {
      romaji
      english
      native
      userPreferred
    }
    type
    format
    status
    description
    startDate {
      year
      month
      day
    }
    endDate {
      year
      month
      day
    }
    season
    seasonYear
    episodes
    duration
    countryOfOrigin
    isLicensed
    source
    hashtag
    trailer {
      id
      site
      thumbnail
    }
    updatedAt
    coverImage {
      extraLarge
      large
      medium
      color
    }
    bannerImage
    genres
    synonyms
    averageScore
    meanScore
    popularity
    isLocked
    trending
    favourites
    tags {
      id
      name
      description
      category
      rank
      isGeneralSpoiler
      isMediaSpoiler
      isAdult
    }
    relations {
      edges {
        id
        relationType
        node {
          id
          idMal
          title {
            romaji
            english
            native
          }
          type
          format
          coverImage {
            large
            medium
          }
        }
      }
    }
    characters(sort: ROLE) {
      edges {
        id
        role
        node {
          id
          name {
            first
            middle
            last
            full
            native
            alternative
          }
          image {
            large
            medium
          }
          siteUrl
        }
        voiceActors(language: JAPANESE) {
          id
          name {
            first
            middle
            last
            full
            native
          }
          language
          image {
            large
            medium
          }
          siteUrl
        }
      }
    }
    staff {
      edges {
        id
        role
        node {
          id
          name {
            first
            middle
            last
            full
            native
          }
          image {
            large
            medium
          }
          siteUrl
        }
      }
    }
    studios {
      edges {
        isMain
        node {
          id
          name
          isAnimationStudio
          siteUrl
        }
      }
    }
    isFavourite
    isFavouriteBlocked
    isAdult
    nextAiringEpisode {
      airingAt
      timeUntilAiring
      episode
    }
    externalLinks {
      id
      url
      site
    }
    streamingEpisodes {
      title
      thumbnail
      url
      site
    }
    rankings {
      id
      rank
      type
      format
      year
      season
      allTime
      context
    }
    stats {
      scoreDistribution {
        score
        amount
      }
      statusDistribution {
        status
        amount
      }
    }
    siteUrl
  }
}
"#;

/// Query to search for anime
pub const SEARCH_ANIME_QUERY: &str = r#"
query ($search: String, $page: Int, $perPage: Int) {
  Page(page: $page, perPage: $perPage) {
    pageInfo {
      total
      perPage
      currentPage
      lastPage
      hasNextPage
    }
    media(search: $search, type: ANIME, sort: POPULARITY_DESC) {
      id
      idMal
      title {
        romaji
        english
        native
        userPreferred
      }
      type
      format
      status
      description(asHtml: false)
      startDate {
        year
        month
        day
      }
      endDate {
        year
        month
        day
      }
      season
      seasonYear
      episodes
      duration
      source
      coverImage {
        extraLarge
        large
        medium
        color
      }
      bannerImage
      genres
      averageScore
      meanScore
      popularity
      favourites
      isAdult
      siteUrl
    }
  }
}
"#;

/// Query to get trending anime
pub const TRENDING_ANIME_QUERY: &str = r#"
query ($page: Int, $perPage: Int) {
  Page(page: $page, perPage: $perPage) {
    pageInfo {
      total
      perPage
      currentPage
      lastPage
      hasNextPage
    }
    media(type: ANIME, sort: TRENDING_DESC) {
      id
      idMal
      title {
        romaji
        english
        native
      }
      coverImage {
        large
        medium
      }
      averageScore
      popularity
      trending
      siteUrl
    }
  }
}
"#;

/// Query to get anime by season
pub const ANIME_BY_SEASON_QUERY: &str = r#"
query ($season: MediaSeason, $year: Int, $page: Int, $perPage: Int) {
  Page(page: $page, perPage: $perPage) {
    pageInfo {
      total
      perPage
      currentPage
      lastPage
      hasNextPage
    }
    media(season: $season, seasonYear: $year, type: ANIME, sort: POPULARITY_DESC) {
      id
      idMal
      title {
        romaji
        english
        native
      }
      format
      status
      episodes
      coverImage {
        large
        medium
      }
      averageScore
      popularity
      siteUrl
    }
  }
}
"#;