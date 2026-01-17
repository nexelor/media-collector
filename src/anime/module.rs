// src/anime/module.rs
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;
use tokio::sync::mpsc;

use crate::global::database::DatabaseInstance;
use crate::global::error::AppError;
use crate::global::module::{ParentModule, ModuleMessage};

pub struct AnimeModule;

impl AnimeModule {
    pub fn new() -> Self {
        Self
    }
}

impl ParentModule for AnimeModule {
    fn name(&self) -> &str {
        "anime"
    }
    
    fn run(
        &self,
        db: Arc<DatabaseInstance>,
        mut rx: mpsc::Receiver<ModuleMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), AppError>> + Send + '_>> {
        Box::pin(async move {
            println!("[{}] Module started", self.name());
            
            loop {
                tokio::select! {
                    msg = rx.recv() => {
                        match msg {
                            Some(ModuleMessage::Shutdown) => {
                                println!("[{}] Shutting down", self.name());
                                break;
                            }
                            Some(ModuleMessage::Custom(data)) => {
                                println!("[{}] Received custom message: {}", self.name(), data);
                            }
                            None => {
                                println!("[{}] Channel closed", self.name());
                                break;
                            }
                        }
                    }
                    
                    // Periodic tasks
                    _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
                        println!("[{}] Heartbeat - running periodic tasks", self.name());
                        // Add periodic maintenance tasks here
                    }
                }
            }
            
            println!("[{}] Module stopped", self.name());
            Ok(())
        })
    }
}