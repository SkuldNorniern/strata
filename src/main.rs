use axum::{routing::get, Router};
use tokio::net::TcpListener;

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

#[tokio::main]
async fn main() -> Result<(), WikiError> {
    let config = Config::new();
    
    // Validate directories exist
    if !config.base_dir.exists() {
        return Err(WikiError::NotFound);
    }

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
    println!("Wiki listening on http://{}", addr);
    
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await.map_err(WikiError::from)
}
