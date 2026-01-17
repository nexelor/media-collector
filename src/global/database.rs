use mongodb::{Database, Collection, IndexModel};
use mongodb::options::{IndexOptions, UpdateOptions};
use mongodb::bson::{doc, Document};
use serde::{Serialize, Deserialize};
use tracing::{debug, info, warn};
use std::time::Duration;

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

        // Initialize indexes and collections
        db.initialize_global_collections().await?;

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

    /// Initialize global collections and indexes
    async fn initialize_global_collections(&self) -> Result<(), DatabaseError> {
        info!("Initializing global database collections");

        // Task queue collection
        self.create_task_queue_indexes().await?;
        
        // Module state collection
        self.create_module_state_indexes().await?;
        
        // API rate limit tracking collection
        self.create_rate_limit_indexes().await?;

        info!("Global collections initialized");
        Ok(())
    }

    async fn create_task_queue_indexes(&self) -> Result<(), DatabaseError> {
        let collection = self.db.collection::<Document>("task_queue");
        
        // Index on task ID (unique)
        let id_index = IndexModel::builder()
            .keys(doc! { "id": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        
        // Index on status for querying pending/failed tasks
        let status_index = IndexModel::builder()
            .keys(doc! { "status": 1 })
            .build();
        
        // Index on priority and created_at for queue ordering
        let priority_index = IndexModel::builder()
            .keys(doc! { "priority": -1, "created_at": 1 })
            .build();
        
        // Index on name for filtering by task type
        let name_index = IndexModel::builder()
            .keys(doc! { "name": 1 })
            .build();

        collection.create_indexes(vec![
            id_index,
            status_index,
            priority_index,
            name_index,
        ]).await
            .map_err(|e| DatabaseError::Query(format!("Failed to create task_queue indexes: {}", e)))?;

        debug!("Created indexes for task_queue collection");
        Ok(())
    }

    async fn create_module_state_indexes(&self) -> Result<(), DatabaseError> {
        let collection = self.db.collection::<Document>("module_state");
        
        // Index on module name (unique)
        let name_index = IndexModel::builder()
            .keys(doc! { "module_name": 1 })
            .options(IndexOptions::builder().unique(true).build())
            .build();
        
        // Index on last_active for monitoring
        let active_index = IndexModel::builder()
            .keys(doc! { "last_active": -1 })
            .build();

        collection.create_indexes(vec![name_index, active_index]).await
            .map_err(|e| DatabaseError::Query(format!("Failed to create module_state indexes: {}", e)))?;

        debug!("Created indexes for module_state collection");
        Ok(())
    }

    async fn create_rate_limit_indexes(&self) -> Result<(), DatabaseError> {
        let collection = self.db.collection::<Document>("rate_limits");
        
        // Compound index on limiter name and timestamp
        let limiter_index = IndexModel::builder()
            .keys(doc! { "limiter_name": 1, "timestamp": -1 })
            .build();
        
        // TTL index to auto-delete old entries after 1 hour
        let ttl_index = IndexModel::builder()
            .keys(doc! { "timestamp": 1 })
            .options(IndexOptions::builder()
                .expire_after(Duration::from_secs(3600))
                .build())
            .build();

        collection.create_indexes(vec![limiter_index, ttl_index]).await
            .map_err(|e| DatabaseError::Query(format!("Failed to create rate_limits indexes: {}", e)))?;

        debug!("Created indexes for rate_limits collection");
        Ok(())
    }

    // ========================================================================
    // Global Database Operations
    // ========================================================================

    /// Get a collection with proper type
    pub fn collection<T>(&self, name: &str) -> Collection<T>
    where
        T: Serialize + for<'de> Deserialize<'de> + Send + Sync,
    {
        self.db.collection(name)
    }

    /// Record module heartbeat
    pub async fn update_module_heartbeat(&self, module_name: &str) -> Result<(), DatabaseError> {
        #[derive(Serialize, Deserialize)]
        struct ModuleState {
            module_name: String,
            last_active: chrono::DateTime<chrono::Utc>,
            status: String,
        }

        let collection = self.collection::<ModuleState>("module_state");
        let filter = doc! { "module_name": module_name };
        let update = doc! {
            "$set": {
                "module_name": module_name,
                "last_active": mongodb::bson::DateTime::now(),
                "status": "active"
            }
        };
        let options = UpdateOptions::builder().upsert(true).build();

        collection.update_one(filter, update).with_options(options)
            .await
            .map_err(|e| DatabaseError::Query(format!("Failed to update heartbeat: {}", e)))?;

        Ok(())
    }

    /// Get all active modules
    pub async fn get_active_modules(&self) -> Result<Vec<String>, DatabaseError> {
        use futures::stream::StreamExt;

        #[derive(Serialize, Deserialize)]
        struct ModuleState {
            module_name: String,
        }

        let collection = self.collection::<ModuleState>("module_state");
        
        // Consider modules active if heartbeat within last 5 minutes
        let threshold = mongodb::bson::DateTime::now().timestamp_millis() - (5 * 60 * 1000);
        let filter = doc! {
            "last_active": { "$gte": threshold },
            "status": "active"
        };

        let mut cursor = collection.find(filter).await
            .map_err(|e| DatabaseError::Query(format!("Failed to query modules: {}", e)))?;

        let mut modules = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(state) => modules.push(state.module_name),
                Err(e) => warn!(error = %e, "Failed to deserialize module state"),
            }
        }

        Ok(modules)
    }

    /// Clean up old data (maintenance task)
    pub async fn cleanup_old_data(&self, days: i64) -> Result<(), DatabaseError> {
        let threshold = mongodb::bson::DateTime::now().timestamp_millis() - (days * 24 * 60 * 60 * 1000);

        // Clean up completed/failed tasks older than threshold
        let task_collection = self.collection::<Document>("task_queue");
        let task_filter = doc! {
            "status": { "$in": ["Completed", "Failed"] },
            "created_at": { "$lt": threshold }
        };
        
        let result = task_collection.delete_many(task_filter).await
            .map_err(|e| DatabaseError::Query(format!("Failed to cleanup tasks: {}", e)))?;

        info!(deleted = result.deleted_count, "Cleaned up old tasks");

        Ok(())
    }

    /// Get database statistics
    pub async fn get_stats(&self) -> Result<DatabaseStats, DatabaseError> {
        let task_queue = self.collection::<Document>("task_queue");
        
        let pending_count = task_queue.count_documents(doc! { "status": "Pending" }).await
            .map_err(|e| DatabaseError::Query(format!("Failed to count pending: {}", e)))?;
        
        let running_count = task_queue.count_documents(doc! { "status": "Running" }).await
            .map_err(|e| DatabaseError::Query(format!("Failed to count running: {}", e)))?;
        
        let completed_count = task_queue.count_documents(doc! { "status": "Completed" }).await
            .map_err(|e| DatabaseError::Query(format!("Failed to count completed: {}", e)))?;
        
        let failed_count = task_queue.count_documents(doc! { "status": "Failed" }).await
            .map_err(|e| DatabaseError::Query(format!("Failed to count failed: {}", e)))?;

        Ok(DatabaseStats {
            pending_tasks: pending_count,
            running_tasks: running_count,
            completed_tasks: completed_count,
            failed_tasks: failed_count,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseStats {
    pub pending_tasks: u64,
    pub running_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
}