use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct Todo {
    pub id: i64,
    pub title: String,
    pub completed: bool,
    pub created_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct CreateTodoRequest {
    pub title: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize)]
pub struct UpdateTodoRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<bool>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ToggleAllRequest {
    pub completed: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct DeleteTodoResponse {
    pub deleted: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct DataResponse<T> {
    pub data: T,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
}
