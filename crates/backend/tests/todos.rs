use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Method, Request, StatusCode},
};
use backend::{
    models::TodoUpdate,
    routes,
    state::AppState,
    todos::{self, TodoRepositoryError},
};
use serde_json::{Value, json};
use sqlx::{Executor, PgPool, postgres::PgPoolOptions};
use std::{
    error::Error,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};
use tower::ServiceExt;

static NEXT_SCHEMA: AtomicU64 = AtomicU64::new(1);

struct TestDb {
    admin: PgPool,
    pool: PgPool,
    schema: String,
}

impl TestDb {
    async fn connect() -> Result<Self, Box<dyn Error>> {
        let database_url = std::env::var("DATABASE_URL")?;
        let suffix = NEXT_SCHEMA.fetch_add(1, Ordering::Relaxed);
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let schema = format!("test_todos_{timestamp}_{suffix}");
        let admin = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await?;

        let create_schema = format!(r#"CREATE SCHEMA "{schema}""#);
        admin.execute(create_schema.as_str()).await?;

        let schema_for_connection = schema.clone();
        let pool = PgPoolOptions::new()
            .max_connections(2)
            .after_connect(move |connection, _metadata| {
                let schema = schema_for_connection.clone();
                Box::pin(async move {
                    let set_search_path = format!(r#"SET search_path TO "{schema}""#);
                    connection.execute(set_search_path.as_str()).await?;
                    Ok(())
                })
            })
            .connect(&database_url)
            .await?;

        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self {
            admin,
            pool,
            schema,
        })
    }

    fn app(&self) -> Router {
        Router::new()
            .merge(routes::todo_routes())
            .with_state(AppState::new(self.pool.clone()))
    }

    async fn cleanup(self) -> Result<(), Box<dyn Error>> {
        self.pool.close().await;
        let drop_schema = format!(r#"DROP SCHEMA IF EXISTS "{}" CASCADE"#, self.schema);
        self.admin
            .execute(drop_schema.as_str())
            .await?;
        self.admin.close().await;
        Ok(())
    }
}

#[tokio::test]
async fn repository_crud_validation_and_bulk_operations() -> Result<(), Box<dyn Error>> {
    let db = TestDb::connect().await?;

    assert!(todos::list(&db.pool).await?.is_empty());

    let first = todos::create(&db.pool, "  first  ").await?;
    assert_eq!(first.title, "first");
    assert!(!first.completed);

    let second = todos::create(&db.pool, "second").await?;
    assert_eq!(todos::list(&db.pool).await?.len(), 2);

    let empty_create = todos::create(&db.pool, "   ").await;
    assert!(matches!(empty_create, Err(TodoRepositoryError::EmptyTitle)));

    let updated = todos::update(
        &db.pool,
        first.id,
        TodoUpdate::title_and_completed("  renamed  ", true),
    )
    .await?
    .ok_or_else(|| invalid_data("updated todo was not returned"))?;
    assert_eq!(updated.title, "renamed");
    assert!(updated.completed);

    let empty_update = todos::update(&db.pool, first.id, TodoUpdate::default()).await;
    assert!(matches!(empty_update, Err(TodoRepositoryError::EmptyUpdate)));

    let missing_update = todos::update(&db.pool, 999_999, TodoUpdate::completed(true)).await?;
    assert!(missing_update.is_none());

    let toggled = todos::toggle_all(&db.pool, true).await?;
    assert_eq!(toggled.len(), 2);
    assert!(toggled.iter().all(|todo| todo.completed));

    let deleted_completed = todos::clear_completed(&db.pool).await?;
    assert_eq!(deleted_completed, 2);
    assert!(todos::list(&db.pool).await?.is_empty());

    let delete_missing = todos::delete(&db.pool, second.id).await?;
    assert!(!delete_missing);

    db.cleanup().await
}

#[tokio::test]
async fn api_crud_validation_and_bulk_operations() -> Result<(), Box<dyn Error>> {
    let db = TestDb::connect().await?;
    let app = db.app();

    let (status, body) = send(app.clone(), Method::GET, "/api/todos", None).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().map(Vec::len), Some(0));

    let (status, body) = send(
        app.clone(),
        Method::POST,
        "/api/todos",
        Some(json!({ "title": "   " })),
    )
    .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "bad_request");

    let (status, body) = send(
        app.clone(),
        Method::POST,
        "/api/todos",
        Some(json!({ "title": " first " })),
    )
    .await?;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["data"]["title"], "first");
    let first_id = body["data"]["id"]
        .as_i64()
        .ok_or_else(|| invalid_data("created todo missing id"))?;

    let (status, body) = send(
        app.clone(),
        Method::POST,
        "/api/todos",
        Some(json!({ "title": "second" })),
    )
    .await?;
    assert_eq!(status, StatusCode::CREATED);
    let second_id = body["data"]["id"]
        .as_i64()
        .ok_or_else(|| invalid_data("second created todo missing id"))?;

    let (status, body) = send(
        app.clone(),
        Method::PATCH,
        &format!("/api/todos/{first_id}"),
        Some(json!({ "completed": true })),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["completed"], true);

    let (status, body) = send(
        app.clone(),
        Method::PATCH,
        &format!("/api/todos/{first_id}"),
        Some(json!({ "title": "   " })),
    )
    .await?;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"]["code"], "bad_request");

    let (status, body) = send(
        app.clone(),
        Method::PATCH,
        &format!("/api/todos/{first_id}"),
        Some(json!({ "title": " renamed " })),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"], "renamed");

    let (status, body) = send(
        app.clone(),
        Method::POST,
        "/api/todos/toggle-all",
        Some(json!({ "completed": true })),
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().map(Vec::len), Some(2));
    assert!(
        body["data"]
            .as_array()
            .ok_or_else(|| invalid_data("toggle-all response missing data"))?
            .iter()
            .all(|todo| todo
                .get("completed")
                .and_then(Value::as_bool)
                .unwrap_or(false))
    );

    let (status, body) = send(app.clone(), Method::DELETE, "/api/todos/completed", None).await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().map(Vec::len), Some(0));

    let (status, body) = send(
        app.clone(),
        Method::POST,
        "/api/todos",
        Some(json!({ "title": "third" })),
    )
    .await?;
    assert_eq!(status, StatusCode::CREATED);
    let third_id = body["data"]["id"]
        .as_i64()
        .ok_or_else(|| invalid_data("third created todo missing id"))?;

    let (status, body) = send(
        app.clone(),
        Method::DELETE,
        &format!("/api/todos/{third_id}"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["deleted"], true);

    let (status, body) = send(
        app.clone(),
        Method::DELETE,
        &format!("/api/todos/{second_id}"),
        None,
    )
    .await?;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["error"]["code"], "not_found");

    db.cleanup().await
}

async fn send(
    app: Router,
    method: Method,
    uri: &str,
    body: Option<Value>,
) -> Result<(StatusCode, Value), Box<dyn Error>> {
    let request = request(method, uri, body)?;
    let response = app.oneshot(request).await?;
    let status = response.status();
    let bytes = to_bytes(response.into_body(), usize::MAX).await?;
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes)?
    };

    Ok((status, body))
}

fn request(
    method: Method,
    uri: &str,
    body: Option<Value>,
) -> Result<Request<Body>, Box<dyn Error>> {
    let mut builder = Request::builder().method(method).uri(uri);
    let body = if let Some(body) = body {
        builder = builder.header("content-type", "application/json");
        Body::from(serde_json::to_vec(&body)?)
    } else {
        Body::empty()
    };

    Ok(builder.body(body)?)
}

fn invalid_data(message: &'static str) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, message)
}
