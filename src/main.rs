mod types;
mod fs_utils;
mod render;
mod handlers;
mod nav;
mod templates;
mod utils;

use std::{path::PathBuf, sync::Arc};

use axum::{routing::get, Router};
use tokio::net::TcpListener;

use handlers::{handle_path, handle_root};
use types::{AppState, WikiError};

#[tokio::main]
async fn main() -> Result<(), WikiError> {
    let base_dir = PathBuf::from("wiki");
    let static_dir = PathBuf::from("static");
    if !base_dir.exists() {
        return Err(WikiError::NotFound);
    }

    let state = AppState { base_dir: Arc::new(base_dir), static_dir: Arc::new(static_dir) };

    let app = Router::new()
        .route("/", get(handle_root))
        .route("/search", get(handlers::handle_search))
        .route("/raw/*path", get(handlers::handle_raw))
        .route("/static/*path", get(handlers::handle_static))
        .route("/*path", get(handle_path))
        .with_state(state);

    let addr = std::net::SocketAddr::from(([0, 0, 0, 0], 5004));
    println!("Wiki listening on http://{}", addr);
    let listener = TcpListener::bind(addr).await?;
    axum::serve(listener, app).await.map_err(WikiError::from)
}
