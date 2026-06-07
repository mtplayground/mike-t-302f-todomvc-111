# mike-t-302f-todomvc-111

TodoMVC implemented as a Rust workspace with an Axum backend, PostgreSQL persistence, and a Leptos CSR frontend built by Trunk.

## Runtime Configuration

The backend reads configuration from environment variables:

- `DATABASE_URL`: required PostgreSQL connection string. The app runs embedded SQLx migrations on startup.
- `BIND_ADDRESS`: optional socket address for the HTTP server. Defaults to `0.0.0.0:8080`.
- `FRONTEND_DIST_DIR`: optional path to the Trunk build output. Defaults to `crates/frontend/dist`.
- `DB_MAX_CONNECTIONS`: optional SQLx pool size. Defaults to `5` and must be greater than zero.

Persistent state must live in PostgreSQL. Do not use SQLite, JSON files, in-memory storage, or local volumes for todo persistence.

## Local Build

Install the Rust WASM target and Trunk, then build the frontend and backend:

```bash
rustup target add wasm32-unknown-unknown
cd crates/frontend
trunk build --release
cd ../..
cargo build --release -p backend
```

Run the server with a PostgreSQL URL:

```bash
export DATABASE_URL='postgres://user:password@host:5432/database'
export BIND_ADDRESS='0.0.0.0:8080'
export FRONTEND_DIST_DIR='crates/frontend/dist'
./target/release/backend
```

The backend serves `/api/todos`, `/health`, and the static frontend assets. `/health` performs a lightweight database query and returns a JSON error response if PostgreSQL is unavailable.

## Tests

Backend tests create isolated schemas inside the configured PostgreSQL database:

```bash
export DATABASE_URL='postgres://user:password@host:5432/database'
cargo test -p backend
```

End-to-end tests use Playwright against a running app:

```bash
npm install
E2E_BASE_URL='http://127.0.0.1:8080' npm run test:e2e
```

If you want Playwright to start the app process, provide `E2E_START_SERVER`:

```bash
E2E_BASE_URL='http://127.0.0.1:8080' \
E2E_START_SERVER='DATABASE_URL=$DATABASE_URL FRONTEND_DIST_DIR=crates/frontend/dist BIND_ADDRESS=127.0.0.1:8080 ./target/release/backend' \
npm run test:e2e
```

## Bare Self-Hosted Deployment

Build artifacts needed on the host:

- `target/release/backend`
- `crates/frontend/dist`

Example systemd service:

```ini
[Unit]
Description=TodoMVC
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=/opt/todomvc
Environment=DATABASE_URL=postgres://user:password@host:5432/database
Environment=BIND_ADDRESS=0.0.0.0:8080
Environment=FRONTEND_DIST_DIR=/opt/todomvc/crates/frontend/dist
ExecStart=/opt/todomvc/target/release/backend
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Keep `DATABASE_URL` in the host secret manager or service environment, not in the repository. Ensure the service user can read the frontend `dist` directory and execute the backend binary.
