use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc;

use crate::global::{database::DatabaseInstance, http::HttpClientManager, module::{ChildModule, ModuleHandle, ParentModule}};

mod anime;
mod global;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Starting media-collector...");

    // Initialize database
    let db = DatabaseInstance::new("localhost", 27017, "media_collector").await?;
    let db = Arc::new(db);

    // Initialize HTTP client manager with rate limiters
    let http_manager = HttpClientManager::new();
    
    // Store module handles for graceful shutdown
    let mut module_handles = Vec::new();

    // Launch anime module
    let anime_module = anime::module::AnimeModule::new();
    let handle = spawn_parent_module(anime_module, db.clone()).await;
    module_handles.push(handle);

    // Add more parent modules here as needed:
    // let manga_module = manga::module::MangaModule::new();
    // let handle = spawn_parent_module(manga_module, db.clone()).await;
    // module_handles.push(handle);

    println!("All modules started successfully");

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    let mal_client = http_manager.my_anime_list().clone(); 
    let mal_module = anime::my_anime_list::module::MyAnimeListModule::new(mal_client.clone());
    let input = anime::my_anime_list::module::FetchAnimeInput { anime_id: 1 };
    
    match spawn_child_module(mal_module, db.clone(), mal_client.client.clone(), input).await {
        Ok(anime_data) => println!("Got anime data: {:?}", anime_data),
        Err(e) => eprintln!("Error fetching anime: {}", e),
    }

    // Keep running until Ctrl+C
    tokio::signal::ctrl_c().await?;
    println!("\nShutdown signal received...");

    // Graceful shutdown
    for handle in module_handles {
        if let Err(e) = handle.shutdown().await {
            eprintln!("Error shutting down module {}: {}", handle.name, e);
        }
    }

    // Give modules time to clean up
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    println!("Shutdown complete");

    Ok(())
}

/// Spawn a parent module that runs continuously
async fn spawn_parent_module<M: ParentModule + 'static>(
    module: M,
    db: Arc<DatabaseInstance>,
) -> ModuleHandle {
    let (tx, rx) = mpsc::channel(100);
    let name = module.name().to_string();

    tokio::spawn(async move {
        if let Err(e) = module.run(db, rx).await {
            eprintln!("[{}] Module error: {}", module.name(), e);
        }
    });

    ModuleHandle { name, tx }
}

/// Spawn a child module for a single task
async fn spawn_child_module<M: ChildModule + 'static>(
    module: M,
    db: Arc<DatabaseInstance>,
    client: reqwest::Client,
    input: M::Input,
) -> Result<M::Output, global::error::AppError> {
    let handle = tokio::spawn(async move {
        module.execute(db, client, input).await
    });

    handle.await
        .map_err(|e| global::error::AppError::Module(format!("Join error: {}", e)))?
}