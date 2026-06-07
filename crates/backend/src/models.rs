use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;

#[derive(Clone, Debug, Eq, FromRow, PartialEq, Serialize)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct TodoUpdate {
    pub title: Option<String>,
    pub completed: Option<bool>,
}

impl TodoUpdate {
    pub fn title(title: impl Into<String>) -> Self {
        Self {
            title: Some(title.into()),
            completed: None,
        }
    }

    pub fn completed(completed: bool) -> Self {
        Self {
            title: None,
            completed: Some(completed),
        }
    }

    pub fn title_and_completed(title: impl Into<String>, completed: bool) -> Self {
        Self {
            title: Some(title.into()),
            completed: Some(completed),
        }
    }
}
