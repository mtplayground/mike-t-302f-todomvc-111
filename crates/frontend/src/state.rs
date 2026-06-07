use crate::{
    api::{self, ApiError},
    types::{Todo, UpdateTodoRequest},
};
use leptos::prelude::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TodoFilter {
    All,
    Active,
    Completed,
}

impl TodoFilter {
    pub fn from_hash(hash: &str) -> Self {
        match hash.trim_start_matches('#') {
            "/active" | "active" => Self::Active,
            "/completed" | "completed" => Self::Completed,
            _ => Self::All,
        }
    }

    pub fn hash(self) -> &'static str {
        match self {
            Self::All => "#/",
            Self::Active => "#/active",
            Self::Completed => "#/completed",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::All => "All",
            Self::Active => "Active",
            Self::Completed => "Completed",
        }
    }

    pub fn matches(self, todo: &Todo) -> bool {
        match self {
            Self::All => true,
            Self::Active => !todo.completed,
            Self::Completed => todo.completed,
        }
    }
}

#[derive(Clone, Copy)]
pub struct TodoState {
    pub todos: RwSignal<Vec<Todo>>,
    pub filter: RwSignal<TodoFilter>,
    pub loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
}

impl TodoState {
    pub fn new() -> Self {
        Self {
            todos: RwSignal::new(Vec::new()),
            filter: RwSignal::new(TodoFilter::All),
            loading: RwSignal::new(false),
            error: RwSignal::new(None),
        }
    }

    pub fn visible_todos(self) -> Vec<Todo> {
        let filter = self.filter.get();

        self.todos
            .get()
            .into_iter()
            .filter(|todo| filter.matches(todo))
            .collect()
    }

    pub fn active_count(self) -> usize {
        self.todos
            .get()
            .iter()
            .filter(|todo| !todo.completed)
            .count()
    }

    pub fn completed_count(self) -> usize {
        self.todos
            .get()
            .iter()
            .filter(|todo| todo.completed)
            .count()
    }

    pub async fn refresh(self) -> Result<(), ApiError> {
        self.run_list_request(api::list_todos()).await
    }

    pub async fn create_todo(self, title: impl Into<String>) -> Result<(), ApiError> {
        let todo = self.run_item_request(api::create_todo(title)).await?;
        self.todos.update(|todos| todos.push(todo));
        Ok(())
    }

    pub async fn update_todo(self, id: i64, update: UpdateTodoRequest) -> Result<(), ApiError> {
        let todo = self.run_item_request(api::update_todo(id, update)).await?;
        self.todos.update(|todos| {
            if let Some(existing) = todos.iter_mut().find(|existing| existing.id == id) {
                *existing = todo;
            }
        });
        Ok(())
    }

    pub async fn delete_todo(self, id: i64) -> Result<(), ApiError> {
        self.start_request();
        let result = api::delete_todo(id).await;
        self.finish_request(result.map(|_deleted| {
            self.todos.update(|todos| {
                todos.retain(|todo| todo.id != id);
            });
        }))
    }

    pub async fn toggle_all(self, completed: bool) -> Result<(), ApiError> {
        self.run_list_request(api::toggle_all(completed)).await
    }

    pub async fn clear_completed(self) -> Result<(), ApiError> {
        self.run_list_request(api::clear_completed()).await
    }

    async fn run_list_request<F>(self, request: F) -> Result<(), ApiError>
    where
        F: std::future::Future<Output = Result<Vec<Todo>, ApiError>>,
    {
        self.start_request();
        let result = request.await;
        self.finish_request(result.map(|todos| self.todos.set(todos)))
    }

    async fn run_item_request<F>(self, request: F) -> Result<Todo, ApiError>
    where
        F: std::future::Future<Output = Result<Todo, ApiError>>,
    {
        self.start_request();
        let result = request.await;
        self.finish_request(result)
    }

    fn start_request(self) {
        self.loading.set(true);
        self.error.set(None);
    }

    fn finish_request<T>(self, result: Result<T, ApiError>) -> Result<T, ApiError> {
        self.loading.set(false);

        if let Err(error) = &result {
            self.error.set(Some(error.to_string()));
        }

        result
    }
}

impl Default for TodoState {
    fn default() -> Self {
        Self::new()
    }
}
