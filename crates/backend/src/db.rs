use crate::config::Config;
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::{error::Error, fmt};

pub async fn init_pool(config: &Config) -> Result<PgPool, DbError> {
    let pool = PgPoolOptions::new()
        .max_connections(config.db_max_connections)
        .connect(&config.database_url)
        .await
        .map_err(DbError::Connect)?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(DbError::Migrate)?;

    Ok(pool)
}

#[derive(Debug)]
pub enum DbError {
    Connect(sqlx::Error),
    Migrate(sqlx::migrate::MigrateError),
}

impl fmt::Display for DbError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(source) => write!(formatter, "failed to connect to Postgres: {source}"),
            Self::Migrate(source) => write!(formatter, "failed to run database migrations: {source}"),
        }
    }
}

impl Error for DbError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Connect(source) => Some(source),
            Self::Migrate(source) => Some(source),
        }
    }
}
