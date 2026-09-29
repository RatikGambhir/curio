# Rust RabbitMQ embedding service and Cloudflare Worker retirement

Status: proposed. This plan does not describe a deployed feature.

## Decision

Replace the disconnected `curio-workers/` Gemini/D1/Cloudflare Queue pipeline with a separate, always-on Rust Railway service named `curio-embed-worker`. Railway's product named *Functions* currently runs single-file TypeScript/Bun; a persistent Rust RabbitMQ consumer belongs in a Railway background service. See [Railway Functions](https://docs.railway.com/functions) and [background workers](https://docs.railway.com/guides/cron-workers-queues).

These are alternative broker implementations: choose this plan or [the NATS alternative](nats-embedding-railway-worker-plan.md), rather than deploying both for the same jobs.

Scope for the first release: embed nonempty, completed **user and assistant message text** from the active PostgreSQL `messages` table, save vectors in PostgreSQL, and leave the public chat HTTP/SSE contract unchanged. The current Axum service remains the only chat streaming backend. Chat attachment embedding remains outside this release. The existing documents domain already stores file versions, blobs, chunks, and vectors; connecting chat attachments to it requires an explicit identity, ownership, and job-source design.

## Current state and gaps

- `curio-service` inserts a completed user message and a pending assistant before calling OpenAI. It commits completed assistant content and `response_id` before emitting the Curio `done` SSE event. Failed/interrupted assistants are stored with separate states.
- The disconnected chat Worker used Gemini streaming, persisted to D1, and optionally sent `userId`, `threadId`, `userMessageId`, `assistantMessageId`, and `assetPath[]` to a Cloudflare Queue. The processor loaded both messages and assets, called Gemini embeddings, and stored a JSON response in D1.
- The existing Railway PostgreSQL database is the target; do not provision a replacement database. The active documents domain already persists file blobs, chunks, and embeddings, but the chat path has no completed message-embedding pipeline. Partial local work from the interrupted MQTT implementation must be inspected and adapted or explicitly removed during implementation, not assumed complete or deployed.
- Chat rows have no owner. The active chat request has no trusted `userId`, so the legacy `userId` cannot safely be carried over as an ownership claim.
- D1 records and Gemini vectors do not automatically exist in PostgreSQL. Inventory them before deleting the old deployment or source.

## Target flow

```text
POST /v1/chat/stream
  -> transaction inserts completed user + pending assistant
  -> OpenAI tokens stream to the client
  -> one transaction completes assistant and inserts two embedding jobs
  -> Curio `done` SSE event after that transaction commits

curio-service job dispatcher
  -> persistent AMQP notification + publisher confirm from private RabbitMQ
  -> curio-embed-worker manual-ack consumer
  -> claim job and fetch saved content from PostgreSQL
  -> OpenAI embeddings API
  -> one transaction upserts vector and completes job
  -> acknowledge RabbitMQ delivery
```

PostgreSQL is the durable work queue; RabbitMQ carries durable notifications. The worker also scans due jobs on an interval so a missed notification or broker outage cannot strand work. Embedding and RabbitMQ latency never hold up the chat response. Broker publication happens only after the completion transaction commits.

### RabbitMQ v1 contract

