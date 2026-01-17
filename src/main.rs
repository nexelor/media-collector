use std::{fs, sync::Arc};

use anyhow::Result;
use tokio::sync::mpsc;
use tracing::{info, debug, error, warn};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::{anime::module::AnimeModule, global::{config::AppConfig, database::DatabaseInstance, http::HttpClientManager, module::{ChildModule, ModuleHandle, ParentModule}}, picture::PictureFetcherModule};

mod anime;
mod global;
mod picture;
mod api;

#[tokio::main]
async fn main() -> Result<()> {
    // Load configuration first
    println!("Loading configuration from config.toml...");
    let config = match AppConfig::load() {
        Ok(cfg) => {
            println!("Configuration loaded successfully");
            Arc::new(cfg)
        }
        Err(e) => {
            eprintln!("Failed to load configuration: {}", e);
            eprintln!("Please ensure config.toml exists in the project root directory");
            return Err(e.into());
        }
    };
    
    // Initialize logging with configured settings
    setup_logging(&config)?;

    info!("Starting media-collector...");
    debug!(?config, "Loaded configuration");

    // Initialize database
    let db = DatabaseInstance::new(&config.database.host, config.database.port, &config.database.name).await?;
    let db = Arc::new(db);

    // Initialize child module collections
    if config.is_parent_module_enabled("anime") {
        if anime::my_anime_list::module::MyAnimeListModule::is_available(&config) {
            info!("Initializing MyAnimeList database collections");
            anime::my_anime_list::database::initialize_collections(db.db()).await?;
        }
    }

    // Spawn database maintenance task
    let db_clone = db.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(3600)).await; // Every hour

            info!("Running database maintenance");

            // Clean up old data (30 days)
            if let Err(e) = db_clone.cleanup_old_data(30).await {
                error!(error = %e, "Database cleanup failed");
            }

            // Get and log stats
            match db_clone.get_stats().await {
                Ok(stats) => {
                    info!(
                        pending = stats.pending_tasks,
                        running = stats.running_tasks,
                        completed = stats.completed_tasks,
                        failed = stats.failed_tasks,
                        "Database statistics"
                    );
                }
                Err(e) => error!(error = %e, "Failed to get database stats"),
            }
        }
    });

    // Initialize HTTP client manager with rate limiters
    let http_manager = HttpClientManager::new(config.clone());
    
    // Store module handles for graceful shutdown
    let mut module_handles = Vec::new();

    // Store module references for API
    let mut anime_module_ref: Option<Arc<AnimeModule>> = None;
    let mut picture_module_ref: Option<Arc<PictureFetcherModule>> = None;

    // Initialize picture fetcher module
    info!("Initializing picture fetcher module");
    let picture_storage_path = "./pictures";
    let picture_client = http_manager.default().client.clone();
    let picture_module = PictureFetcherModule::new(
        db.clone(),
        picture_client,
        picture_storage_path,
    );
    
    picture_module_ref = Some(Arc::new(picture_module.clone()));
    let picture_handle = spawn_parent_module(picture_module, db.clone()).await;
    module_handles.push(picture_handle);
    
    // Launch anime module
    if config.is_parent_module_enabled("anime") {
        info!("Initializing anime module");
        
        let mal_client = http_manager.my_anime_list().client.clone();
        let anime_module = AnimeModule::new(
            db.clone(), 
            mal_client,
        );
    
        anime_module_ref = Some(Arc::new(anime_module.clone()));
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

    // Initialize API state and server
    if config.api.enabled {
        info!("Initializing API server");
        
        let mut api_state = api::state::ApiState::new(
            config.clone(),
            db.clone(),
            Arc::new(http_manager.clone()),
        );
        
        // Add module references
        if let Some(ref anime_mod) = anime_module_ref {
            api_state = api_state.with_anime_module(anime_mod.clone());
        }
        
        if let Some(ref picture_mod) = picture_module_ref {
            api_state = api_state.with_picture_module(picture_mod.clone());
        }
        
        let api_host = config.api.host.clone();
        let api_port = config.api.port;
        
        // Spawn API server in background
        tokio::spawn(async move {
            if let Err(e) = api::start_api_server(api_state, &api_host, api_port).await {
                error!(error = %e, "API server failed");
            }
        });
        
        info!(
            host = %config.api.host,
            port = config.api.port,
            "API server started"
        );
    } else {
        info!("API server is disabled in config");
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
#[allow(dead_code)]
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

/// Setup logging based on configuration
fn setup_logging(config: &AppConfig) -> Result<()> {
    let log_level = &config.app.log_level;
    let logging_config = &config.app.logging;

    // Create the log directory if it doesn't exist
    if logging_config.log_to_file {
        fs::create_dir_all(&logging_config.log_directory)?;
        println!("Logs will be written to: {}/", logging_config.log_directory);
    }

    // Build the env filter
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| format!("media_collector={},info", log_level).into());

    // Build the subscriber with layers
    let registry = tracing_subscriber::registry().with(env_filter);

    if logging_config.log_to_file {
        // Determine rotation strategy
        let rotation = match logging_config.log_rotation {
            global::config::LogRotation::Daily => Rotation::DAILY,
            global::config::LogRotation::Hourly => Rotation::HOURLY,
            global::config::LogRotation::Never => Rotation::NEVER,
        };

        // Create file appender
        let file_appender = RollingFileAppender::new(
            rotation,
            &logging_config.log_directory,
            &format!("{}.log", logging_config.log_file_prefix),
        );

        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        
        // Create file layer
        let file_layer = tracing_subscriber::fmt::layer()
            .with_writer(non_blocking)
            .with_ansi(false)
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true);

        if logging_config.log_to_console {
            // Both console and file
            let console_layer = tracing_subscriber::fmt::layer()
                .with_writer(std::io::stdout)
                .with_target(false);

            registry
                .with(file_layer)
                .with(console_layer)
                .init();
        } else {
            // File only
            registry
                .with(file_layer)
                .init();
        }

        // Keep the guard alive for the duration of the program
        // This is a workaround - in production you'd want to store this properly
        std::mem::forget(_guard);
    } else if logging_config.log_to_console {
        // Console only
        let console_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout)
            .with_target(false);

        registry
            .with(console_layer)
            .init();
    } else {
        // No logging configured - use default console
        let console_layer = tracing_subscriber::fmt::layer()
            .with_writer(std::io::stdout)
            .with_target(false);

        registry
            .with(console_layer)
            .init();
    }

    Ok(())
}