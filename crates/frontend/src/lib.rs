pub mod api;
pub mod state;
pub mod types;

use leptos::prelude::*;
use leptos::task::spawn_local;
use state::TodoState;

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
                <section class="main"></section>
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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn mount() {
    leptos::mount::mount_to_body(App);
}
