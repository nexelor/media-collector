use mongodb::Database;

use crate::error::{AppError, DatabaseError};

pub struct DatabaseInstance {
    client: Database,
}

impl DatabaseInstance {
    pub async fn new(host: &str, port: i32, db_name: &str) -> Result<Self, AppError> {
        let uri = format!("mongodb://{}:{}", host, port);
        let client = mongodb::Client::with_uri_str(&uri).await
            .map_err(|e| DatabaseError::ConnexionFailed(format!("MongoDB connection failed: {}", e)))?;

        let db = client.database(&db_name);

        println!("Connecting to MongoDB at {}/{}", uri, db_name);

        Ok(Self { client: db })
    }
}