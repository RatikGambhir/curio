# Curio

Curio has one React/Vite frontend in `web`. It is built either as a web
SPA or as the UI embedded in the Tauri desktop shell located at
`web/src-tauri`.

```text
                            web/src
                   shared React application
                              │
                typed API layer + platform contract
                    /                    \
        browser HTTP adapter       Tauri channel adapter
                   |                      |
                   |             Rust HTTP bridge (src-tauri)
                    \                    /
                          curio-service
                  Axum + OpenAI + PostgreSQL
```

`curio-service` is the backend for both clients. On the web, requests go
straight from the browser to the service; on desktop, the renderer invokes an
app-owned Tauri command and the Rust process performs the HTTP request.
Authentication remains local mock state; it is shared by both builds but is not
production authentication. The Cloudflare workers under `curio-workers` are
legacy and no longer referenced by the clients.

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the code boundaries and streaming
flow, and [docs/deployment.md](docs/deployment.md) for deployment, SPA fallback,
CORS, and desktop signing notes.

## Local development

Copy `curio-service/.env.example` to `curio-service/.env`, then provide a
PostgreSQL application-role URL in `DATABASE_URL`, a migrator-role URL in
`CURIO_MIGRATOR_DATABASE_URL`, and `CURIO_DB_SCHEMA=curio_dev`. Local access to
Railway requires its TLS public TCP proxy; do not use the production or Railway
administrator credential for development.

The current Vite server uses port 1420. Until the service example is aligned,
also include `http://localhost:1420,http://127.0.0.1:1420` in
`CURIO_CORS_ALLOWED_ORIGINS`; copying the example as-is lists only port 5173.

Apply migrations explicitly, then start the service:

```bash
cd curio-service
cargo run --bin curio_db -- migrate
cargo run --bin curio-service
```

The web and desktop clients continue to connect only to `curio-service`; no
PostgreSQL URL belongs in a Vite variable or client bundle.

Install the canonical frontend once, then choose a target:

```bash
cd web
npm install
npm run dev:web
```

```bash
cd web
npm run dev:desktop
```

Development defaults to `http://127.0.0.1:3000`. Override the public service URL
with `VITE_CURIO_SERVICE_URL`. Desktop configuration is compiled into its UI,
so the packaged app does not require a runtime shell variable.

Production builds require that variable to be an explicit HTTPS URL.

## Builds and verification

Set `VITE_CURIO_SERVICE_URL` to the deployed HTTPS service (or place it in a
mode-specific environment file) before running production builds:

```bash
cd web
npm run check:architecture
npm run lint
npm test
npm run build:web
npm run build:desktop-ui
npm run build:desktop

cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Service checks run from `curio-service`. `CURIO_TEST_DATABASE_URL` must identify
a non-production test-runner role; the tests create and remove isolated
`curio_test_*` schemas:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo test --features sqlite-import --test sqlite_import
```
