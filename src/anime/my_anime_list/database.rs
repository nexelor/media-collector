use anyhow::Result;
use mongodb::Database;

use super::model::AnimeData;
use crate::{global::error::DatabaseError};

pub async fn insert_anime(db: &Database, data: &AnimeData) -> Result<(), DatabaseError> {
    let temp_col = db.collection::<AnimeData>("temp");
    let res = temp_col.insert_one(data).await?;

    Ok(())
}