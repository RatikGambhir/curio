# Curio workers

The chat worker is the active streaming backend. It writes conversations, messages,
attachment BLOBs, and embeddings to Cloudflare D1, which uses SQLite semantics. The
processor worker consumes the chat queue, reads the saved records, generates
embeddings, and writes those embeddings back to the same D1 database.

## Local setup

Apply the shared migration before starting either worker:

```bash
cd curio-chat-worker
npx wrangler d1 migrations apply curio-worker-db --local --persist-to ../.wrangler-state
npm run dev
```

Run the processor in another terminal after applying the same migration:

```bash
cd curio-processor-worker
npx wrangler d1 migrations apply curio-worker-db --local --persist-to ../.wrangler-state
npm run dev
```

The shared frontend defaults to `http://127.0.0.1:8787` during development.
Override it at build time with `VITE_CURIO_CHAT_WORKER_URL` for either target.

## Remote setup

Create one D1 database, add its returned `database_id` to both `wrangler.jsonc`
files, apply the migrations remotely, then deploy both workers. Both workers must
bind `CURIO_DB` to the same database.

Set `CURIO_ALLOWED_ORIGINS` on the chat worker to the exact, comma-separated web
origins allowed to call it. The fallback allowlist contains only common local
development origins. Native desktop traffic is bridged by Rust and does not
depend on browser CORS headers.
