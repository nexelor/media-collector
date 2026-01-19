use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tracing::{info, debug, warn};

use crate::global::{
    database::DatabaseInstance,
    error::AppError,
    queue::{Task, TaskPriority, TaskData, TaskStatus},
};
use crate::anime::anilist::{database::get_anime_by_id, model::AniListAnimeData};
use crate::anime::my_anime_list::model::Image;
use crate::picture::PictureFetcherModule;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchAniListAnimePicturesPayload {
    pub anilist_id: u32,
}

/// Task to fetch all pictures associated with an AniList anime
pub struct FetchAniListAnimePicturesTask {
    id: String,
    anilist_id: u32,
    picture_module: Arc<PictureFetcherModule>,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl FetchAniListAnimePicturesTask {
    pub fn new(
        anilist_id: u32,
        picture_module: Arc<PictureFetcherModule>,
    ) -> Self {
        let id = format!("fetch_anilist_anime_pictures_{}", anilist_id);
        Self {
            id,
            anilist_id,
            picture_module,
            created_at: chrono::Utc::now(),
        }
    }
    
    /// Queue an image download if URL is not empty
    async fn queue_image(
        &self,
        image: &Image,
        category: &str,
        sub_category: Option<&str>,
    ) -> Result<(), AppError> {
        let mut tags = vec![
            "anime".to_string(),
            "anilist".to_string(),
            self.anilist_id.to_string(),
            category.to_string(),
        ];
        
        if let Some(sub) = sub_category {
            tags.push(sub.to_string());
        }
        
        // Queue JPG version
        if !image.image_url.is_empty() {
            debug!(
                anilist_id = self.anilist_id,
                category = category,
                url = %image.image_url,
                "Queueing JPG image"
            );
            
            self.picture_module.queue_fetch_picture_for_entity(
                image.image_url.clone(),
                None,
                "anime_anilist".to_string(),
                self.anilist_id.to_string(),
                tags.clone(),
            ).await?;
        }
        
        // Queue large JPG version if different
        if !image.large_image_url.is_empty() && image.large_image_url != image.image_url {
            let mut large_tags = tags.clone();
            large_tags.push("large".to_string());
            
            debug!(
                anilist_id = self.anilist_id,
                category = category,
                url = %image.large_image_url,
                "Queueing large JPG image"
            );
            
            self.picture_module.queue_fetch_picture_for_entity(
                image.large_image_url.clone(),
                None,
                "anime_anilist".to_string(),
                self.anilist_id.to_string(),
                large_tags,
            ).await?;
        }
        
        // Queue small JPG version if different
        if !image.small_image_url.is_empty() 
            && image.small_image_url != image.image_url 
            && image.small_image_url != image.large_image_url {
            
            let mut small_tags = tags.clone();
            small_tags.push("small".to_string());
            
            debug!(
                anilist_id = self.anilist_id,
                category = category,
                url = %image.small_image_url,
                "Queueing small JPG image"
            );
            
            self.picture_module.queue_fetch_picture_for_entity(
                image.small_image_url.clone(),
                None,
                "anime_anilist".to_string(),
                self.anilist_id.to_string(),
                small_tags,
            ).await?;
        }
        
        Ok(())
    }
    
    /// Queue a single URL with tags
    async fn queue_url(
        &self,
        url: &str,
        category: &str,
        sub_category: Option<&str>,
    ) -> Result<(), AppError> {
        if url.is_empty() {
            return Ok(());
        }
        
        let mut tags = vec![
            "anime".to_string(),
            "anilist".to_string(),
            self.anilist_id.to_string(),
            category.to_string(),
        ];
        
        if let Some(sub) = sub_category {
            tags.push(sub.to_string());
        }
        
        debug!(
            anilist_id = self.anilist_id,
            category = category,
            url = %url,
            "Queueing image"
        );
        
        self.picture_module.queue_fetch_picture_for_entity(
            url.to_string(),
            None,
            "anime_anilist".to_string(),
            self.anilist_id.to_string(),
            tags,
        ).await?;
        
        Ok(())
    }
}

#[async_trait::async_trait]
impl Task for FetchAniListAnimePicturesTask {
    fn id(&self) -> String {
        self.id.clone()
    }

    fn name(&self) -> &str {
        "fetch_anilist_anime_pictures"
    }

    fn priority(&self) -> TaskPriority {
        TaskPriority::Low
    }

    fn to_data(&self) -> TaskData {
        let payload = FetchAniListAnimePicturesPayload {
            anilist_id: self.anilist_id,
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
            anilist_id = self.anilist_id,
            "Fetching all pictures for AniList anime"
        );

        // Get anime data
        let anime = match get_anime_by_id(db.db(), self.anilist_id as i32).await? {
            Some(a) => a,
            None => {
                warn!(
                    task = %self.name(),
                    anilist_id = self.anilist_id,
                    "Anime not found in database"
                );
                return Ok(());
            }
        };

        let mut total_queued = 0;

        // 1. Main cover images
        debug!(anilist_id = self.anilist_id, "Queueing main anime images");
        
        self.queue_image(&anime.images.jpg, "cover", Some("jpg")).await?;
        self.queue_image(&anime.images.webp, "cover", Some("webp")).await?;
        total_queued += 2;

        // 2. Banner image (AniList specific)
        if let Some(banner) = &anime.banner_image {
            info!(
                anilist_id = self.anilist_id,
                "Queueing banner image"
            );
            
            self.queue_url(banner, "banner", None).await?;
            total_queued += 1;
        }

        // 3. Character images
        info!(
            anilist_id = self.anilist_id,
            count = anime.characters.len(),
            "Queueing character images"
        );
        
        for character in &anime.characters {
            let character_tags_suffix = format!("character_{}", character.character.mal_id);
            
            // Character image (JPG and WebP)
            self.queue_image(
                &character.character.images.jpg,
                "character",
                Some(&format!("{}_jpg", character_tags_suffix))
            ).await?;
            
            self.queue_image(
                &character.character.images.webp,
                "character",
                Some(&format!("{}_webp", character_tags_suffix))
            ).await?;
            
            total_queued += 2;
            
            // Voice actor images
            for va in &character.voice_actors {
                let va_tags_suffix = format!("va_{}_{}", character.character.mal_id, va.person.mal_id);
                
                self.queue_image(
                    &va.person.images.jpg,
                    "voice_actor",
                    Some(&format!("{}_jpg", va_tags_suffix))
                ).await?;
                
                self.queue_image(
                    &va.person.images.webp,
                    "voice_actor",
                    Some(&format!("{}_webp", va_tags_suffix))
                ).await?;
                
                total_queued += 2;
            }
        }

        // 4. Staff images
        info!(
            anilist_id = self.anilist_id,
            count = anime.staffs.len(),
            "Queueing staff images"
        );
        
        for staff in &anime.staffs {
            let staff_tags_suffix = format!("staff_{}", staff.person.mal_id);
            
            // Staff image (JPG and WebP)
            self.queue_image(
                &staff.person.images.jpg,
                "staff",
                Some(&format!("{}_jpg", staff_tags_suffix))
            ).await?;
            
            self.queue_image(
                &staff.person.images.webp,
                "staff",
                Some(&format!("{}_webp", staff_tags_suffix))
            ).await?;
            
            total_queued += 2;
        }

        info!(
            task = %self.name(),
            anilist_id = self.anilist_id,
            total_queued = total_queued,
            "All AniList anime pictures queued successfully"
        );

        Ok(())
    }
}