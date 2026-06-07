pub mod api;
pub mod state;
pub mod types;

use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <main class="todoapp">
            <h1>"todos"</h1>
        </main>
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn mount() {
    leptos::mount::mount_to_body(App);
}
