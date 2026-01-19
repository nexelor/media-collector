use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn};

use crate::global::{
    database::DatabaseInstance,
    error::AppError,
    queue::{Task, TaskPriority, TaskData, TaskStatus},
};
use crate::anime::my_anime_list::{database::get_anime_by_id, model::Image};
use crate::picture::PictureFetcherModule;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchAnimePicturesPayload {
    pub anime_id: u32,
}

/// Task to fetch all pictures associated with an anime
pub struct FetchAnimePicturesTask {
    id: String,
    anime_id: u32,
    picture_module: Arc<PictureFetcherModule>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl FetchAnimePicturesTask {
    pub fn new(
        anime_id: u32,
        picture_module: Arc<PictureFetcherModule>,
    ) -> Self {
        let id = format!("fetch_anime_pictures_{}", anime_id);
        Self {
            id,
            anime_id,
            picture_module,
            created_at: chrono::Utc::now(),
        }
    }
    
    /// Queue an image download if URL is not empty
    async fn queue_image(
        &self,
        image: &Image,
        entity_id: i32,
        category: &str,
        entity_type: &str,
        sub_category: Option<&str>,
    ) -> Result<(), AppError> {
        let mut tags = vec![
            entity_type.to_string(),
            entity_id.to_string(),
            category.to_string(),
        ];
        
        if let Some(sub) = sub_category {
            tags.push(sub.to_string());
        }
        
        // Queue JPG version
        if !image.image_url.is_empty() {
            debug!(
                anime_id = self.anime_id,
                category = category,
                entity_type = entity_type,
                entity_id = entity_id,
                url = %image.image_url,
                "Queueing JPG image"
            );
            
            self.picture_module.queue_fetch_picture_for_entity(
                image.image_url.clone(),
                None,
                entity_type.to_string(),
                entity_id.to_string(),
                tags.clone(),
            ).await?;
        }
        
        // Queue large JPG version if different
        if !image.large_image_url.is_empty() && image.large_image_url != image.image_url {
            tags.push("large".to_string());
            
            debug!(
                anime_id = self.anime_id,
                category = category,
                url = %image.large_image_url,
                "Queueing large JPG image"
            );
            
            self.picture_module.queue_fetch_picture_for_entity(
                image.large_image_url.clone(),
                None,
                entity_type.to_string(),
                entity_id.to_string(),
                tags.clone(),
            ).await?;
        }
        
        // Queue small JPG version if different
        if !image.small_image_url.is_empty() 
            && image.small_image_url != image.image_url 
            && image.small_image_url != image.large_image_url {
            
            let mut small_tags = tags.clone();
            small_tags.push("small".to_string());
            
            debug!(
                anime_id = self.anime_id,
                category = category,
                url = %image.small_image_url,
                "Queueing small JPG image"
            );
            
            self.picture_module.queue_fetch_picture_for_entity(
                image.small_image_url.clone(),
                None,
                entity_type.to_string(),
                entity_id.to_string(),
                small_tags,
            ).await?;
        }
        
        Ok(())
    }
}

#[async_trait::async_trait]
impl Task for FetchAnimePicturesTask {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> &str {
        "fetch_anime_pictures"
    }

    fn priority(&self) -> TaskPriority {
        TaskPriority::Low
    }

    fn to_data(&self) -> TaskData {
        let payload = FetchAnimePicturesPayload {
            anime_id: self.anime_id,
        };

        TaskData {
            id: self.id.clone(),
            name: self.name().to_string(),
            priority: self.priority(),
            status: TaskStatus::Pending,
            created_at: self.created_at,
            payload: serde_json::json!(payload),
        }
    }

