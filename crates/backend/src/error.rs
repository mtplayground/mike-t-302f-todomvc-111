use crate::todos::TodoRepositoryError;
use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use std::{error::Error, fmt};

pub type ApiResult<T> = Result<Json<DataResponse<T>>, AppError>;

#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Conflict(String),
    Database(sqlx::Error),
    Internal(String),
}

impl AppError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::NotFound(message.into())
    }

    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict(message.into())
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }

    pub fn status(&self) -> StatusCode {
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Database(_) | Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "bad_request",
            Self::NotFound(_) => "not_found",
            Self::Conflict(_) => "conflict",
            Self::Database(_) => "database_error",
            Self::Internal(_) => "internal_error",
        }
    }

    pub fn client_message(&self) -> String {
        match self {
            Self::BadRequest(message)
            | Self::NotFound(message)
            | Self::Conflict(message)
            | Self::Internal(message) => message.clone(),
            Self::Database(_) => "database error".to_owned(),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BadRequest(message) => write!(formatter, "bad request: {message}"),
            Self::NotFound(message) => write!(formatter, "not found: {message}"),
            Self::Conflict(message) => write!(formatter, "conflict: {message}"),
            Self::Database(source) => write!(formatter, "database error: {source}"),
            Self::Internal(message) => write!(formatter, "internal error: {message}"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(source) => Some(source),
            Self::BadRequest(_) | Self::NotFound(_) | Self::Conflict(_) | Self::Internal(_) => None,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status();

        if status.is_server_error() {
            log_server_error(status, &self);
        }

        let body = ErrorResponse::new(self.code(), self.client_message());
        (status, Json(body)).into_response()
    }
}

fn log_server_error(status: StatusCode, error: &AppError) {
    eprintln!(
        "server_error status={} code={} error={error}",
        status.as_u16(),
        error.code()
    );
    eprintln!("server_error debug={error:?}");

    let mut source = error.source();
    while let Some(error) = source {
        eprintln!("server_error source={error}");
        source = error.source();
    }
}

impl From<sqlx::Error> for AppError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

impl From<TodoRepositoryError> for AppError {
    fn from(error: TodoRepositoryError) -> Self {
        match error {
            TodoRepositoryError::EmptyTitle => Self::BadRequest("todo title must not be empty".to_owned()),
            TodoRepositoryError::EmptyUpdate => {
                Self::BadRequest("todo update must include a title or completed value".to_owned())
            }
            TodoRepositoryError::Database(source) => Self::Database(source),
        }
    }
}

impl From<JsonRejection> for AppError {
    fn from(error: JsonRejection) -> Self {
        Self::BadRequest(error.body_text())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DataResponse<T> {
    pub data: T,
}

impl<T> DataResponse<T> {
    pub fn new(data: T) -> Self {
        Self { data }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

impl ErrorResponse {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            error: ErrorDetails {
                code: code.into(),
                message: message.into(),
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
}
