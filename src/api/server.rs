use std::net::SocketAddr;
use axum::Router;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tower_http::cors::{CorsLayer, Any};
use tracing::{info, error};

use crate::api::{routes, state::ApiState};

/// Start the API server
pub async fn start_api_server(
    state: ApiState,
    host: &str,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app(state);
    
    let addr = format!("{}:{}", host, port);
    let socket_addr: SocketAddr = addr.parse()?;
    
    info!(address = %addr, "Starting API server");
    
    let listener = TcpListener::bind(socket_addr).await?;
    
    info!(address = %addr, "API server listening");
    
    axum::serve(listener, app)
        .await
        .map_err(|e| {
            error!(error = %e, "API server error");
            e.into()
        })
}

/// Create the Axum application with middleware
fn create_app(state: ApiState) -> Router {
    let router = routes::create_router(state);
    
    router
        // Add CORS middleware
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
        )
        // Add tracing middleware
        .layer(TraceLayer::new_for_http())
}