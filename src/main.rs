use std::sync::Arc;

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, debug, error, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::global::{config::AppConfig, database::DatabaseInstance, http::HttpClientManager, module::{ChildModule, ModuleHandle, ParentModule}};

mod anime;
mod global;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration first
    info!("Loading configuration from config.toml...");
    let config = match AppConfig::load() {
        Ok(cfg) => {
            info!("Configuration loaded successfully");
            Arc::new(cfg)
        }
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            eprintln!("Please ensure config.toml exists in the project root directory");
            return Err(e.into());
        }
    };
    
    // Initialize logging
    let log_level = config.app.log_level.clone();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| format!("media_collector={},info", log_level).into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting media-collector...");
    debug!(?config, "Loaded configuration");

    // Initialize database
    let db = DatabaseInstance::new(&config.database.host, config.database.port, &config.database.name).await?;
    let db = Arc::new(db);

    // Initialize HTTP client manager with rate limiters
    let http_manager = HttpClientManager::new(config.clone());
    
    // Store module handles for graceful shutdown
    let mut module_handles = Vec::new();

    // Launch anime module
    if config.is_parent_module_enabled("anime") {
        info!("Initializing anime module");
        let anime_module = anime::module::AnimeModule::new();
        let handle = spawn_parent_module(anime_module, db.clone()).await;
        module_handles.push(handle);
    } else {
        info!("Anime module is disabled in config");
    }

    if config.is_parent_module_enabled("manga") {
        info!("Manga module is enabled but not implemented yet");
        // let manga_module = manga::module::MangaModule::new();
        // let handle = spawn_parent_module(manga_module, db.clone()).await;
        // module_handles.push(handle);
    }

    if module_handles.is_empty() {
        warn!("No parent modules are enabled in configuration");
    } else {
        info!(count = module_handles.len(), "All enabled modules started successfully");
    }

    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    if anime::my_anime_list::module::MyAnimeListModule::is_available(&config) {
        info!("Testing MyAnimeList child module");
        let mal_client = http_manager.my_anime_list().clone();
        
        match anime::my_anime_list::module::MyAnimeListModule::new(mal_client.clone(), config.clone()) {
            Some(mal_module) => {
                let input = anime::my_anime_list::module::FetchAnimeInput { anime_id: 1 };
                
                match spawn_child_module(mal_module, db.clone(), mal_client.client.clone(), input).await {
                    Ok(anime_data) => info!(?anime_data, "Successfully fetched anime data"),
                    Err(e) => error!(error = %e, "Failed to fetch anime"),
                }
            }
            None => {
                error!("Failed to initialize MyAnimeList module - check configuration");
            }
        }
    } else {
        warn!("MyAnimeList child module is not available (disabled or missing required configuration)");
    }

    // Keep running until Ctrl+C
    info!("Application running, press Ctrl+C to shutdown");
    tokio::signal::ctrl_c().await?;
    warn!("Shutdown signal received, initiating graceful shutdown");

    // Graceful shutdown
    for handle in module_handles {
        info!(module = %handle.name, "Sending shutdown signal");
        if let Err(e) = handle.shutdown().await {
            error!(module = %handle.name, error = %e, "Failed to shutdown module");
        }
    }

    // Give modules time to clean up
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    info!("Shutdown complete");

    Ok(())
}

/// Spawn a parent module that runs continuously
async fn spawn_parent_module<M: ParentModule + 'static>(
    module: M,
    db: Arc<DatabaseInstance>,
) -> ModuleHandle {
    let (tx, rx) = mpsc::channel(100);
    let name = module.name().to_string();

    info!(module = %name, "Spawning parent module");

    tokio::spawn(async move {
        if let Err(e) = module.run(db, rx).await {
            error!(module = %module.name(), error = %e, "Module terminated with error");
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
    let module_name = module.name().to_string();
    info!(module = %module_name, "Spawning child module");

    let handle = tokio::spawn(async move {
        module.execute(db, client, input).await
    });

    handle.await
        .map_err(|e| {
            error!(module = %module_name, error = %e, "Child module join error");
            global::error::AppError::Module(format!("Join error: {}", e))
        })?
}