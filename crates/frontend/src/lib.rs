pub mod api;
pub mod state;
pub mod types;

use leptos::prelude::*;
use leptos::task::spawn_local;
use state::TodoState;
use types::{Todo, UpdateTodoRequest};

#[component]
pub fn App() -> impl IntoView {
    view! { <TodoApp/> }
}

#[component]
pub fn TodoApp() -> impl IntoView {
    let state = TodoState::new();

    Effect::new(move |_| {
        spawn_local(async move {
            let _result = state.refresh().await;
        });
    });

    view! {
        <section class="todoapp">
            <TodoHeader state=state/>
            <Show when=move || !state.todos.get().is_empty()>
                <section class="main">
                    <TodoList state=state/>
                </section>
                <footer class="footer"></footer>
            </Show>
        </section>
    }
}

#[component]
fn TodoHeader(state: TodoState) -> impl IntoView {
    let title = RwSignal::new(String::new());
    let disabled = move || state.loading.get();
    let submit = move || {
        let trimmed = title.get().trim().to_owned();

        if trimmed.is_empty() {
            title.set(String::new());
            return;
        }

        title.set(String::new());
        spawn_local(async move {
            let _result = state.create_todo(trimmed).await;
        });
    };

    view! {
        <header class="header">
            <h1>"todos"</h1>
            <input
                class="new-todo"
                placeholder="What needs to be done?"
                autofocus=true
                disabled=disabled
                prop:value=move || title.get()
                on:input=move |event| title.set(event_target_value(&event))
                on:keydown=move |event| {
                    if event.key() == "Enter" {
                        submit();
                    }
                }
            />
        </header>
    }
}

#[component]
fn TodoList(state: TodoState) -> impl IntoView {
    view! {
        <ul class="todo-list">
            <For
                each=move || state.visible_todos()
                key=|todo| todo.id
                let:todo
            >
                <TodoItem state=state todo=todo/>
            </For>
        </ul>
    }
}

#[component]
fn TodoItem(state: TodoState, todo: Todo) -> impl IntoView {
    let id = todo.id;
    let title = todo.title;
    let completed = todo.completed;
    let editing = RwSignal::new(false);
    let draft = RwSignal::new(title.clone());
    let title_for_start = title.clone();
    let title_for_cancel = title.clone();
    let title_for_save = title.clone();
    let label_title = title;
    let item_class = move || match (completed, editing.get()) {
        (true, true) => "completed editing",
        (true, false) => "completed",
        (false, true) => "editing",
        (false, false) => "",
    };
    let toggle = move || {
        spawn_local(async move {
            let _result = state
                .update_todo(
                    id,
                    UpdateTodoRequest {
                        title: None,
                        completed: Some(!completed),
                    },
                )
                .await;
        });
    };
    let destroy = move || {
        spawn_local(async move {
            let _result = state.delete_todo(id).await;
        });
    };
    let start_editing = move || {
        draft.set(title_for_start.clone());
        editing.set(true);
    };
    let cancel_editing = move || {
        draft.set(title_for_cancel.clone());
        editing.set(false);
    };
    let save_editing = move || {
        let trimmed = draft.get().trim().to_owned();
        editing.set(false);

        if trimmed.is_empty() {
            spawn_local(async move {
                let _result = state.delete_todo(id).await;
            });
            return;
        }

        if trimmed == title_for_save {
            return;
        }

        spawn_local(async move {
            let _result = state
                .update_todo(
                    id,
                    UpdateTodoRequest {
                        title: Some(trimmed),
                        completed: None,
                    },
                )
                .await;
        });
    };

    view! {
        <li class=item_class>
            <div class="view">
                <input
                    class="toggle"
                    type="checkbox"
                    prop:checked=completed
                    on:change=move |_event| toggle()
                />
                <label on:dblclick=move |_event| start_editing()>{label_title}</label>
                <button
                    class="destroy"
                    type="button"
                    aria-label="Delete todo"
                    on:click=move |_event| destroy()
                ></button>
            </div>
            <input
                class="edit"
                prop:value=move || draft.get()
                on:input=move |event| draft.set(event_target_value(&event))
                on:keydown=move |event| {
                    match event.key().as_str() {
                        "Enter" => save_editing(),
                        "Escape" => cancel_editing(),
                        _ => {}
                    }
                }
            />
        </li>
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn mount() {
    leptos::mount::mount_to_body(App);
}
