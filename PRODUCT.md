# Product Snapshot

## What This Project Is

`mike-t-302f-todomvc-111` is a TodoMVC web app implemented as a Rust workspace. It has an Axum backend, a Leptos CSR frontend built with Trunk, and PostgreSQL-backed persistence for all todo state.

## What It Does

The app implements the standard TodoMVC workflow:

- Add todos from the header input, ignoring empty or whitespace-only titles.
- List todos with completed styling.
- Toggle individual todos and delete them.
- Edit todos inline with double-click, Enter to save, Escape to cancel, and empty edits deleting the todo.
- Filter by `All`, `Active`, and `Completed` using URL hash routing.
- Show live active-item counts.
- Clear completed todos when any completed item exists.
- Toggle all todos between active and completed.
- Hide the main list and footer when there are no todos.

## Architecture

- Root workspace: `Cargo.toml`
- Backend crate: `crates/backend`
- Frontend crate: `crates/frontend`
- E2E tests: `e2e`

Backend:

- Axum serves `/api/todos`, `/health`, and built frontend assets.
- SQLx uses PostgreSQL only; migrations are embedded and run at startup.
- `todos` table stores `id`, `title`, `completed`, and `created_at`.
- Centralized `AppError` maps failures to JSON error bodies.
- Server-side failures are logged with status, error code, debug output, and source chain.
- `/health` verifies database connectivity with a lightweight query.

Frontend:

- Leptos CSR app compiled by Trunk.
- TodoMVC CSS is bundled under `crates/frontend/assets`.
- Browser state is held in Leptos signals with a small API client layer for backend calls.

## Configuration

Required:

- `DATABASE_URL`: PostgreSQL connection string.

Optional:

- `BIND_ADDRESS`: backend bind address, default `0.0.0.0:8080`.
- `FRONTEND_DIST_DIR`: Trunk output path, default `crates/frontend/dist`.
- `DB_MAX_CONNECTIONS`: SQLx pool size, default `5`.

Persistent storage must remain PostgreSQL. Do not add SQLite, JSON-file persistence, in-memory persistence, or local-volume persistence.

## Build, Run, And Test

Build:

```bash
rustup target add wasm32-unknown-unknown
cd crates/frontend && trunk build --release
cd ../..
cargo build --release -p backend
```

Run:

```bash
export DATABASE_URL='postgres://user:password@host:5432/database'
export BIND_ADDRESS='0.0.0.0:8080'
export FRONTEND_DIST_DIR='crates/frontend/dist'
./target/release/backend
```

Tests:

```bash
export DATABASE_URL='postgres://user:password@host:5432/database'
cargo test -p backend
npm install
E2E_BASE_URL='http://127.0.0.1:8080' npm run test:e2e
```

## Conventions

- Keep API responses wrapped as `{ "data": ... }` on success and `{ "error": { "code", "message" } }` on failure.
- Validate todo titles on both create and update paths.
- Keep routes under `/api/...` so they do not collide with static frontend fallback serving.
- Keep deployment documentation in `README.md` and this product snapshot current after feature batches.
