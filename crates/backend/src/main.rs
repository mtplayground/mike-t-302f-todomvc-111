mod config;

use axum::{Router, routing::get};
use config::Config;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = Config::from_env()?;
    let _database_url = config.database_url.as_str();

    let app = Router::new().route("/health", get(health));
    let listener = TcpListener::bind(config.bind_address).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health() -> &'static str {
    "ok"
}
