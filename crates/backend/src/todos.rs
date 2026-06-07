use crate::models::{Todo, TodoUpdate};
use sqlx::PgPool;
use std::{error::Error, fmt};

pub async fn list(pool: &PgPool) -> Result<Vec<Todo>, TodoRepositoryError> {
    sqlx::query_as::<_, Todo>(
        r#"
        SELECT id, title, completed, created_at
        FROM todos
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .map_err(TodoRepositoryError::Database)
}

pub async fn create(pool: &PgPool, title: &str) -> Result<Todo, TodoRepositoryError> {
    let title = normalize_title(title)?;

    sqlx::query_as::<_, Todo>(
        r#"
        INSERT INTO todos (title)
        VALUES ($1)
        RETURNING id, title, completed, created_at
        "#,
    )
    .bind(title)
    .fetch_one(pool)
    .await
    .map_err(TodoRepositoryError::Database)
}

pub async fn update(
    pool: &PgPool,
    id: i64,
    update: TodoUpdate,
) -> Result<Option<Todo>, TodoRepositoryError> {
    let title = update.title.map(|title| normalize_title(&title)).transpose()?;

    if title.is_none() && update.completed.is_none() {
        return Err(TodoRepositoryError::EmptyUpdate);
    }

    sqlx::query_as::<_, Todo>(
        r#"
        UPDATE todos
        SET
            title = COALESCE($2, title),
            completed = COALESCE($3, completed)
        WHERE id = $1
        RETURNING id, title, completed, created_at
        "#,
    )
    .bind(id)
    .bind(title.as_deref())
    .bind(update.completed)
    .fetch_optional(pool)
    .await
    .map_err(TodoRepositoryError::Database)
}

pub async fn delete(pool: &PgPool, id: i64) -> Result<bool, TodoRepositoryError> {
    sqlx::query(
        r#"
        DELETE FROM todos
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await
    .map(|result| result.rows_affected() > 0)
    .map_err(TodoRepositoryError::Database)
}

pub async fn clear_completed(pool: &PgPool) -> Result<u64, TodoRepositoryError> {
    sqlx::query(
        r#"
        DELETE FROM todos
        WHERE completed = TRUE
        "#,
    )
    .execute(pool)
    .await
    .map(|result| result.rows_affected())
    .map_err(TodoRepositoryError::Database)
}

pub async fn toggle_all(pool: &PgPool, completed: bool) -> Result<Vec<Todo>, TodoRepositoryError> {
    sqlx::query(
        r#"
        UPDATE todos
        SET completed = $1
        "#,
    )
    .bind(completed)
    .execute(pool)
    .await
    .map_err(TodoRepositoryError::Database)?;

    list(pool).await
}

#[derive(Debug)]
pub enum TodoRepositoryError {
    EmptyTitle,
    EmptyUpdate,
    Database(sqlx::Error),
}

impl fmt::Display for TodoRepositoryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTitle => write!(formatter, "todo title must not be empty"),
            Self::EmptyUpdate => write!(formatter, "todo update must include a title or completed value"),
            Self::Database(source) => write!(formatter, "todo repository query failed: {source}"),
        }
    }
}

impl Error for TodoRepositoryError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::EmptyTitle | Self::EmptyUpdate => None,
            Self::Database(source) => Some(source),
        }
    }
}

fn normalize_title(title: &str) -> Result<String, TodoRepositoryError> {
    let trimmed = title.trim();

    if trimmed.is_empty() {
        Err(TodoRepositoryError::EmptyTitle)
    } else {
        Ok(trimmed.to_owned())
    }
}