    async fn execute(
        &self,
        db: Arc<DatabaseInstance>,
        _client: reqwest::Client,
    ) -> Result<(), AppError> {
        info!(
            task = %self.name(),
            anime_id = self.anime_id,
            "Fetching all pictures for anime"
        );

        // Get anime data
        let anime = match get_anime_by_id(db.db(), self.anime_id as i32).await? {
            Some(a) => a,
            None => {
                warn!(
                    task = %self.name(),
                    anime_id = self.anime_id,
                    "Anime not found in database"
                );
                return Ok(());
            }
        };

        let mut total_queued = 0;

        // 1. Main images (JPG and WebP)
        debug!(anime_id = self.anime_id, "Queueing main anime images");
        
        // JPG images
        self.queue_image(&anime.images.jpg, self.anime_id as i32, "main", "anime", Some("jpg")).await?;
        total_queued += 1;
        
        // WebP images
        self.queue_image(&anime.images.webp, self.anime_id as i32, "main", "anime", Some("webp")).await?;
        total_queued += 1;

        // 2. Additional pictures
        info!(
            anime_id = self.anime_id,
            count = anime.pictures.len(),
            "Queueing additional pictures"
        );
        
        for (idx, picture) in anime.pictures.iter().enumerate() {
            // JPG
            self.queue_image(&picture.jpg, self.anime_id as i32, "picture", "anime", Some(&format!("jpg_{}", idx))).await?;
            // WebP
            self.queue_image(&picture.webp, self.anime_id as i32, "picture", "anime", Some(&format!("webp_{}", idx))).await?;
            total_queued += 2;
        }

        // 3. Character images
        info!(
            anime_id = self.anime_id,
            count = anime.characters.len(),
            "Queueing character images"
        );
        
        for character in &anime.characters {
            let character_tags_suffix = format!("character_{}", character.character.mal_id);
            
            // Character image (JPG and WebP)
            self.queue_image(
                &character.character.images.jpg,
                character.character.mal_id,
                "character",
                "character",
                Some(&format!("{}_jpg", character_tags_suffix))
            ).await?;
            
            self.queue_image(
                &character.character.images.webp,
                character.character.mal_id,
                "character",
                "character",
                Some(&format!("{}_webp", character_tags_suffix))
            ).await?;
            
            total_queued += 2;
            
            // Voice actor images
            for va in &character.voice_actors {
                let va_tags_suffix = format!("va_{}_{}", character.character.mal_id, va.person.mal_id);
                
                self.queue_image(
                    &va.person.images.jpg,
                    va.person.mal_id,
                    "voice_actor",
                    "voice_actor",
                    Some(&format!("{}_jpg", va_tags_suffix))
                ).await?;
                
                self.queue_image(
                    &va.person.images.webp,
                    va.person.mal_id,
                    "voice_actor",
                    "voice_actor",
                    Some(&format!("{}_webp", va_tags_suffix))
                ).await?;
                
                total_queued += 2;
            }
        }

        // 4. Staff images
        info!(
            anime_id = self.anime_id,
            count = anime.staffs.len(),
            "Queueing staff images"
        );
        
        for staff in &anime.staffs {
            let staff_tags_suffix = format!("staff_{}", staff.person.mal_id);
            
            // Staff image (JPG and WebP)
            self.queue_image(
                &staff.person.images.jpg,
                staff.person.mal_id,
                "staff",
                "staff",
                Some(&format!("{}_jpg", staff_tags_suffix))
            ).await?;
            
            self.queue_image(
                &staff.person.images.webp,
                staff.person.mal_id,
                "staff",
                "staff",
                Some(&format!("{}_webp", staff_tags_suffix))
            ).await?;
            
            total_queued += 2;
        }

        // 5. Video thumbnails (if videos exist)
        if let Some(videos) = &anime.videos {
            info!(
                anime_id = self.anime_id,
                promo_count = videos.promo.len(),
                episode_count = videos.episodes.len(),
                music_count = videos.music_videos.len(),
                "Queueing video thumbnails"
            );
            
            // Promo video images
            for (idx, promo) in videos.promo.iter().enumerate() {
                if let Some(images) = &promo.trailer.images {
                    // Tags format: ["anime", "123", "video_promo", "video_promo_jpg_0"]
                    let tags = vec![
                        "anime".to_string(),
                        self.anime_id.to_string(),
                        "video_promo".to_string(),
                        format!("video_promo_jpg_{}", idx),  // Used to extract video ID
                    ];
                    
                    self.queue_image_with_custom_tags(
                        &images.jpg,
                        self.anime_id as i32,
                        "anime",
                        tags.clone(),
                    ).await?;

                    // WebP version
                    let mut webp_tags = tags.clone();
                    webp_tags[3] = format!("video_promo_webp_{}", idx);
                    
                    self.queue_image_with_custom_tags(
                        &images.webp,
                        self.anime_id as i32,
                        "anime",
                        webp_tags,
                    ).await?;

                    total_queued += 2;
                }
            }
            
            // Episode video images - stored under anime/{id}/videos/{episode_mal_id}/
            for episode in &videos.episodes {
                let tags = vec![
                    "anime".to_string(),
                    self.anime_id.to_string(),
                    "video_episode".to_string(),
                    format!("video_episode_jpg_{}", episode.mal_id),  // MAL ID used for directory
                ];
                
                self.queue_image_with_custom_tags(
                    &episode.images.jpg,
                    self.anime_id as i32,
                    "anime",
                    tags.clone(),
                ).await?;
                
                // WebP version
                let mut webp_tags = tags.clone();
                webp_tags[3] = format!("video_episode_webp_{}", episode.mal_id);
                
                self.queue_image_with_custom_tags(
                    &episode.images.webp,
                    self.anime_id as i32,
                    "anime",
                    webp_tags,
                ).await?;
                
                total_queued += 2;
            }
            
            // Music video images - stored under anime/{id}/videos/{music_index}/
            for (idx, music) in videos.music_videos.iter().enumerate() {
                if let Some(images) = &music.video.images {
                    let tags = vec![
                        "anime".to_string(),
                        self.anime_id.to_string(),
                        "video_music".to_string(),
                        format!("video_music_jpg_{}", idx),
                    ];
                    
                    self.queue_image_with_custom_tags(
                        &images.jpg,
                        self.anime_id as i32,
                        "anime",
                        tags.clone(),
                    ).await?;
                    
                    // WebP version
                    let mut webp_tags = tags.clone();
                    webp_tags[3] = format!("video_music_webp_{}", idx);
                    
                    self.queue_image_with_custom_tags(
                        &images.webp,
                        self.anime_id as i32,
                        "anime",
                        webp_tags,
                    ).await?;
                    
                    total_queued += 2;
                }
            }
        }

        // 6. Recommendation images
        // info!(
        //     anime_id = self.anime_id,
        //     count = anime.recommendations.len(),
        //     "Queueing recommendation images"
        // );
        
        // for recommendation in &anime.recommendations {
        //     let rec_tags_suffix = format!("recommendation_{}", recommendation.entry.mal_id);
            
        //     self.queue_image(
        //         &recommendation.entry.images.jpg,
        //         "recommendation",
        //         Some(&format!("{}_jpg", rec_tags_suffix))
        //     ).await?;
            
        //     self.queue_image(
        //         &recommendation.entry.images.webp,
        //         "recommendation",
        //         Some(&format!("{}_webp", rec_tags_suffix))
        //     ).await?;
            
        //     total_queued += 2;
        // }

        info!(
            task = %self.name(),
            anime_id = self.anime_id,
            total_queued = total_queued,
            "All anime pictures queued successfully"
        );

        Ok(())
    }
}

impl FetchAnimePicturesTask {
    // Helper method to queue images with custom tags
    async fn queue_image_with_custom_tags(
        &self,
        image: &Image,
        entity_id: i32,
        entity_type: &str,
        tags: Vec<String>,
    ) -> Result<(), AppError> {
        if !image.image_url.is_empty() {
            self.picture_module.queue_fetch_picture_for_entity(
                image.image_url.clone(),
                None,
                entity_type.to_string(),
                entity_id.to_string(),
                tags,
            ).await?;
        }
        Ok(())
    }
}