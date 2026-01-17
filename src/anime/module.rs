use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use tokio::sync::mpsc;
use tracing::{info, debug, warn};

use crate::global::database::DatabaseInstance;
use crate::global::error::AppError;
use crate::global::module::{ParentModule, ModuleMessage};
use crate::global::queue::{QueueWorker, TaskQueue};

pub struct AnimeModule {
    queue: TaskQueue,
}

impl AnimeModule {
    pub fn new(db: Arc<DatabaseInstance>, client: reqwest::Client) -> Self {
        let (queue, rx) = TaskQueue::new("anime_queue".to_string(), 1000);
        
        // Spawn the queue worker
        let worker = QueueWorker::new("anime_worker".to_string(), db, client);
        tokio::spawn(async move {
            if let Err(e) = worker.run(rx).await {
                tracing::error!(error = %e, "Queue worker error");
            }
        });

        Self { queue }
    }

    pub fn queue(&self) -> &TaskQueue {
        &self.queue
    }
}

impl ParentModule for AnimeModule {
    fn name(&self) -> &str {
        "anime"
    }
    
    fn run(
        &self,
        _db: Arc<DatabaseInstance>,
        mut rx: mpsc::Receiver<ModuleMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + '_>> {
        Box::pin(async move {
            info!(module = %self.name(), "Module started");
            
            loop {
                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Some(ModuleMessage::Shutdown) => {
                                info!(module = %self.name(), "Received shutdown signal");
                                // Shutdown the queue
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
                    
                    // Periodic tasks
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                        debug!(module = %self.name(), "Running periodic maintenance tasks");
                        // Add periodic maintenance tasks here
                    }
                }
            }
            
            info!(module = %self.name(), "Module stopped");
            Ok(())
        })
    }
}