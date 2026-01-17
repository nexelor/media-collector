use mongodb::Database;
use tracing::{debug, info};

use super::error::{AppError, DatabaseError};

#[derive(Clone)]
pub struct DatabaseInstance {
    db: Database,
}

impl DatabaseInstance {
    pub async fn new(host: &str, port: i32, db_name: &str) -> Result<Self, AppError> {
        let uri = format!("mongodb://{}:{}", host, port);

        info!(uri = %uri, database = %db_name, "Connecting to MongoDB");

        let db = Self::connect(&uri, db_name).await?;

        info!(database = %db_name, "Successfully connected to MongoDB");

        Ok(db)
    }

    async fn connect(uri: &str, db_name: &str) -> Result<Self, DatabaseError> {
        debug!(uri = %uri, "Creating MongoDB client");
        let client = mongodb::Client::with_uri_str(uri).await?;
        Ok(Self { db: client.database(db_name) })
    }

    pub fn db(&self) -> &Database {
        &self.db
    }
}