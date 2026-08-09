# Curio chat vertical slice

Curio’s development chat path is centralized through `curio-service`:

```text
Web ───────────────────────┐
                           ├─> curio-service ─> OpenAI Responses API
Desktop React ─> Tauri Rust┘         │
                                     └─> local SQLite
```

This is a local, development-ready vertical slice. It is not a public production deployment.

## Configuration

Only `curio-service` receives `OPENAI_API_KEY`. Do not add it to Vite variables, desktop configuration, or committed files.

Service variables:

- `OPENAI_API_KEY` — required; keep the value local.
- `OPENAI_MODEL` — required; choose the model in local configuration.
- `OPENAI_BASE_URL` — optional; defaults to `https://api.openai.com` and is useful for mock-provider testing.
- `CURIO_SERVICE_ADDR` — optional; defaults to `127.0.0.1:3000`.
- `CURIO_DATABASE_URL` — optional; defaults to `sqlite://curio.db`.
- `CURIO_CORS_ALLOWED_ORIGINS` — optional comma-separated web origins; defaults to the common Vite localhost origins.

Web variables are documented in `web/curio/.env.example`. Supabase remains an authentication provider only; chat streaming and persistence do not call its database.

Desktop reads `CURIO_SERVICE_URL` in Rust and defaults to `http://127.0.0.1:3000`.

## Run locally

Start the service from `curio-service` so the default database is created at `curio-service/curio.db`:

```bash
cd curio-service
cp .env.example .env
# Fill OPENAI_API_KEY and OPENAI_MODEL locally, then export the file.
set -a
source .env
set +a
cargo run
```

Start the web client in another terminal:

```bash
cd web/curio
cp .env.example .env
npm install
npm run dev
```

Start the desktop client in another terminal:

```bash
cd desktop/Curio
npm install
CURIO_SERVICE_URL=http://127.0.0.1:3000 npm run tauri dev
```

The service calls the OpenAI Responses endpoint with `stream: true` and normalizes the documented `response.output_text.delta`, `response.completed`, and failure lifecycle into Curio `token`, `done`, and `error` SSE events. See the [official OpenAI streaming documentation](https://developers.openai.com/api/docs/guides/streaming-responses).

## Local persistence API

- `GET /health`
- `POST /v1/chat/stream`
- `GET /v1/conversations`
- `GET /v1/conversations/{conversationId}/messages`

The user message and pending assistant row are created transactionally. Assistant tokens are buffered in memory and written once on completion or terminal failure; tokens are not inserted individually.

To verify persistence across a restart, send a completed chat, restart the service with the same `CURIO_DATABASE_URL`, then query:

```bash
curl http://127.0.0.1:3000/v1/conversations
curl http://127.0.0.1:3000/v1/conversations/CONVERSATION_ID/messages
```

## Verification

```bash
cd curio-service
cargo fmt --check
cargo check
cargo clippy --all-targets -- -D warnings
cargo test

cd ../web/curio
npm test
npm run lint
npm run build

cd ../../desktop/Curio
npm run build
cd src-tauri
cargo check
cargo test
cargo clippy --all-targets -- -D warnings
cd ..
npm run tauri build -- --debug --no-bundle
```

## Intentionally deferred

- Public deployment, production authentication/authorization for service chat routes, rate limiting, and production CORS.
- Loading the persisted conversation index into the current prototype sidebars; persistence is available through the service history endpoints.
- Model selection in clients; the model is centrally configured with `OPENAI_MODEL`.
- macOS DMG packaging/signing/notarization. The development binary is covered by the no-bundle Tauri debug build.
- Existing large frontend asset/chunk optimization.
