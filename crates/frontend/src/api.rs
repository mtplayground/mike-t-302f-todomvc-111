use crate::types::{
    CreateTodoRequest, DataResponse, DeleteTodoResponse, ErrorResponse, Todo, ToggleAllRequest,
    UpdateTodoRequest,
};
use serde::de::DeserializeOwned;
use std::{error::Error, fmt};

const TODOS_PATH: &str = "/api/todos";

pub async fn list_todos() -> Result<Vec<Todo>, ApiError> {
    get_json(TODOS_PATH).await
}

pub async fn create_todo(title: impl Into<String>) -> Result<Todo, ApiError> {
    post_json(TODOS_PATH, &CreateTodoRequest { title: title.into() }).await
}

pub async fn update_todo(id: i64, update: UpdateTodoRequest) -> Result<Todo, ApiError> {
    patch_json(&format!("{TODOS_PATH}/{id}"), &update).await
}

pub async fn delete_todo(id: i64) -> Result<DeleteTodoResponse, ApiError> {
    delete_json(&format!("{TODOS_PATH}/{id}")).await
}

pub async fn toggle_all(completed: bool) -> Result<Vec<Todo>, ApiError> {
    post_json(
        &format!("{TODOS_PATH}/toggle-all"),
        &ToggleAllRequest { completed },
    )
    .await
}

pub async fn clear_completed() -> Result<Vec<Todo>, ApiError> {
    delete_json(&format!("{TODOS_PATH}/completed")).await
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApiError {
    Http {
        status: u16,
        code: String,
        message: String,
    },
    Network(String),
    Decode(String),
    UnsupportedTarget,
}

impl fmt::Display for ApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Http {
                status,
                code,
                message,
            } => {
                write!(formatter, "HTTP {status} {code}: {message}")
            }
            Self::Network(message) => write!(formatter, "network error: {message}"),
            Self::Decode(message) => write!(formatter, "response decode error: {message}"),
            Self::UnsupportedTarget => write!(formatter, "API client is only available in the browser"),
        }
    }
}

impl Error for ApiError {}

#[cfg(target_arch = "wasm32")]
async fn get_json<T>(path: &str) -> Result<T, ApiError>
where
    T: DeserializeOwned,
{
    use gloo_net::http::Request;

    decode_response(Request::get(path).send().await).await
}

#[cfg(not(target_arch = "wasm32"))]
async fn get_json<T>(_path: &str) -> Result<T, ApiError>
where
    T: DeserializeOwned,
{
    Err(ApiError::UnsupportedTarget)
}

#[cfg(target_arch = "wasm32")]
async fn post_json<T, B>(path: &str, body: &B) -> Result<T, ApiError>
where
    T: DeserializeOwned,
    B: serde::Serialize,
{
    use gloo_net::http::Request;

    let request = Request::post(path)
        .json(body)
        .map_err(|error| ApiError::Network(error.to_string()))?;
    decode_response(request.send().await).await
}

#[cfg(not(target_arch = "wasm32"))]
async fn post_json<T, B>(_path: &str, _body: &B) -> Result<T, ApiError>
where
    T: DeserializeOwned,
    B: serde::Serialize,
{
    Err(ApiError::UnsupportedTarget)
}

#[cfg(target_arch = "wasm32")]
async fn patch_json<T, B>(path: &str, body: &B) -> Result<T, ApiError>
where
    T: DeserializeOwned,
    B: serde::Serialize,
{
    use gloo_net::http::Request;

    let request = Request::patch(path)
        .json(body)
        .map_err(|error| ApiError::Network(error.to_string()))?;
    decode_response(request.send().await).await
}

#[cfg(not(target_arch = "wasm32"))]
async fn patch_json<T, B>(_path: &str, _body: &B) -> Result<T, ApiError>
where
    T: DeserializeOwned,
    B: serde::Serialize,
{
    Err(ApiError::UnsupportedTarget)
}

#[cfg(target_arch = "wasm32")]
async fn delete_json<T>(path: &str) -> Result<T, ApiError>
where
    T: DeserializeOwned,
{
    use gloo_net::http::Request;

    decode_response(Request::delete(path).send().await).await
}

#[cfg(not(target_arch = "wasm32"))]
async fn delete_json<T>(_path: &str) -> Result<T, ApiError>
where
    T: DeserializeOwned,
{
    Err(ApiError::UnsupportedTarget)
}

#[cfg(target_arch = "wasm32")]
async fn decode_response<T>(
    response: Result<gloo_net::http::Response, gloo_net::Error>,
) -> Result<T, ApiError>
where
    T: DeserializeOwned,
{
    let response = response.map_err(|error| ApiError::Network(error.to_string()))?;
    let status = response.status();

    if (200..300).contains(&status) {
        let body = response
            .json::<DataResponse<T>>()
            .await
            .map_err(|error| ApiError::Decode(error.to_string()))?;

        return Ok(body.data);
    }

    let text = response.text().await.unwrap_or_else(|error| error.to_string());
    let error = serde_json::from_str::<ErrorResponse>(&text).ok();
    let code = error
        .as_ref()
        .map(|error| error.error.code.clone())
        .unwrap_or_else(|| "http_error".to_owned());
    let message = error
        .map(|error| error.error.message)
        .filter(|message| !message.is_empty())
        .unwrap_or(text);

    Err(ApiError::Http {
        status,
        code,
        message,
    })
}
