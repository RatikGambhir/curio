# Rust MQTT embedding service and Cloudflare Worker retirement

Status: proposed. This plan does not describe a deployed feature.

## Decision

Replace the disconnected `curio-workers/` Gemini/D1/Cloudflare Queue pipeline with a separate, always-on Rust Railway service named `curio-embed-worker`. Railway's product named *Functions* currently runs single-file TypeScript/Bun; a persistent Rust MQTT subscriber belongs in a Railway background service. See [Railway Functions](https://docs.railway.com/functions) and [background workers](https://docs.railway.com/guides/cron-workers-queues).

Scope for the first release: embed nonempty, completed **user and assistant message text** from the active PostgreSQL `messages` table, save vectors in PostgreSQL, and leave the public chat HTTP/SSE contract unchanged. The current Axum service remains the only chat streaming backend. Attachment embedding needs its own storage migration before it can be part of the active flow.

## Current state and gaps

- `curio-service` inserts a completed user message and a pending assistant before calling OpenAI. It commits completed assistant content and `response_id` before emitting the Curio `done` SSE event. Failed/interrupted assistants are stored with separate states.
- The disconnected chat Worker used Gemini streaming, persisted to D1, and optionally sent `userId`, `threadId`, `userMessageId`, `assistantMessageId`, and `assetPath[]` to a Cloudflare Queue. The processor loaded both messages and assets, called Gemini embeddings, and stored a JSON response in D1.
- Active PostgreSQL has no embedding, attachment, or durable embedding-job table. Its chat rows have no owner. The active chat request has no trusted `userId`, so the legacy `userId` cannot safely be carried over as an ownership claim.
- D1 records and Gemini vectors do not automatically exist in PostgreSQL. Inventory them before deleting the old deployment or source.

## Target flow

```text
POST /v1/chat/stream
  -> transaction inserts completed user + pending assistant
  -> OpenAI tokens stream to the client
  -> one transaction completes assistant and inserts two embedding jobs
  -> Curio `done` SSE event after that transaction commits

curio-service job dispatcher
  -> MQTT QoS 1 notification to private broker
  -> curio-embed-worker subscription
  -> claim job and fetch saved content from PostgreSQL
  -> OpenAI embeddings API
  -> one transaction upserts vector and completes job
```

PostgreSQL is the durable work queue; MQTT is a prompt notification. The worker also scans due jobs on an interval so a missed notification or broker outage cannot strand work. Embedding and MQTT latency never hold up the chat response. Broker publication happens only after the completion transaction commits.

### MQTT v1 contract

Topic: `curio/v1/embeddings/messages`; QoS 1; no retained message. One notification per message job:

```json
{
  "version": 1,
  "jobId": "<server-generated UUID>",
  "messageId": "<saved message ID>",
  "conversationId": "<saved conversation ID>",
  "role": "assistant",
  "responseId": "<provider response ID or null>",
  "completedAt": "<UTC timestamp>",
  "model": "text-embedding-3-small"
}
```

The user-message event has `role: "user"` and `responseId: null`. `conversationId` replaces legacy `threadId`; the former `userMessageId` and `assistantMessageId` become independent jobs. No prompt, response text, vector, API key, or attachment bytes enter MQTT. Validate topic, version, payload size, field lengths, and shape. Treat IDs and metadata as hints: fetch the job and row, then verify message ID, conversation, role, status, model, and content hash against PostgreSQL before using the content. Log mismatches without sensitive content.

Do not send `userId` until chat has authenticated, persisted ownership; do not copy an unverified client value. Do not send `assetPath` because the active service has no asset records. When attachments are ported, use immutable database asset IDs and a versioned contract.

## Database and code changes

Add an append-only `curio-service/migrations/postgres` migration:

1. `embedding_jobs`: UUID primary key; `message_id` FK with cascade delete; model; SHA-256 of the exact saved UTF-8 content; state (`pending`, `processing`, `retry`, `completed`, `dead`, `superseded`); attempts; `available_at`, lease expiry, last publish time, timestamps, and sanitized error code. Unique `(message_id, model, content_sha256)`; index due jobs and expired leases.
2. `message_embeddings`: `(message_id, model)` primary key and FK to `messages` with cascade delete; source SHA-256; dimension count; `real[]` vector; timestamps; array shape/dimension constraint. Save numeric floats, not the full provider response. Validate finite values, model, and dimensions in Rust. Upsert vector and mark job complete in the same transaction.

