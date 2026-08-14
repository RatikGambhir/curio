# Curio

Curio has one React/Vite frontend in `web/curio`. It is built either as a web
SPA or as the UI embedded in the Tauri desktop shell located at
`web/curio/src-tauri`.

```text
                         web/curio/src
                   shared React application
                              │
                   typed platform contract
                    /                    \
          browser fetch adapter     Tauri channel adapter
                    \                    /
                       Cloudflare chat worker
                          Gemini + D1 + queue
```

The chat worker is the active backend. `curio-service` is an optional
OpenAI/SQLite reference implementation and is not used by either client.
Authentication remains local mock state; it is shared by both builds but is not
production authentication.

See [ARCHITECTURE.md](ARCHITECTURE.md) for the code boundaries and streaming
flow, and [docs/deployment.md](docs/deployment.md) for deployment, SPA fallback,
CORS, and desktop signing notes.

## Local development

Start the chat worker after applying its local D1 migration:

```bash
cd curio-workers/curio-chat-worker
npx wrangler d1 migrations apply curio-worker-db --local --persist-to ../.wrangler-state
npm run dev
```

Install the canonical frontend once, then choose a target:

```bash
cd web/curio
npm install
npm run dev:web
```

```bash
cd web/curio
npm run dev:desktop
```

Development defaults to `http://127.0.0.1:8787`. Override the public worker URL
with `VITE_CURIO_CHAT_WORKER_URL`. Desktop configuration is compiled into its UI,
so the packaged app does not require a runtime shell variable.

Production builds require that variable to be an explicit HTTPS URL. The worker
has not been deployed from this repository yet, so no guessed Workers hostname
is compiled as a fallback.

## Builds and verification

Set `VITE_CURIO_CHAT_WORKER_URL` to the deployed HTTPS worker (or place it in a
mode-specific environment file) before running production builds:

```bash
cd web/curio
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

Worker checks remain in each worker package. Both workers must bind `CURIO_DB`
to the same D1 database and use `curio-workers/migrations`.
