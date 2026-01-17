use std::sync::Arc;
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::future::Future;
use tokio::sync::mpsc;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{info, debug, warn, error};

use crate::global::database::DatabaseInstance;
use crate::global::error::AppError;
use crate::global::module::{ParentModule, ModuleMessage};
use crate::global::queue::{QueueWorker, TaskQueue};

pub mod task;

#[derive(Clone)]
pub struct PictureFetcherModule {
    queue: TaskQueue,
    storage_path: PathBuf,
}

impl PictureFetcherModule {
    pub fn new(
        db: Arc<DatabaseInstance>, 
        client: reqwest::Client,
        storage_path: impl AsRef<Path>,
    ) -> Self {
        let storage_path = storage_path.as_ref().to_path_buf();
        
        // Create storage directory if it doesn't exist
        if let Err(e) = std::fs::create_dir_all(&storage_path) {
            warn!(error = %e, path = ?storage_path, "Failed to create picture storage directory");
        }
        
        let (queue, rx) = TaskQueue::new("picture_queue".to_string(), 1000);
        
        // Spawn the queue worker
        let worker = QueueWorker::new("picture_worker".to_string(), db, client);
        tokio::spawn(async move {
            if let Err(e) = worker.run(rx).await {
                error!(error = %e, "Picture queue worker error");
            }
        });

        Self { queue, storage_path }
    }

    pub fn queue(&self) -> &TaskQueue {
        &self.queue
    }

    pub fn storage_path(&self) -> &Path {
        &self.storage_path
    }

    /// Queue a task to fetch and store a picture
    pub async fn queue_fetch_picture(
        &self,
        url: String,
        filename: Option<String>,
    ) -> Result<(), AppError> {
        let task = task::FetchPictureTask::new(
            url,
            self.storage_path.clone(),
            filename,
        );
        
        self.queue.enqueue(Box::new(task)).await
    }

    /// Queue multiple pictures
    pub async fn queue_fetch_pictures(
        &self,
        urls: Vec<String>,
    ) -> Result<(), AppError> {
        for url in urls {
            self.queue_fetch_picture(url, None).await?;
        }
        Ok(())
    }
}

impl ParentModule for PictureFetcherModule {
    fn name(&self) -> &str {
        "picture_fetcher"
    }
    
    fn run(
        &self,
        _db: Arc<DatabaseInstance>,
        mut rx: mpsc::Receiver<ModuleMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + '_>> {
        Box::pin(async move {
            info!(
                module = %self.name(),
                storage_path = ?self.storage_path,
                "Picture fetcher module started"
            );
            
            loop {
                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Some(ModuleMessage::Shutdown) => {
                                info!(module = %self.name(), "Received shutdown signal");
                                if let Err(e) = self.queue.shutdown().await {
                                    warn!(module = %self.name(), error = %e, "Failed to shutdown queue");
                                }
                                break;
                            }
                            Some(ModuleMessage::Custom(data)) => {
                                debug!(module = %self.name(), message = %data, "Received custom message");
                            }
                            None => {
                                warn!(module = %self.name(), "Channel closed unexpectedly");
                                break;
                            }
                        }
                    }
                    
                    // Periodic cleanup of old/orphaned files
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(3600)) => {
                        debug!(module = %self.name(), "Running periodic cleanup");
                        // Add cleanup logic here if needed
                    }
                }
            }
            
            info!(module = %self.name(), "Picture fetcher module stopped");
            Ok(())
        })
    }
}