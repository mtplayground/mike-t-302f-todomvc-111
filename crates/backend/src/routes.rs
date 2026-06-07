use crate::{
    error::{ApiResult, AppError, DataResponse},
    models::{Todo, TodoUpdate},
    state::AppState,
    todos,
};
use axum::{
    Json, Router,
    extract::{Path, State, rejection::JsonRejection},
    http::StatusCode,
    routing::{get, patch},
};
use serde::{Deserialize, Serialize};

pub fn todo_routes() -> Router<AppState> {
    Router::new()
        .route("/api/todos", get(list_todos).post(create_todo))
        .route("/api/todos/{id}", patch(update_todo).delete(delete_todo))
}

async fn list_todos(State(state): State<AppState>) -> ApiResult<Vec<Todo>> {
    let todos = todos::list(&state.db).await?;

    Ok(Json(DataResponse::new(todos)))
}

async fn create_todo(
    State(state): State<AppState>,
    payload: Result<Json<CreateTodoRequest>, JsonRejection>,
) -> Result<(StatusCode, Json<DataResponse<Todo>>), AppError> {
    let Json(payload) = payload.map_err(AppError::from)?;
    let todo = todos::create(&state.db, &payload.title).await?;

    Ok((StatusCode::CREATED, Json(DataResponse::new(todo))))
}

async fn update_todo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    payload: Result<Json<UpdateTodoRequest>, JsonRejection>,
) -> ApiResult<Todo> {
    let Json(payload) = payload.map_err(AppError::from)?;
    let update = payload.into_todo_update();
    let todo = todos::update(&state.db, id, update)
        .await?
        .ok_or_else(|| AppError::not_found(format!("todo {id} was not found")))?;

    Ok(Json(DataResponse::new(todo)))
}

async fn delete_todo(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> ApiResult<DeleteTodoResponse> {
    let deleted = todos::delete(&state.db, id).await?;

    if !deleted {
        return Err(AppError::not_found(format!("todo {id} was not found")));
    }

    Ok(Json(DataResponse::new(DeleteTodoResponse { deleted })))
}

#[derive(Clone, Debug, Deserialize)]
struct CreateTodoRequest {
    title: String,
}

#[derive(Clone, Debug, Deserialize)]
struct UpdateTodoRequest {
    title: Option<String>,
    completed: Option<bool>,
}

impl UpdateTodoRequest {
    fn into_todo_update(self) -> TodoUpdate {
        TodoUpdate {
            title: self.title,
            completed: self.completed,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct DeleteTodoResponse {
    deleted: bool,
}
