use axum::{routing::get, Router};
use tokio::net::TcpListener;
use log::{info, error};

use crate::config::Config;
use crate::errors::WikiError;
use crate::types::AppState;
use crate::handlers::{handle_path, handle_root, handle_search, handle_raw, handle_static};

mod components;
mod config;
mod errors;
mod handlers;
mod services;
mod types;
mod utils;
mod logger;

#[tokio::main]
async fn main() -> Result<(), WikiError> {
    // Initialize logging first
    if let Err(e) = logger::Logger::init() {
        eprintln!("Failed to initialize logger: {}", e);
        // Continue without logging if it fails
    }
    
    info!("Starting Strata Wiki server...");
    
    let config = Config::new();
    info!("Configuration loaded successfully");
    
    // Validate directories exist
    if !config.base_dir.exists() {
        error!("Base directory does not exist: {:?}", config.base_dir);
        return Err(WikiError::NotFound);
    }
    
    info!("Base directory validated: {:?}", config.base_dir);

    let state = AppState { 
        base_dir: config.base_dir.clone(), 
        static_dir: config.static_dir.clone() 
    };

    let app = Router::new()
        .route("/", get(handle_root))
        .route("/search", get(handle_search))
        .route("/raw/*path", get(handle_raw))
        .route("/static/*path", get(handle_static))
        .route("/*path", get(handle_path))
        .with_state(state);

    let addr = config.socket_addr();
    info!("Wiki server starting on http://{}", addr);
    
    let listener = TcpListener::bind(addr).await?;
    info!("Server listening successfully on {}", addr);
    
    axum::serve(listener, app).await.map_err(|e| {
        error!("Server error: {}", e);
        WikiError::from(e)
    })
}
