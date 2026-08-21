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
                     Axum + OpenAI + SQLite
```

`curio-service` is the backend for both clients. On the web, requests go
straight from the browser to the service; on desktop, the renderer invokes an
app-owned Tauri command and the Rust process performs the HTTP request.
Authentication remains local mock state; it is shared by both builds but is not
production authentication. The Cloudflare workers under `curio-workers` are
legacy and no longer referenced by the clients.

See [ARCHITECTURE.md](ARCHITECTURE.md) for the code boundaries and streaming
flow, and [docs/deployment.md](docs/deployment.md) for deployment, SPA fallback,
CORS, and desktop signing notes.

## Local development

Start the service (it applies its SQLite migrations at startup):

```bash
cd curio-service
OPENAI_API_KEY=... OPENAI_MODEL=gpt-4.1-mini cargo run
```

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

Service checks run from `curio-service` with `cargo fmt --check`,
`cargo clippy --all-targets -- -D warnings`, and `cargo test`.
