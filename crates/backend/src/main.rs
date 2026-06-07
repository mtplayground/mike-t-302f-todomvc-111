use axum::{Json, Router, extract::State, routing::get};
use backend::{
    config::Config,
    db,
    error::{ApiResult, DataResponse},
    routes,
    state::AppState,
};
use std::path::{Path, PathBuf};
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::from_env()?;
    let static_files = ServeDir::new(&config.frontend_dist_dir).not_found_service(ServeFile::new(
        index_file(&config.frontend_dist_dir),
    ));
    let db = db::init_pool(&config).await?;
    let state = AppState::new(db);

    let app = Router::new()
        .route("/health", get(health))
        .merge(routes::todo_routes())
        .fallback_service(static_files)
        .with_state(state);
    let listener = TcpListener::bind(config.bind_address).await?;

    eprintln!("listening on {}", config.bind_address);
    eprintln!(
        "serving frontend assets from {}",
        config.frontend_dist_dir.display()
    );

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health(State(state): State<AppState>) -> ApiResult<&'static str> {
    sqlx::query("SELECT 1").execute(&state.db).await?;

    Ok(Json(DataResponse::new("ok")))
}

fn index_file(root: &Path) -> PathBuf {
    root.join("index.html")
}
