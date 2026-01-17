use mongodb::Database;

use super::error::{AppError, DatabaseError};

#[derive(Clone)]
pub struct DatabaseInstance {
    db: Database,
}

impl DatabaseInstance {
    pub async fn new(host: &str, port: i32, db_name: &str) -> Result<Self, AppError> {
        let uri = format!("mongodb://{}:{}", host, port);

        println!("Connecting to MongoDB at {}/{}", uri, db_name);

        let db = Self::connect(&uri, db_name).await?;

        Ok(db)
    }

    async fn connect(uri: &str, db_name: &str) -> Result<Self, DatabaseError> {
        let client = mongodb::Client::with_uri_str(uri).await?;
        Ok(Self { db: client.database(db_name) })
    }

    pub fn db(&self) -> &Database {
        &self.db
    }
}