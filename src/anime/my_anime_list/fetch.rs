// https://myanimelist.net/apiconfig/references/api/v2#tag/anime
// async fn fetch_anime_mal(reqwest_client: Client, db_client: Arc<CDatabase>, id: &u32, mal_key: String) -> Result<FetchResult, AppError> {
//     let url = format!("https://api.myanimelist.net/v2/anime/{}?fields=id,title,main_picture,alternative_titles,start_date,end_date,synopsis,mean,rank,popularity,num_list_users,num_scoring_users,nsfw,genres,created_at,updated_at,media_type,status,num_episodes,start_season,broadcast,source,average_episode_duration,rating,studios,pictures,background,related_anime,related_manga,statistics", id);
//     Ok(FetchResult::AnimeMal(fetch_json(reqwest_client, db_client, url, id, Some(vec![("X-MAL-CLIENT-ID", mal_key.to_string())]), "anime", "mal").await?))
// }