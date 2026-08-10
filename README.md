# Curio

Curio uses email-only mock authentication and a Cloudflare Worker chat backend:

```text
Web ───────────────┐
                   ├─> chat worker ─> Gemini streaming ─> D1 (SQLite)
Desktop ─> Tauri ──┘                         │
                                             └─> queue ─> processor worker ─> D1
```

The chat worker stores conversations, messages, and attachment BLOBs. The
processor worker reads those records, generates embeddings, and stores the
embedding payloads in the same D1 database. Authentication remains local mock
state; entering a valid email creates a mock user and opens Home.

## Worker configuration

Both workers must bind `CURIO_DB` to the same D1 database and use the shared
`curio-workers/migrations` directory. Before remote deployment:

1. Create a D1 database named `curio-worker-db`.
2. Put its `database_id` in both worker `wrangler.jsonc` files.
3. Store `GEMINI_API_KEY` as a secret for both workers.
4. Apply the D1 migrations remotely.
5. Deploy the processor and chat workers.

See `curio-workers/README.md` for the exact local and remote commands.

## Run locally

Start the chat worker after applying the local D1 migration:

```bash
cd curio-workers/curio-chat-worker
npx wrangler d1 migrations apply curio-worker-db --local --persist-to ../.wrangler-state
npm run dev
```

Start the web client in another terminal:

```bash
cd web/curio
cp .env.example .env
npm install
npm run dev
```

The web client defaults to `http://127.0.0.1:8787`. Set
`VITE_CURIO_CHAT_WORKER_URL` to use a deployed chat worker.

Start the desktop client with the same worker URL:

```bash
cd desktop/Curio
npm install
CURIO_CHAT_WORKER_URL=http://127.0.0.1:8787 npm run tauri dev
```

## Optional local service

`curio-service` remains available as a standalone OpenAI/SQLite reference
implementation, but the web and desktop clients no longer use it by default.

## Verification

```bash
cd curio-workers/curio-chat-worker
npx tsc --noEmit
npm test -- --run

cd ../curio-processor-worker
npx tsc --noEmit
npm test -- --run

cd ../../web/curio
npm test
npm run lint
npm run build

cd ../../desktop/Curio
npm run build
cd src-tauri
cargo fmt --check
cargo test
```
