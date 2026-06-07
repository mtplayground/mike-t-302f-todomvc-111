mod config;

use axum::{Router, routing::get};
use config::Config;
use std::path::{Path, PathBuf};
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::from_env()?;
    let _database_url = config.database_url.as_str();
    let static_files = static_files(config.frontend_dist_dir.clone());

    let app = Router::new()
        .route("/health", get(health))
        .fallback_service(static_files);
    let listener = TcpListener::bind(config.bind_address).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}

fn static_files(root: PathBuf) -> ServeDir<ServeFile> {
    ServeDir::new(&root).not_found_service(ServeFile::new(index_file(&root)))
}

fn index_file(root: &Path) -> PathBuf {
    root.join("index.html")
}