Use OpenAI `text-embedding-3-small` at the documented default 1,536 dimensions initially, with explicit model/dimension configuration; see the [OpenAI embedding guide](https://developers.openai.com/api/docs/guides/embeddings). Store model and source hash to support deliberate re-embedding. Start with PostgreSQL arrays because Railway's default PostgreSQL image does not include pgvector; see [Railway extension guidance](https://docs.railway.com/databases/postgresql#extensions). If similarity search is later required, provision a pgvector-capable database and migrate to `vector` plus an appropriate index, following [pgvector](https://github.com/pgvector/pgvector). Array storage here does not promise indexed vector search.

Insert both jobs in the same transaction that marks the assistant completed. Skip empty content. Do not enqueue pending, failed, or interrupted assistants. If job insertion fails, roll back completion and emit the existing sanitized storage error, retaining persistence-before-`done`. Backfill policy for a user message whose assistant failed must be decided explicitly; the proposed live path waits for a completed assistant turn. Add verification checks to `curio-service/ops/postgres/verify.sql`, assess SQLite importer compatibility, and continue to apply DDL only through `curio_db migrate` with migrator credentials.

Place embedding domain, repository, provider adapter, MQTT contract, and processor under `curio-service/src/embedding/`, with `curio-service/src/bin/curio_embed_worker.rs` as a separate process in the existing Cargo package. Extend Chat's repository/service boundary to create the jobs. Do not add a root Cargo workspace.

The service-side dispatcher claims due unpublished jobs safely across API replicas, publishes notifications, and records broker acceptance. Publication failures leave rows retryable. The worker starts with one replica, reconnects with bounded backoff, uses a stable MQTT client ID, limits concurrency, and leases jobs with `FOR UPDATE SKIP LOCKED` or equivalent. Hold no database lock during the provider call. On receipt or periodic scan, claim a job, fetch the completed row, check nonempty content and hash, call `/v1/embeddings` over HTTPS, then recheck source status/hash while committing. A stale result must not overwrite an embedding for edited content; edits supersede old jobs and enqueue a new hash, while deletes cascade. Current chat exposes no message editing, but retries still need this guard.

Retry transport failures, 429, and provider 5xx with exponential backoff and jitter. Classify invalid inputs/permanent failures as `dead`, cap attempts, and provide operator replay. Duplicate QoS 1 deliveries converge through the unique job key, lease, and idempotent `(message_id, model)` upsert. Expired leases and a reconciliation scan recover from crashes and find completed messages missing a job or vector. Log only IDs, error class, attempts, lag, and duration; never log message content, provider bodies, secrets, or full MQTT payloads.

## Railway deployment

1. Add a private MQTT broker in the same Railway environment, for example Mosquitto. Enable authentication, separate producer/consumer ACLs, a persistent volume, and no public TCP proxy/domain. Railway [private networking](https://docs.railway.com/networking/private-networking) carries TCP; [Mosquitto configuration](https://www.mosquitto.org/man/mosquitto-conf-5.html) covers persistence and ACLs.
2. Deploy the Rust binary as a separate Railway service rooted at `/curio-service`, with its own config such as `curio-service/railway-embedding.toml` and explicit build/start commands. Keep the Axum service's `railway.toml`. Confirm the new binary is built and both processes verify the same migration. See Railway's [monorepo guidance](https://docs.railway.com/deployments/monorepo).
3. Set private DB URL, validated `CURIO_DB_SCHEMA`, broker host/port and distinct MQTT credentials, `OPENAI_API_KEY`, model/dimensions, pool/concurrency limits, and retry bounds through Railway secrets/config. The worker gets a least-privilege application role, never the migrator URL or `VITE_*` variables. Budget the combined SQLx pools across replicas.
4. If Railway uses an HTTP healthcheck, give the worker a private liveness/readiness listener. Readiness requires DB migration verification and an active MQTT subscription. Monitor pending/dead jobs, oldest job age, retries, broker connectivity, provider errors, and processing time.

## Implementation and retirement gates

| Phase | Work | Exit criterion |
| --- | --- | --- |
| 0. Inventory | Check live Railway/Cloudflare deployments, bindings, D1 row/assets counts, queue backlog, and retention needs. Confirm no client uses either Worker. | Decide whether D1 data/assets need import, export, or archival. |
| 1. Persistence | Add migration, DB grants, verification SQL, job/vector repositories, and provider validation. | Disposable PostgreSQL tests prove constraints, transaction boundaries, duplicates, cascade, and stale-result guards. |
| 2. Runtime | Add atomic completion/job insertion, dispatcher, MQTT v1 contract, Rust worker, and replay/reconciliation. | End-to-end tests show `done` after DB commit, DB content lookup by ID, eventual vector with duplicate/lost MQTT signals, and chat success despite later embedding failure. |
| 3. Deployment | Migrate schema first; deploy broker/worker privately; enable producer with a server-side flag. Backfill nonempty completed PostgreSQL messages using bounded, resumable job insertion. | Staging and production backlog drains; vectors have expected model/dimension; pause/replay is exercised. |
| 4. Retirement | Retain or migrate verified D1 data, then remove `curio-workers/` and unused Cloudflare deployments, bindings, and secrets. Update `AGENTS.md`, `README.md`, `docs/ARCHITECTURE.md`, `docs/deployment.md`, and CI/verification references. Keep older `plans/` as history. | No live integration or current guide depends on the Workers; recovery data is retained for the agreed window. |

If D1 contains attachments that must remain usable, or attachment embedding is required for parity, add an attachment-storage/import phase before Phase 4: choose PostgreSQL/object storage, define ownership and stable asset IDs, extraction per supported MIME type, size/security limits, an asset job source, and tests. The old processor base64-encoded arbitrary bytes into a text embedding request; do not repeat that behavior. Record the no-assets finding if this phase is unnecessary.

## Verification and rollback

Run unit tests for the MQTT schema, source-hash validation, embedding shape, retry classification, and sanitized errors. Use disposable `curio_test_*` PostgreSQL schemas for migration, atomic completion, lease competition, idempotent upsert, crash/retry, delete cascade, and backfill tests. A green `cargo test` without safe `CURIO_TEST_DATABASE_URL` is partial evidence. Exercise a local broker and mocked provider through disconnects before/after publish, duplicate QoS 1 events, broker outage, timeout, 429/5xx, and worker restart. Confirm `done` never waits for embedding.

From `curio-service`, run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, database-backed tests with a safe test URL, and importer targets if affected. Run `curio_db verify` only against an explicitly identified safe schema. Run frontend chat contract checks if the public HTTP/SSE contract changes. Check `git diff --check` and the final diff. As each phase lands, compare implementation with canonical `docs/ARCHITECTURE.md` and update current-state sections and `docs/deployment.md` when behavior or operations actually change.

Ship schema and worker before enabling the producer. To stop a bad rollout, disable producer/worker while retaining jobs and vectors, then fix and replay. Do not edit an applied migration to roll back. Delete Cloudflare resources only after the Rust path and D1 retention decision pass the gates.