Use AMQP 0-9-1 with durable direct exchange `curio.embeddings.v1`, routing key `messages.embed.v1`, and durable, nonexclusive, non-auto-delete queue `curio.embeddings.messages.v1`. Bind that queue to the exact routing key. Publish persistent JSON messages (`content_type=application/json`, delivery mode 2) with mandatory routing, publisher confirms, and `message_id=<jobId>:<publishGeneration>`. Treat an unroutable return, negative confirmation, or timeout as a publication failure, even if a confirm arrives. These requirements follow [RabbitMQ confirms](https://www.rabbitmq.com/docs/confirms).

One notification per message job:

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

The user-message event has `role: "user"` and `responseId: null`. `conversationId` replaces legacy `threadId`; the former `userMessageId` and `assistantMessageId` become independent jobs. No prompt, response text, vector, API key, or attachment bytes enter RabbitMQ. Validate exchange/routing key, version, payload size, field lengths, and shape. Treat IDs and metadata as hints: fetch the job and row, then verify message ID, conversation, role, status, model, and content hash against PostgreSQL before using the content. Log mismatches without sensitive content.

Do not send `userId` until chat has authenticated, persisted ownership; do not copy an unverified client value. Do not send legacy `assetPath` values: chat messages currently have no authoritative link to the active document/file records. When chat attachments are integrated, use immutable database file/version IDs and a versioned contract.

## Database and code changes

Inspect any unapplied local migration from the interrupted implementation, then add or complete an append-only migration under `curio-service/migrations/postgres`. Never edit an applied migration:

1. `embedding_jobs`: UUID primary key; `message_id` FK with cascade delete; model; SHA-256 of the exact saved UTF-8 content; state (`pending`, `processing`, `retry`, `completed`, `dead`, `superseded`); attempts; `available_at`, lease expiry and fencing token, separate publication lease/token, last confirmed publish time, publish-generation counter, timestamps, and sanitized error code. Unique `(message_id, model, content_sha256)`; index due jobs and expired leases.
2. `message_embeddings`: `(message_id, model)` primary key and FK to `messages` with cascade delete; source SHA-256; dimension count; `real[]` vector; timestamps; array shape/dimension constraint. Save numeric floats, not the full provider response. Validate finite values, model, and dimensions in Rust. Upsert vector and mark job complete in the same transaction.

Use OpenAI `text-embedding-3-small` at the documented default 1,536 dimensions initially, with explicit model/dimension configuration; see the [OpenAI embedding guide](https://developers.openai.com/api/docs/guides/embeddings). Store model and source hash to support deliberate re-embedding. Start with PostgreSQL arrays to reuse the existing Railway PostgreSQL database without requiring an extension or database-image change; see [Railway extension guidance](https://docs.railway.com/databases/postgresql#extensions). If indexed message similarity search is later required, assess pgvector support on the existing database or plan an explicit database migration, then move to `vector` plus an appropriate index, following [pgvector](https://github.com/pgvector/pgvector). Array storage here does not promise indexed vector search.

Insert both jobs in the same transaction that marks the assistant completed. Skip empty content. Do not enqueue pending, failed, or interrupted assistants. If job insertion fails, roll back completion and emit the existing sanitized storage error, retaining persistence-before-`done`. Backfill policy for a user message whose assistant failed must be decided explicitly; the proposed live path waits for a completed assistant turn. Update the expected catalog in `curio-service/src/bin/curio_db.rs` and checks in `curio-service/ops/postgres/verify.sql`; verify application-role grants cover the new tables. The SQLite importer has already been retired. Continue to apply DDL only through `curio_db migrate` with migrator credentials.

Follow the current service layout: put message-embedding contracts, repository, and processor under `curio-service/src/domains/embeddings/`; put broker and OpenAI clients under `src/adapters/`, with no adapter-to-domain imports. Compose them through `src/app/` and a thin `curio-service/src/bin/curio_embed_worker.rs` in the existing Cargo package. Extend Chat's repository/service boundary to create the jobs. Preserve shared SQL-builder conventions, adding narrowly tested support for composite conflicts and `FOR UPDATE SKIP LOCKED` if needed. Do not add a root Cargo workspace.

The service-side dispatcher claims due unpublished jobs with a separate, fenced publication lease across API replicas. Record publication only after broker confirmation; a socket write or local client enqueue is insufficient. Uncertain outcomes leave publication retryable, including a crash after broker acceptance but before the database marker. Each intentional retry/replay advances a durable publication generation; transport retries of that generation retain its publication identity. The worker starts with one replica, reconnects with bounded backoff, recreates channels/consumers after reconnect, limits prefetch and processing concurrency, and leases jobs with `FOR UPDATE SKIP LOCKED` or equivalent. Hold no database lock during the provider call. Bound provider timeout below the processing lease duration and fence every result/retry write by lease token and expiry; an expired worker cannot commit over a new claimant. Broker acknowledgements never substitute for database fencing. On receipt or periodic scan, claim a job, fetch the completed row, check nonempty content and hash, call `/v1/embeddings` over HTTPS, then recheck source status/hash while committing. A stale result must not overwrite an embedding for edited content; edits supersede old jobs and enqueue a new hash, while deletes cascade. Current chat exposes no message editing, but retries still need this guard.

Retry transport failures, 429, and provider 5xx with exponential backoff and jitter. Classify invalid inputs/permanent failures as `dead`, cap attempts, and provide operator replay. Duplicate AMQP deliveries converge through the unique job key, lease, and idempotent `(message_id, model)` upsert. Expired leases and a bounded reconciliation scan recover from crashes and find eligible completed messages missing a job or vector. Reconciliation must honor the explicit failed-turn user backfill policy and must not silently revive `dead` jobs. Operator replay is scoped by job ID, resets bounded attempts, and revalidates saved content. Log only IDs, error class, attempts, lag, and duration; never log message content, provider bodies, secrets, or full broker payloads.

### RabbitMQ delivery and retry rules

Use a durable quorum queue, initially on one private broker node with a persistent volume. This provides persistence, not high availability; an HA rollout needs an explicitly configured multi-node quorum cluster with independent storage, rather than merely raising a Railway replica count. See [quorum queues](https://www.rabbitmq.com/docs/quorum-queues).

Use manual acknowledgements and per-consumer prefetch bounded by worker capacity. Acknowledge after the database commits `completed`, a scheduled `retry`, or a terminal state. A duplicate for a completed job, an active lease, or a future `available_at` can be acknowledged after authoritative lookup: PostgreSQL still owns recovery. If lookup/commit fails, leave unacknowledged or requeue with bounded consumer backoff; avoid a tight nack/requeue loop. Re-establish channel state and delivery handling after reconnect; delivery tags are channel-scoped.

PostgreSQL owns provider retry timing and the maximum attempt count. After committing `retry`, clear publication status and allow the dispatcher to send the next generation when due. Do not stack an independent broker provider-retry schedule on top of this. Malformed, oversized, or mismatched notifications are rejected without requeue into a bounded quarantine/dead-letter queue; never mutate a job based solely on their metadata. Configure the dead-letter exchange, binding, retention, and delivery-limit policy explicitly, and alert on quarantined messages. The database `dead` state remains the authoritative replay source; a dead-letter queue alone cannot guarantee recovery. See [dead-letter handling](https://www.rabbitmq.com/docs/dlx).

## Railway deployment

1. Reuse the existing PostgreSQL service. Add a separate private RabbitMQ service in the same Railway environment, with a pinned supported image, persistent volume at `/var/lib/rabbitmq`, and stable node identity. Provision a Curio-specific virtual host, separate producer/consumer credentials, and narrowly scoped permissions; bootstrap topology with a separate administrative identity. Keep AMQP and management ports private, with no public TCP proxy/domain. Bind listeners for Railway's private-network address family. See [Railway private networking](https://docs.railway.com/networking/private-networking).
2. Deploy the Rust binary as a separate Railway service rooted at `/curio-service`, with its own config such as `curio-service/railway-embedding-rabbitmq.toml` and explicit build/start commands. Keep the Axum service's `railway.toml`. Confirm the new binary is built and both processes verify the same migration. See Railway's [monorepo guidance](https://docs.railway.com/deployments/monorepo).
3. Set private DB URL, validated `CURIO_DB_SCHEMA`, `CURIO_RABBITMQ_URL` (secret AMQP URI with virtual host), exchange/routing-key/queue settings, and distinct broker credentials, `OPENAI_API_KEY`, model/dimensions, pool/concurrency limits, and retry bounds through Railway secrets/config. The worker gets a least-privilege application role, never the migrator URL or `VITE_*` variables. Budget the combined SQLx pools across replicas.
4. If Railway uses an HTTP healthcheck, give the worker a private liveness/readiness listener. Readiness requires DB migration verification and an active RabbitMQ consumer on the expected queue. Monitor pending/dead jobs, oldest job age, retries, broker connectivity, provider errors, and processing time.

Use a Tokio-compatible Rust AMQP client such as `lapin`, validating its current runtime/TLS integration when implementing and pinning the resolved dependency in the existing Cargo lockfile. Keep broker transport code in its adapter and provider/job logic in the domain.

## Implementation and retirement gates

| Phase | Work | Exit criterion |
| --- | --- | --- |
| 0. Inventory | Check live Railway/Cloudflare deployments, bindings, D1 row/assets counts, queue backlog, and retention needs. Confirm no client uses either Worker. | Decide whether D1 data/assets need import, export, or archival. |
| 1. Persistence | Add migration, DB grants, verification SQL, job/vector repositories, and provider validation. | Disposable PostgreSQL tests prove constraints, transaction boundaries, duplicates, cascade, and stale-result guards. |
| 2. Runtime | Add atomic completion/job insertion, dispatcher, RabbitMQ v1 contract, Rust worker, and replay/reconciliation. | End-to-end tests show `done` after DB commit, DB content lookup by ID, eventual vector with duplicate/lost RabbitMQ notifications, and chat success despite later embedding failure. |
| 3. Deployment | Migrate schema first; deploy broker/worker privately; enable producer with a server-side flag. Backfill nonempty completed PostgreSQL messages using bounded, resumable job insertion. | The live synthetic-message test passes against staging on Railway; staging and production backlog drains, vectors have expected model/dimension, and pause/replay is exercised. |
| 4. Retirement | Retain or migrate verified D1 data, then remove `curio-workers/` and unused Cloudflare deployments, bindings, and secrets. Update `AGENTS.md`, `README.md`, `docs/ARCHITECTURE.md`, `docs/deployment.md`, and CI/verification references. Keep older `plans/` as history. | No live integration or current guide depends on the Workers; recovery data is retained for the agreed window. |

If D1 contains attachments that must remain usable, or attachment embedding is required for parity, add an attachment import/integration phase before Phase 4: assess reuse of the existing PostgreSQL document storage, define ownership and stable file/version IDs, extraction per supported MIME type, size/security limits, an attachment job source, and tests. The old processor base64-encoded arbitrary bytes into a text embedding request; do not repeat that behavior. Record the no-assets finding if this phase is unnecessary.

## Verification and rollback

Run unit tests for the RabbitMQ payload schema, source-hash validation, embedding shape, retry classification, and sanitized errors. Use disposable `curio_test_*` PostgreSQL schemas for migration, atomic completion, lease competition, idempotent upsert, crash/retry, delete cascade, and backfill tests. A green `cargo test` without safe `CURIO_TEST_DATABASE_URL` is partial evidence. Separately exercise a real local broker and a fault-injection provider through disconnects before/after publish, duplicate/redelivered AMQP messages, unroutable publishes, lost confirms, broker outage, timeout, 429/5xx, and worker restart. Confirm `done` never waits for embedding.

### Required live integration test: synthetic input, real downstream calls

Provide an explicitly opt-in `embedding_live` integration target under `curio-service/tests/`. The only fabricated part of the acceptance scenario is the incoming message content and its fixture IDs. After fixture setup, use production paths and real services: no fake broker, mocked OpenAI response, in-memory repository, precomputed vector, or direct processor invocation that bypasses delivery.

1. Require an explicit live-test flag, real `OPENAI_API_KEY`, safe `CURIO_TEST_DATABASE_URL`, and isolated broker credentials/resources. Fail clearly when the explicitly requested live target lacks prerequisites; do not silently report a skipped live test as passing. Document that this test makes billable provider calls using synthetic, nonsensitive text.
2. Identify the existing Railway PostgreSQL instance and provision a dedicated test role with no access to application schemas. Create and migrate only a disposable `curio_test_*` schema. Use unique per-run broker resource names and fixture IDs. Never run this test with the production application's schema, queue/stream, or administrator credentials.
3. Start the actual worker binary against the isolated schema and broker resources, then await database-and-consumer readiness. For the delivery assertion, use a test configuration with the fallback scan delayed/disabled so a scan cannot mask broken broker consumption.
4. Seed a synthetic completed turn and its jobs through the real Chat repository completion transaction. Publish the v1 notification using the real broker adapter and wait for its broker confirmation. From receipt onward, the real worker must decode/validate, claim the job, fetch saved content, call OpenAI over HTTPS, persist the returned vector, complete the job, and acknowledge delivery.
5. Assert the job is `completed`, the source hash matches exact saved UTF-8 text, and exactly one `(message_id, model)` embedding exists with the configured model, finite values, and 1,536 dimensions by default. Assert database completion and broker acknowledgement; record only redacted identifiers/counts/timing. Exercise both user and assistant messages.
6. Send duplicate deliveries and restart the worker; verify the completed vector remains unique and unchanged. Run a separate missed-notification scenario with scans enabled and no publish, proving recovery performs real OpenAI and PostgreSQL calls. A crash during an external call may repeat a billable call; do not claim exactly-once provider execution.
7. Add an end-to-end producer case that sends a synthetic prompt to the actual chat HTTP route, uses the real OpenAI Responses API, and observes real dispatcher/broker/worker/embedding persistence. Assert jobs exist when `done` arrives and completion does not await the embedding path. This verifies the producer in addition to the incoming-notification scenario.
8. Stop only test-owned processes and delete only the per-run schema and broker resources. Execute cleanup on assertion failure too; document a targeted recovery command for cleanup interrupted by process termination.

Run locally with real broker/PostgreSQL dependencies and again with broker and worker inside a Railway staging environment using the existing database instance and isolated test schema. Keep provider fault-injection tests for deterministic 429/5xx/timeouts separate from this live acceptance test; they do not satisfy the requirement that every downstream call is real. Document the exact commands and pass/fail/skip results for each layer.

From `curio-service`, run `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo test`, database-backed tests with a safe test URL, and the new broker integration targets. Run `curio_db verify` only against an explicitly identified safe schema. Run frontend chat contract checks if the public HTTP/SSE contract changes. Check `git diff --check` and the final diff. As each phase lands, compare implementation with canonical `docs/ARCHITECTURE.md` and update current-state sections and `docs/deployment.md` when behavior or operations actually change.

Ship schema and worker before enabling the producer. Stop broker intake on shutdown and drain in-flight work within a deadline; unfinished claims recover through lease expiry. To stop a bad rollout, disable producer/worker while retaining jobs and vectors, then fix and replay. Do not edit an applied migration to roll back. Delete Cloudflare resources only after the Rust path and D1 retention decision pass the gates.
