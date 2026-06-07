use axum::{Json, Router, extract::State, routing::get};
use backend::{config::Config, db, error::DataResponse, routes, state::AppState};
use std::path::{Path, PathBuf};
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::from_env()?;
    let static_files = static_files(config.frontend_dist_dir.clone());
    let db = db::init_pool(&config).await?;
    let state = AppState::new(db);

    let app = Router::new()
        .route("/health", get(health))
        .merge(routes::todo_routes())
        .fallback_service(static_files)
        .with_state(state);
    let listener = TcpListener::bind(config.bind_address).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health(State(state): State<AppState>) -> Json<DataResponse<&'static str>> {
    let _db = state.db.clone();

    Json(DataResponse::new("ok"))
}

fn static_files(root: PathBuf) -> ServeDir<ServeFile> {
    ServeDir::new(&root).not_found_service(ServeFile::new(index_file(&root)))
}

fn index_file(root: &Path) -> PathBuf {
    root.join("index.html")
}
